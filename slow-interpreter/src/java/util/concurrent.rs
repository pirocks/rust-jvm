pub mod concurrent_hash_map {

    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{check_initing_or_inited_class, InterpreterStateGuard, JVMState, NewJavaValue};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};
    use crate::utils::run_static_or_virtual;

    pub struct ConcurrentHashMap<'gc_life> {
        normal_object: AllocatedObjectHandle<'gc_life>,
    }

    impl<'gc_life> AllocatedObjectHandle<'gc_life> {
        pub fn cast_concurrent_hash_map(self) -> ConcurrentHashMap<'gc_life> {
            ConcurrentHashMap { normal_object: self }
        }
    }

    impl<'gc_life> NewJavaValueHandle<'gc_life> {
        pub fn cast_concurrent_hash_map(self) -> Option<ConcurrentHashMap<'gc_life>> {
            Some(self.unwrap_object()?.cast_concurrent_hash_map())
        }
    }

    impl<'gc_life> ConcurrentHashMap<'gc_life> {
        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>) -> Self {
            let concurrent_hash_map_class = check_initing_or_inited_class(jvm, int_state, CClassName::concurrent_hash_map().into()).unwrap();
            let concurrent_hash_map = new_object(jvm, int_state, &concurrent_hash_map_class);
            run_constructor(jvm, int_state, concurrent_hash_map_class, vec![concurrent_hash_map.new_java_value()], &CMethodDescriptor::void_return(vec![])).unwrap();
            NewJavaValueHandle::Object(concurrent_hash_map).cast_concurrent_hash_map().expect("error creating hashmap")
        }

        pub fn table(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
            self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_table())
        }

        pub fn size_ctl(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
            self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_sizeCtl())
        }

        pub fn put_if_absent(&mut self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, key: NewJavaValue<'gc_life, '_>, value: NewJavaValue<'gc_life, '_>) -> NewJavaValueHandle<'gc_life> {
            let desc = CMethodDescriptor {
                arg_types: vec![CPDType::object(), CPDType::object()],
                return_type: CPDType::object(),
            };
            let properties_class = assert_inited_or_initing_class(jvm, CClassName::concurrent_hash_map().into());
            let args = vec![NewJavaValue::AllocObject(self.normal_object.as_allocated_obj()), key, value];
            let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_putIfAbsent(), &desc, args).unwrap();
            res.unwrap()
        }

        pub fn debug_print_table(&self, jvm: &'gc_life JVMState<'gc_life>) {
            let table = self.table(jvm);
            let array = table.unwrap_array(jvm);
            for (i, njv) in array.array_iterator().enumerate() {
                match njv.try_cast_concurrent_hash_map_node() {
                    None => {
                        eprintln!("#{} None", i);
                    }
                    Some(node) => {
                        let value = node.value(jvm).cast_string().unwrap();
                        let key = node.key(jvm).cast_string().unwrap();
                        eprintln!("#{} Key: {}, Value: {}", i, key.to_rust_string(jvm), value.to_rust_string(jvm));
                    }
                }
            }
        }
    }

    pub mod node {
        use rust_jvm_common::compressed_classfile::names::FieldName;

        use crate::JVMState;
        use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};

        pub struct Node<'gc_life> {
            normal_object: AllocatedObjectHandle<'gc_life>,
        }

        impl<'gc_life> AllocatedObjectHandle<'gc_life> {
            pub fn cast_concurrent_hash_map_node(self) -> Node<'gc_life> {
                Node { normal_object: self }
            }
        }

        impl<'gc_life> NewJavaValueHandle<'gc_life> {
            pub fn cast_concurrent_hash_map_node(self) -> Node<'gc_life> {
                Node { normal_object: self.unwrap_object_nonnull() }
            }

            pub fn try_cast_concurrent_hash_map_node(self) -> Option<Node<'gc_life>> {
                Some(Node { normal_object: self.unwrap_object()? })
            }
        }

        impl<'gc_life> Node<'gc_life> {
            pub fn key(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_key())
            }

            pub fn value(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_val())
            }
        }
    }
}

pub mod concurrent_hash_map {

    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{check_initing_or_inited_class, InterpreterStateGuard, JVMState, NewJavaValue};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};
    use crate::utils::run_static_or_virtual;

    pub struct ConcurrentHashMap<'gc> {
        normal_object: AllocatedObjectHandle<'gc>,
    }

    impl<'gc> AllocatedObjectHandle<'gc> {
        pub fn cast_concurrent_hash_map(self) -> ConcurrentHashMap<'gc> {
            ConcurrentHashMap { normal_object: self }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_concurrent_hash_map(self) -> Option<ConcurrentHashMap<'gc>> {
            Some(self.unwrap_object()?.cast_concurrent_hash_map())
        }
    }

    impl<'gc> ConcurrentHashMap<'gc> {
        pub fn new(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>) -> Self {
            let concurrent_hash_map_class = check_initing_or_inited_class(jvm, int_state, CClassName::concurrent_hash_map().into()).unwrap();
            let concurrent_hash_map = new_object(jvm, int_state, &concurrent_hash_map_class);
            run_constructor(jvm, int_state, concurrent_hash_map_class, vec![concurrent_hash_map.new_java_value()], &CMethodDescriptor::void_return(vec![])).unwrap();
            NewJavaValueHandle::Object(concurrent_hash_map).cast_concurrent_hash_map().expect("error creating hashmap")
        }

        pub fn table(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_table())
        }

        pub fn size_ctl(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_sizeCtl())
        }

        pub fn put_if_absent(&mut self, jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, key: NewJavaValue<'gc, '_>, value: NewJavaValue<'gc, '_>) -> NewJavaValueHandle<'gc> {
            let desc = CMethodDescriptor {
                arg_types: vec![CPDType::object(), CPDType::object()],
                return_type: CPDType::object(),
            };
            let properties_class = assert_inited_or_initing_class(jvm, CClassName::concurrent_hash_map().into());
            let args = vec![NewJavaValue::AllocObject(self.normal_object.as_allocated_obj()), key, value];
            let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_putIfAbsent(), &desc, args).unwrap();
            res.unwrap()
        }

        pub fn debug_print_table(&self, jvm: &'gc JVMState<'gc>) {
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

        pub struct Node<'gc> {
            normal_object: AllocatedObjectHandle<'gc>,
        }

        impl<'gc> AllocatedObjectHandle<'gc> {
            pub fn cast_concurrent_hash_map_node(self) -> Node<'gc> {
                Node { normal_object: self }
            }
        }

        impl<'gc> NewJavaValueHandle<'gc> {
            pub fn cast_concurrent_hash_map_node(self) -> Node<'gc> {
                Node { normal_object: self.unwrap_object_nonnull() }
            }

            pub fn try_cast_concurrent_hash_map_node(self) -> Option<Node<'gc>> {
                Some(Node { normal_object: self.unwrap_object()? })
            }
        }

        impl<'gc> Node<'gc> {
            pub fn key(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_key())
            }

            pub fn value(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_val())
            }
        }
    }
}

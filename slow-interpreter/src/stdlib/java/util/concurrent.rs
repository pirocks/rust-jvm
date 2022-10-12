pub mod concurrent_hash_map {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;

    use crate::{check_initing_or_inited_class, JVMState, NewJavaValue, PushableFrame};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_util::{new_object_full, run_constructor};
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::utils::run_static_or_virtual;

    pub struct ConcurrentHashMap<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_concurrent_hash_map(self) -> Option<ConcurrentHashMap<'gc>> {
            Some(self.unwrap_object()?.cast_concurrent_hash_map())
        }
    }

    impl<'gc> ConcurrentHashMap<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Self {
            let concurrent_hash_map_class = check_initing_or_inited_class(jvm, int_state, CClassName::concurrent_hash_map().into()).unwrap();
            let concurrent_hash_map = new_object_full(jvm, int_state, &concurrent_hash_map_class);
            run_constructor(jvm, int_state, concurrent_hash_map_class, vec![concurrent_hash_map.new_java_value()], &CMethodDescriptor::void_return(vec![])).unwrap();
            NewJavaValueHandle::Object(concurrent_hash_map).cast_concurrent_hash_map().expect("error creating hashmap")
        }

        pub fn table(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_table())
        }

        pub fn size_ctl(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_sizeCtl())
        }

        pub fn put_if_absent(&mut self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, key: NewJavaValue<'gc, '_>, value: NewJavaValue<'gc, '_>) -> NewJavaValueHandle<'gc> {
            let desc = CMethodDescriptor {
                arg_types: vec![CPDType::object(), CPDType::object()],
                return_type: CPDType::object(),
            };
            let properties_class = assert_inited_or_initing_class(jvm, CClassName::concurrent_hash_map().into());
            let args = vec![self.normal_object.new_java_value(), key, value];
            let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_putIfAbsent(), &desc, args).unwrap();
            res.unwrap()
        }

        pub fn get(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, key: NewJavaValue<'gc, '_>) -> NewJavaValueHandle<'gc> {
            let desc = CMethodDescriptor {
                arg_types: vec![CPDType::object()],
                return_type: CPDType::object(),
            };
            let properties_class = assert_inited_or_initing_class(jvm, CClassName::concurrent_hash_map().into());
            let args = vec![self.normal_object.new_java_value(), key];
            let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_get(), &desc, args).unwrap();
            res.unwrap()
        }

        pub fn debug_print_table(&self, jvm: &'gc JVMState<'gc>) -> Option<()> {
            let table = self.table(jvm);
            let nonnull = table.unwrap_object()?;
            let array = nonnull.unwrap_array();
            for (i, njv) in array.array_iterator().enumerate() {
                match njv.try_cast_concurrent_hash_map_node() {
                    None => {
                        eprintln!("#{} None", i);
                    }
                    Some(node) => {
                        // let value = node.value(jvm).cast_string().unwrap();
                        // let key = node.key(jvm).cast_string().unwrap();
                        let raw_key = node.key(jvm).to_interpreter_jv().to_raw();
                        let raw_value = node.value(jvm).to_interpreter_jv().to_raw();
                        eprintln!("#{} Key: {:X}, Value: {:X}", i, raw_key, raw_value/*key.to_rust_string(jvm), value.to_rust_string(jvm)*/);
                    }
                }
            }
            Some(())
        }
    }

    pub mod node {
        use rust_jvm_common::compressed_classfile::class_names::CClassName;
        use rust_jvm_common::compressed_classfile::field_names::FieldName;
        use crate::class_loading::assert_inited_or_initing_class;
        use crate::JVMState;
        use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
        use crate::new_java_values::NewJavaValueHandle;

        pub struct Node<'gc> {
            pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
        }

        impl<'gc> NewJavaValueHandle<'gc> {
            pub fn try_cast_concurrent_hash_map_node(self) -> Option<Node<'gc>> {
                Some(Node { normal_object: self.unwrap_object()?.unwrap_normal_object() })
            }
        }

        impl<'gc> Node<'gc> {
            pub fn key(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
                let rc = assert_inited_or_initing_class(jvm, CClassName::concurrent_hash_map_node().into());
                self.normal_object.get_var(jvm, &rc, FieldName::field_key())
            }

            pub fn value(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
                let rc = assert_inited_or_initing_class(jvm, CClassName::concurrent_hash_map_node().into());
                self.normal_object.get_var(jvm, &rc, FieldName::field_val())
            }
        }
    }
}

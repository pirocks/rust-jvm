pub mod concurrent_hash_map {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::{check_initing_or_inited_class, InterpreterStateGuard, JVMState};
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};

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
        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life,'_>) -> Self{
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
    }

    pub mod node{
        use rust_jvm_common::compressed_classfile::names::FieldName;
        use crate::java::util::concurrent::concurrent_hash_map::ConcurrentHashMap;
        use crate::JVMState;
        use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};

        pub struct Node<'gc_life> {
            normal_object: AllocatedObjectHandle<'gc_life>,
        }

        impl<'gc_life> AllocatedObjectHandle<'gc_life> {
            pub fn cast_node(self) -> ConcurrentHashMap<'gc_life> {
                ConcurrentHashMap { normal_object: self }
            }
        }

        impl<'gc_life> Node<'gc_life> {
            pub fn key(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_key())
            }

            pub fn value(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_value())
            }
        }
    }
}

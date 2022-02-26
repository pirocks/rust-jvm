pub mod pre_hashed_map{
    use itertools::Itertools;
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::java_values::GcManagedObject;
    use crate::JVMState;
    use crate::new_java_values::AllocatedObjectHandle;
    use crate::new_java_values::array_wrapper::ArrayWrapper;

    pub struct PreHashedMap<'gc_life> {
        handle: AllocatedObjectHandle<'gc_life>
    }

    impl <'gc_life> AllocatedObjectHandle<'gc_life>{
        pub fn cast_pre_hashed_map(self) -> PreHashedMap<'gc_life>{
            PreHashedMap{ handle: self }
        }
    }

    impl <'gc_life> PreHashedMap<'gc_life> {
        pub fn ht(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<Vec<Option<Vec<Option<AllocatedObjectHandle<'gc_life>>>>>> {
            let current_class_pointer = assert_inited_or_initing_class(jvm, CClassName::pre_hashed_map().into());
            let ht = self.handle.as_allocated_obj().get_var(jvm,&current_class_pointer, FieldName::field_ht());
            let object_handle = ht.unwrap_object()?;
            let top_leve_array = object_handle.unwrap_array(jvm);
            Some(top_leve_array.array_iterator().map(|handle|{
                let obj = handle.unwrap_object()?;
                let mut inner_array_res = vec![];
                let inner_array = obj.unwrap_array(jvm);
                for inner_elem in inner_array.array_iterator(){
                    inner_array_res.push(inner_elem.unwrap_object());
                }
                Some(inner_array_res)
            }).collect_vec())
        }
    }
}

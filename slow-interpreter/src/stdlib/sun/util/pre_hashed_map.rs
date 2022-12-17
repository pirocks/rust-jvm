use itertools::Itertools;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{AllocatedHandle, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;

pub struct PreHashedMap<'gc> {
    handle: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> PreHashedMap<'gc> {
    pub fn ht(&self, jvm: &'gc JVMState<'gc>) -> Option<Vec<Option<Vec<Option<AllocatedHandle<'gc>>>>>> {
        let current_class_pointer = assert_inited_or_initing_class(jvm, CClassName::pre_hashed_map().into());
        let ht = self.handle.get_var(jvm, &current_class_pointer, FieldName::field_ht());
        let object_handle = ht.unwrap_object()?;
        let top_leve_array = object_handle.unwrap_array();
        Some(top_leve_array.array_iterator().map(|handle| {
            let obj = handle.unwrap_object()?;
            let mut inner_array_res = vec![];
            let inner_array = obj.unwrap_array();
            for inner_elem in inner_array.array_iterator() {
                inner_array_res.push(inner_elem.unwrap_object());
            }
            Some(inner_array_res)
        }).collect_vec())
    }
}

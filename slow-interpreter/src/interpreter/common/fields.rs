use std::ops::Deref;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{JVMState, PushableFrame, WasException};
use crate::class_loading::assert_inited_or_initing_class;
use crate::new_java_values::NewJavaValueHandle;
use crate::static_vars::static_vars;

pub(crate) fn get_static_impl<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    field_class_name: CClassName,
    field_name: FieldName,
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException<'gc>> {
    let target_classfile = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    //todo handle interfaces in setting as well
    let temp = static_vars(target_classfile.deref(), jvm);
    let attempted_get = temp.try_get(field_name);
    match attempted_get {
        None => {
            let possible_super = target_classfile.view().super_name();
            if let Some(super_) = possible_super {
                return get_static_impl(jvm, int_state, super_, field_name).into();
            }
        }
        Some(val) => {
            return Ok(val.into());
        }
    };
    for interfaces in target_classfile.view().interfaces() {
        let interface_lookup_res = get_static_impl(jvm, int_state, interfaces.interface_name(), field_name.clone())?;
        if interface_lookup_res.is_some() {
            return Ok(interface_lookup_res);
        }
    }
    panic!()
}

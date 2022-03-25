use rust_jvm_common::compressed_classfile::{CFieldDescriptor, CompressedFieldDescriptor};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::interpreter::WasException;
use crate::java_values::JavaValue;
use crate::new_java_values::NewJavaValueHandle;
use crate::utils::throw_npe;


pub(crate) fn get_static_impl<'gc_life, 'l>(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>,
    field_class_name: CClassName,
    field_name: FieldName
) -> Result<Option<NewJavaValueHandle<'gc_life>>, WasException> {
    let target_classfile = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    //todo handle interfaces in setting as well
    let temp = target_classfile.static_vars(jvm);
    let attempted_get = temp.try_get(field_name);
    match attempted_get {
        None => {
            let possible_super = target_classfile.view().super_name();
            if let Some(super_) = possible_super {
                return get_static_impl(jvm, int_state, super_, field_name).into();
            }
        }
        Some(val) => {
            return Ok(val.into())
        },
    };
    for interfaces in target_classfile.view().interfaces() {
        let interface_lookup_res = get_static_impl(jvm, int_state, interfaces.interface_name(), field_name.clone())?;
        if interface_lookup_res.is_some() {
            return Ok(interface_lookup_res);
        }
    }
    panic!()
}

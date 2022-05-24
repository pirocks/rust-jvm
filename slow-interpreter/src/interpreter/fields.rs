use std::mem::size_of;
use std::ops::Deref;
use jvmti_jni_bindings::jlong;
use rust_jvm_common::compressed_classfile::{CFieldDescriptor, CompressedFieldDescriptor};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;
use stage0::compiler::fields::recursively_find_field_number_and_type;

use crate::{JVMState};
use crate::class_loading::{assert_inited_or_initing_class};
use crate::interpreter::real_interpreter_state::{InterpreterJavaValue, RealInterpreterStateGuard};
use crate::interpreter::{PostInstructionAction};
use crate::runtime_class::static_vars;

//
pub fn putstatic<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc,'l,'k>, field_class_name: CClassName, field_name: FieldName, field_descriptor: &CFieldDescriptor) -> PostInstructionAction<'gc> {
    let target_classfile = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    let mut entry_mut = int_state.current_frame_mut();
    let field_value = entry_mut.pop(field_descriptor.0.to_runtime_type().unwrap());
    static_vars(target_classfile.deref(), jvm).set(field_name, field_value.to_new_java_handle(jvm));
    PostInstructionAction::Next {}
}

pub fn putfield<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc,'l,'k>, field_class_name: CClassName, field_name: FieldName, field_descriptor: &CFieldDescriptor) -> PostInstructionAction<'gc>{
    let CompressedFieldDescriptor(field_type) = field_descriptor;
    let target_class = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    let (field_number,_) = recursively_find_field_number_and_type(target_class.unwrap_class_class(),field_name);
    let mut entry_mut = int_state.current_frame_mut();
    let val = entry_mut.pop(field_type.to_runtime_type().unwrap());
    let object_ref = entry_mut.pop(RuntimeType::object());
    match object_ref {
        InterpreterJavaValue::Object(o) => {
            unsafe {
                match o {
                    Some(x) => {
                        let raw_field_ptr = x.as_ptr().add(field_number.0 as usize * size_of::<jlong>()) as *mut u64;
                        raw_field_ptr.write(val.to_raw());
                    },
                    None => {
                        todo!()/*return throw_npe(jvm, int_state);*/
                    }
                }
            }
        }
        _ => {
            dbg!(object_ref);
            panic!()
        }
    };
    PostInstructionAction::Next {}
}
//
// pub fn get_static(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, field_class_name: CClassName, field_name: FieldName, _field_descriptor: &CFieldDescriptor) {
//     //todo make sure class pointer is updated correctly
//     let field_value = match match get_static_impl(jvm, int_state, field_class_name, field_name) {
//         Ok(val) => val,
//         Err(WasException {}) => return,
//     } {
//         None => {
//             return;
//         }
//         Some(val) => val,
//     };
//     int_state.push_current_operand_stack(field_value);
// }
//
// fn get_static_impl(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, field_class_name: CClassName, field_name: FieldName) -> Result<Option<JavaValue<'gc>>, WasException> {
//     let target_classfile = check_initing_or_inited_class(jvm, int_state, field_class_name.clone().into())?;
//     //todo handle interfaces in setting as well
//     for interfaces in target_classfile.view().interfaces() {
//         let interface_lookup_res = get_static_impl(jvm, int_state, interfaces.interface_name(), field_name.clone())?;
//         if interface_lookup_res.is_some() {
//             return Ok(interface_lookup_res);
//         }
//     }
//     let temp = target_classfile.static_vars();
//     let attempted_get = temp.get(&field_name);
//     let field_value = match attempted_get {
//         None => {
//             let possible_super = target_classfile.view().super_name();
//             match possible_super {
//                 None => None,
//                 Some(super_) => {
//                     return get_static_impl(jvm, int_state, super_, field_name).into();
//                 }
//             }
//         }
//         Some(val) => val.clone().into(),
//     };
//     Ok(field_value)
// }
//
// pub fn get_field(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, field_class_name: CClassName, field_name: FieldName, _field_desc: &CompressedFieldDescriptor, _debug: bool) {
//     let target_class_pointer = assert_inited_or_initing_class(jvm, field_class_name.into());
//     let object_ref = int_state.current_frame_mut().pop(Some(RuntimeType::object()));
//     match object_ref {
//         JavaValue::Object(o) => {
//             let res = o.unwrap().unwrap_normal_object().get_var(jvm, target_class_pointer, field_name);
//             int_state.current_frame_mut().push(res);
//         }
//         _ => panic!(),
//     }
// }

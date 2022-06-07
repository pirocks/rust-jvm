use std::mem::size_of;
use std::ops::Deref;
use another_jit_vm_ir::WasException;
use jvmti_jni_bindings::jlong;
use rust_jvm_common::compressed_classfile::{CFieldDescriptor, CompressedFieldDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;
use stage0::compiler::fields::recursively_find_field_number_and_type;

use crate::{check_initing_or_inited_class, JVMState, NewJavaValueHandle};
use crate::class_loading::{assert_inited_or_initing_class};
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue, RealInterpreterStateGuard};
use crate::interpreter::{PostInstructionAction};
use crate::runtime_class::static_vars;

//
pub fn putstatic<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, field_class_name: CClassName, field_name: FieldName, field_descriptor: &CFieldDescriptor) -> PostInstructionAction<'gc> {
    let mut entry_mut = int_state.current_frame_mut();
    let target_classfile = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    let field_value = entry_mut.pop(field_descriptor.0.to_runtime_type().unwrap());
    static_vars(target_classfile.deref(), jvm).set(field_name, field_value.to_new_java_handle(jvm));
    PostInstructionAction::Next {}
}

pub fn putfield<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, field_class_name: CClassName, field_name: FieldName, field_descriptor: &CFieldDescriptor) -> PostInstructionAction<'gc> {
    let mut entry_mut = int_state.current_frame_mut();
    let CompressedFieldDescriptor(field_type) = field_descriptor;
    let target_class = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    let field_number = recursively_find_field_number_and_type(target_class.unwrap_class_class(), field_name).number;
    let val = entry_mut.pop(field_type.to_runtime_type().unwrap());
    let object_ref = entry_mut.pop(RuntimeType::object());
    match object_ref {
        InterpreterJavaValue::Object(o) => {
            unsafe {
                match o {
                    Some(x) => {
                        let raw_field_ptr = x.as_ptr().add(field_number.0 as usize * size_of::<jlong>()) as *mut u64;
                        assert_ne!(val.to_raw(), 0xDDDDDDDDDDDDDDDD);
                        raw_field_ptr.write(val.to_raw());
                    }
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
pub fn get_static<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, field_class_name: CClassName, field_name: FieldName, field_descriptor: &CFieldDescriptor) ->PostInstructionAction<'gc>  {  //todo make sure class pointer is updated correctly
    let field_value = match match get_static_impl(jvm, int_state, field_class_name, field_name,field_descriptor.0) {
        Ok(val) => val,
        Err(WasException {}) => return PostInstructionAction::Exception { exception: WasException{} },
    } {
        None => {
            todo!()
        }
        Some(val) => val,
    };
    int_state.current_frame_mut().push(field_value.to_interpreter_jv());
    PostInstructionAction::Next {}
}

fn get_static_impl<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, field_class_name: CClassName, field_name: FieldName, cpdtype: CPDType) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let target_classfile = check_initing_or_inited_class(jvm, int_state.inner(), field_class_name.clone().into())?;
    //todo handle interfaces in setting as well
    for interfaces in target_classfile.view().interfaces() {
        let interface_lookup_res = get_static_impl(jvm, int_state, interfaces.interface_name(), field_name.clone(),cpdtype)?;
        if interface_lookup_res.is_some() {
            return Ok(interface_lookup_res);
        }
    }
   /* let temp = todo!()/*target_classfile.unwrap_class_class().static_vars.read().unwrap()*/;
    let attempted_get = todo!()/*temp.get(&field_name)*/;
    let field_value = match attempted_get {
        None => {
            let possible_super = target_classfile.view().super_name();
            match possible_super {
                None => None,
                Some(super_) => {
                    return get_static_impl(jvm, int_state, super_, field_name,cpdtype).into();
                }
            }
        }
        Some(val) => Some(native_to_new_java_value(val.clone().into(),cpdtype,jvm))
    };
    Ok(field_value)*/
    todo!()
}

pub fn get_field<'gc, 'k, 'l, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, field_class_name: CClassName, field_name: FieldName, field_desc: &CompressedFieldDescriptor) -> PostInstructionAction<'gc>{
    let target_class = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    let field_number = recursively_find_field_number_and_type(target_class.unwrap_class_class(), field_name).number;
    let object_ref = current_frame.pop(RuntimeType::object());
    unsafe {
        match object_ref {
            InterpreterJavaValue::Object(Some(x)) => {
                let raw_field_ptr = x.as_ptr().add(field_number.0 as usize * size_of::<jlong>()) as *mut u64;
                let res = InterpreterJavaValue::from_raw(raw_field_ptr.read(), field_desc.0.to_runtime_type().unwrap());
                current_frame.push(res);
                PostInstructionAction::Next {}
            }
            _ => panic!(),
        }
    }
}
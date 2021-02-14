use std::sync::Arc;

use rust_jvm_common::classnames::ClassName;
use verification::verifier::instructions::special::extract_field_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::java_values::JavaValue;

pub fn putstatic(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let view = &int_state.current_class_view();
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let target_classfile = assert_inited_or_initing_class(jvm, int_state, field_class_name.clone().into());
    let stack = int_state.current_frame_mut().operand_stack_mut();
    let field_value = stack.pop().unwrap();
    if field_name.as_str() == "NF_internalMemberName" {
        field_value.unwrap_object().unwrap();
    }
    target_classfile.static_vars().insert(field_name, field_value);
}

pub fn putfield(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let view = &int_state.current_class_view();
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let _target_classfile = check_initing_or_inited_class(jvm, int_state, field_class_name.clone().into());//todo sus this should be assert
    let stack = &mut int_state.current_frame_mut().operand_stack_mut();
    let val = stack.pop().unwrap();
    let object_ref = stack.pop().unwrap();
    match object_ref {
        JavaValue::Object(o) => {
            {
                o.unwrap().unwrap_normal_object().fields_mut().insert(field_name, val);
            }
        }
        _ => {
            dbg!(object_ref);
            panic!()
        }
    }
}

pub fn get_static(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    //todo make sure class pointer is updated correctly

    let view = &int_state.current_class_view();
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let field_value = match get_static_impl(jvm, int_state, &field_class_name, &field_name) {
        None => { return; }
        Some(val) => val
    };
    int_state.push_current_operand_stack(field_value);
}

fn get_static_impl(jvm: &JVMState, int_state: &mut InterpreterStateGuard, field_class_name: &ClassName, field_name: &str) -> Option<JavaValue> {
    let target_classfile = check_initing_or_inited_class(jvm, int_state, field_class_name.clone().into()).unwrap();
    //todo handle interfaces in setting as well
    for interfaces in target_classfile.view().interfaces() {
        let interface_lookup_res = get_static_impl(jvm, int_state, &interfaces.interface_name(), field_name);
        if interface_lookup_res.is_some() {
            return interface_lookup_res;
        }
    }
    let temp = target_classfile.static_vars();
    let attempted_get = temp.get(field_name);
    let field_value = match attempted_get {
        None => {
            let possible_super = target_classfile.view().super_name();
            match possible_super {
                None => None,
                Some(super_) => { return get_static_impl(jvm, int_state, &super_, field_name).into(); }
            }
        }
        Some(val) => {
            val.clone().into()
        }
    };
    field_value
}

pub fn get_field(int_state: &mut InterpreterStateGuard, cp: u16, _debug: bool) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let view = &current_frame.class_pointer().view();
    let (_field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let object_ref = current_frame.pop();
    match object_ref {
        JavaValue::Object(o) => {
            let fields = match o.as_ref() {
                Some(x) => x,
                None => {
                    int_state.print_stack_trace();
                    unimplemented!()
                }
            }.unwrap_normal_object().fields_mut();
            if fields.get(field_name.as_str()).is_none() {
                dbg!(&o);
                dbg!(&fields.keys());
                dbg!(&field_name);
            }
            let res = match fields.get(field_name.as_str()) {
                Some(x) => x,
                None => {
                    int_state.print_stack_trace();
                    panic!()
                },
            }.clone();
            current_frame.push(res);
        }
        _ => panic!(),
    }
}


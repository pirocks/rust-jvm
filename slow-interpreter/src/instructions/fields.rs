use classfile_view::loading::LoaderArc;
use rust_jvm_common::classnames::ClassName;
use verification::verifier::instructions::special::extract_field_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;

pub fn putstatic<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, cp: u16) -> () {
    let view = &int_state.current_class_view();
    let loader_arc = &int_state.current_loader(jvm);
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let target_classfile = check_inited_class(jvm, int_state, &field_class_name.into(), loader_arc.clone());
    let stack = &mut int_state.current_frame_mut().operand_stack;
    let field_value = stack.pop().unwrap();
    target_classfile.static_vars().insert(field_name, field_value);
}

pub fn putfield<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, cp: u16) -> () {
    let view = &int_state.current_class_view();
    let loader_arc = &int_state.current_loader(jvm);
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let _target_classfile = check_inited_class(jvm, int_state, &field_class_name.into(), loader_arc.clone());
    let stack = &mut int_state.current_frame_mut().operand_stack;
    let val = stack.pop().unwrap();
    let object_ref = stack.pop().unwrap();
    match object_ref {
        JavaValue::Object(o) => {
            {
                o.unwrap().unwrap_normal_object().fields.borrow_mut().insert(field_name, val);
            }
        }
        _ => {
            dbg!(object_ref);
            panic!()
        }
    }
}

pub fn get_static<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, cp: u16) -> () {
    //todo make sure class pointer is updated correctly

    let view = &int_state.current_class_view();
    let loader_arc = &int_state.current_loader(jvm);
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    get_static_impl(jvm, int_state, cp, loader_arc, &field_class_name, &field_name);
}

fn get_static_impl<'l>(state: &'static JVMState, int_state: &mut InterpreterStateGuard, cp: u16, loader_arc: &LoaderArc, field_class_name: &ClassName, field_name: &String) {
    let target_classfile = check_inited_class(state, int_state, &field_class_name.clone().into(), loader_arc.clone());
    let temp = target_classfile.static_vars();
    let attempted_get = temp.get(field_name);
    let field_value = match attempted_get {
        None => {
            return get_static_impl(state, int_state, cp, loader_arc, &target_classfile.view().super_name().unwrap(), field_name);
        }
        Some(val) => {
            val.clone()
        }
    };
    let stack = &mut int_state.current_frame_mut().operand_stack;
    stack.push(field_value);
}

pub fn get_field(current_frame: &mut StackEntry, cp: u16, _debug: bool) -> () {
    let view = &current_frame.class_pointer.view();
    let (_field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let object_ref = current_frame.pop();
    match object_ref {
        JavaValue::Object(o) => {
            let fields = o.as_ref().unwrap().unwrap_normal_object().fields.borrow();
            if fields.get(field_name.as_str()).is_none() {
                dbg!(&o);
                dbg!(&fields.keys());
            }
            let res = fields.get(field_name.as_str()).unwrap().clone();
            current_frame.push(res);
        }
        _ => panic!(),
    }
}


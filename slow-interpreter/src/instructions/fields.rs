use runtime_common::{InterpreterState, StackEntry};
use crate::interpreter_util::check_inited_class;
use std::rc::Rc;
use verification::verifier::instructions::special::extract_field_descriptor;
use runtime_common::java_values::JavaValue;


pub fn putstatic(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) -> () {
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
    let target_classfile = check_inited_class(state, &field_class_name, current_frame.clone().into(), loader_arc.clone());
    let mut stack = current_frame.operand_stack.borrow_mut();
    let field_value = stack.pop().unwrap();
    target_classfile.static_vars.borrow_mut().insert(field_name, field_value);
}

pub fn putfield(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) -> () {
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
    let _target_classfile = check_inited_class(state, &field_class_name, current_frame.clone().into(), loader_arc.clone());
    let mut stack = current_frame.operand_stack.borrow_mut();
    let val = stack.pop().unwrap();
    let object_ref = stack.pop().unwrap();
    match object_ref {
        JavaValue::Object(o) => {
            {
                o.unwrap().unwrap_object().fields.borrow_mut().insert(field_name, val);
            }
        }
        _ => {
            dbg!(object_ref);
            panic!()
        }
    }
}

pub fn get_static(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) -> () {
    //todo make sure class pointer is updated correctly

    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
    let target_classfile = check_inited_class(state, &field_class_name, current_frame.clone().into(), loader_arc.clone());
    let field_value = target_classfile.static_vars.borrow().get(&field_name).unwrap().clone();
    let mut stack = current_frame.operand_stack.borrow_mut();
    stack.push(field_value);
}

pub fn get_field(current_frame: &Rc<StackEntry>, cp: u16) -> () {
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (_field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
    let object_ref = current_frame.pop();
    match object_ref {
        JavaValue::Object(o) => {
//            dbg!(_field_class_name);
//            dbg!(_field_descriptor);
//            dbg!(&field_name);
            let fields = o.as_ref().unwrap().unwrap_object().fields.borrow();
            let res = fields.get(field_name.as_str()).unwrap().clone();
            current_frame.push(res);
        }
        _ => panic!(),
    }
}


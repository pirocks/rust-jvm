use runtime_common::{StackEntry, InterpreterState};
use std::rc::Rc;
use runtime_common::java_values::JavaValue;
use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::ClassName;

pub fn arraylength(current_frame: &Rc<StackEntry>) -> () {
    let array = current_frame.pop();
    match array {
        JavaValue::Array(a) => {
            current_frame.push(JavaValue::Int(a.unwrap().borrow().len() as i32));
        }
        _ => panic!()
    }
}

pub fn invoke_instanceof(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16){
    let object = current_frame.pop().unwrap_object();
    if object.is_none(){
        current_frame.push(JavaValue::Int(0));
        return;
    }
    let classfile = &current_frame.class_pointer.classfile;
    let instance_of_class_name = classfile.extract_class_from_constant_pool_name(cp);
    let instanceof_class = check_inited_class(state, &ClassName::Str(instance_of_class_name), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
    unimplemented!()
}
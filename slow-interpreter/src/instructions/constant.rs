use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn fconst_0(current_frame: &Rc<StackEntry>) {
    current_frame.push(JavaValue::Float(0.0));
}

pub fn bipush(current_frame: &Rc<StackEntry>, b: u8) -> () {
    current_frame.push(JavaValue::Int(b as i32))
}

pub fn sipush(current_frame: &Rc<StackEntry>, val: u16) {
    current_frame.push(JavaValue::Int(val as i32));
}


pub fn aconst_null(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Object(None))
}
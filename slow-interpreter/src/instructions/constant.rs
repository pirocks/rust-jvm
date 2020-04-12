use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn fconst_0(current_frame: & StackEntry) {
    current_frame.push(JavaValue::Float(0.0));
}

pub fn fconst_1(current_frame: & StackEntry) {
    current_frame.push(JavaValue::Float(1.0));
}


pub fn bipush(current_frame: & StackEntry, b: u8) -> () {
    current_frame.push(JavaValue::Int(b as i32))
}

pub fn sipush(current_frame: & StackEntry, val: u16) {
    current_frame.push(JavaValue::Int(val as i32));
}


pub fn aconst_null(current_frame: & StackEntry) -> () {
    current_frame.push(JavaValue::Object(None))
}
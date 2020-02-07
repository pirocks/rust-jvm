use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn i2l(current_frame: &Rc<StackEntry>) -> () {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Long(int as i64));
}

pub fn i2f(current_frame: &Rc<StackEntry>) -> () {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Float(int as f32));
}

pub fn i2c(current_frame: &Rc<StackEntry>) -> () {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(int as u8 as char as i32));
}


pub fn f2i(current_frame: &Rc<StackEntry>) -> () {
    let f = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Int(f as i32))
}
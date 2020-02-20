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


pub fn l2f(current_frame: &Rc<StackEntry>) -> () {
    let long = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Float(long as f32));
}

pub fn l2i(current_frame: &Rc<StackEntry>) -> () {
    let long = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Int(long as i32));
}


pub fn i2d(current_frame: &Rc<StackEntry>) -> () {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Double(int as f64));
}


pub fn i2c(current_frame: &Rc<StackEntry>) -> () {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(int as u8 as char as i32));
}


pub fn i2b(current_frame: &Rc<StackEntry>) -> () {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(int as u8 as i32));
}


pub fn f2i(current_frame: &Rc<StackEntry>) -> () {
    let f = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Int(f as i32))
}

pub fn f2d(current_frame: &Rc<StackEntry>) -> () {
    let f = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Double(f as f64))
}

pub fn d2i(current_frame: &Rc<StackEntry>) -> () {
    let f = current_frame.pop().unwrap_double();
    current_frame.push(JavaValue::Int(f as i32))
}


pub fn d2l(current_frame: &Rc<StackEntry>) -> () {
    let f = current_frame.pop().unwrap_double();
    current_frame.push(JavaValue::Long(f as i64))
}

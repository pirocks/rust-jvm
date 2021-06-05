use crate::java_values::JavaValue;
use crate::stack_entry::StackEntryMut;

pub fn i2l(mut current_frame: StackEntryMut) {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Long(int as i64));
}

pub fn i2s(mut current_frame: StackEntryMut) {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Short(int as i16));
}

pub fn i2f(mut current_frame: StackEntryMut) {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Float(int as f32));
}


pub fn l2f(mut current_frame: StackEntryMut) {
    let long = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Float(long as f32));
}

pub fn l2i(mut current_frame: StackEntryMut) {
    let long = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Int(long as i32));
}


pub fn i2d(mut current_frame: StackEntryMut) {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Double(int as f64));
}


pub fn i2c(mut current_frame: StackEntryMut) {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(int as u8 as char as i32));
}


pub fn i2b(mut current_frame: StackEntryMut) {
    let int = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(int as u8 as i32));
}


pub fn f2i(mut current_frame: StackEntryMut) {
    let f = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Int(f as i32))
}

pub fn f2d(mut current_frame: StackEntryMut) {
    let f = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Double(f as f64))
}

pub fn d2i(mut current_frame: StackEntryMut) {
    let f = current_frame.pop().unwrap_double();
    current_frame.push(JavaValue::Int(f as i32))
}


pub fn d2l(mut current_frame: StackEntryMut) {
    let f = current_frame.pop().unwrap_double();
    current_frame.push(JavaValue::Long(f as i64))
}

pub fn d2f(mut current_frame: StackEntryMut) {
    let f = current_frame.pop().unwrap_double();
    current_frame.push(JavaValue::Float(f as f32))
}

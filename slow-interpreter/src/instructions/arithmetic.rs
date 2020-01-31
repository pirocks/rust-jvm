use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn fmul(current_frame: Rc<StackEntry>) -> () {
    let value2 = current_frame.pop().unwrap_float();
    let value1 = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Float(value2 * value1));
}


pub fn land(current_frame: Rc<StackEntry>) -> () {
    let first = current_frame.pop().unwrap_long();
    let second = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(first & second))
}

pub fn iand(current_frame: &Rc<StackEntry>) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(first & second))
}


pub fn ixor(current_frame: &Rc<StackEntry>) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(first ^ second))
}


pub fn iadd(current_frame: &Rc<StackEntry>) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(first + second))
}

pub fn imul(current_frame: &Rc<StackEntry>) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(first * second))
}


pub fn irem(current_frame: &Rc<StackEntry>) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(value1 % value2));
}


pub fn ishl(current_frame: &Rc<StackEntry>) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(value1 << (value2 & 63)));
}


pub fn iushr(current_frame: &Rc<StackEntry>) -> () {
    let value2 = current_frame.pop().unwrap_int() as u32;
    let value1 = current_frame.pop().unwrap_int() as u32;
    let res = value1 >> (value2 & 63);
    current_frame.push(JavaValue::Int(res as i32));
}


pub fn isub(current_frame: &Rc<StackEntry>) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(value1 - value2));
}


pub fn ladd(current_frame: Rc<StackEntry>) -> () {
    let first = current_frame.pop().unwrap_long();
    let second = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(first + second));
}


pub fn lshl(current_frame: Rc<StackEntry>) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(value1 << ((value2 & 0x7F) as i64)));
}

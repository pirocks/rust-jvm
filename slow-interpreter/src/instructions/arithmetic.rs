use std::rc::Rc;
use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn fmul(current_frame: & StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_float();
    let value1 = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Float(value2 * value1));
}

pub fn fdiv(current_frame: & StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_float();
    let value1 = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Float(value1 / value2));
}

pub fn dmul(current_frame: & StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_double();
    let value1 = current_frame.pop().unwrap_double();
    current_frame.push(JavaValue::Double(value2 * value1));
}

pub fn dadd(current_frame: & StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_double();
    let value1 = current_frame.pop().unwrap_double();
    current_frame.push(JavaValue::Double(value2 + value1));
}


pub fn land(current_frame: &StackEntry) -> () {
    let first = current_frame.pop().unwrap_long();
    let second = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(first & second))
}

pub fn iand(current_frame: & StackEntry) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(first & second))
}


pub fn ixor(current_frame: & StackEntry) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(first ^ second))
}


pub fn ior(current_frame: & StackEntry) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(first | second))
}


pub fn iadd(current_frame: & StackEntry) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(((first as i64) + (second as i64)) as i32))
}

pub fn idiv(current_frame: & StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(((value1 as i64) / (value2 as i64)) as i32))
}

pub fn imul(current_frame: &StackEntry) -> () {
    let first = current_frame.pop().unwrap_int();
    let second = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int((first as i64 * second as i64) as i32))
}

pub fn ineg(current_frame: &StackEntry) -> () {
    let first = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int((0 - first as i64) as i32))
}


pub fn irem(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(value1 % value2));
}


pub fn ishl(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(value1 << (value2 & 63)));
}
pub fn ishr(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(value1 >> (value2 & 63)));
}

pub fn iushr(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_int() as u32;
    let value1 = current_frame.pop().unwrap_int() as u32;
    let res = value1 >> (value2 & 63);
    current_frame.push(JavaValue::Int(res as i32));
}


pub fn isub(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    current_frame.push(JavaValue::Int(value1 - value2));
}


pub fn lsub(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_long();
    let value1 = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(value1 - value2));
}

pub fn lcmp(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_long();
    let value1 = current_frame.pop().unwrap_long();
    if value1 == value2{
        current_frame.push(JavaValue::Int(0))
    }
    if value1 > value2{
        current_frame.push(JavaValue::Int(1))
    }
    if value1 < value2{
        current_frame.push(JavaValue::Int(-1))
    }
}


pub fn ladd(current_frame: &StackEntry) -> () {
    let first = current_frame.pop().unwrap_long();
    let second = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(first + second));
}


pub fn lor(current_frame: &StackEntry) -> () {
    let first = current_frame.pop().unwrap_long();
    let second = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(first | second));
}

pub fn lshl(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(value1 << ((value2 & 0x7F) as i64)));
}


pub fn lshr(current_frame: &StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Long(value1 >> ((value2 & 0x7F) as i64)));
}

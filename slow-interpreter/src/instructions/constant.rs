use std::mem::transmute;

use crate::java_values::JavaValue;
use crate::stack_entry::StackEntryMut;

pub fn fconst_0(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Float(0.0));
}

pub fn fconst_1(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Float(1.0));
}

pub fn fconst_2(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Float(2.0));
}


pub fn bipush(mut current_frame: StackEntryMut, b: u8) {
    current_frame.push(JavaValue::Int(unsafe { transmute::<u8, i8>(b) } as i32))//todo get rid of unneeded transmute
}

pub fn sipush(mut current_frame: StackEntryMut, val: u16) {
    current_frame.push(JavaValue::Int(unsafe { transmute::<u16, i16>(val) } as i32));
}


pub fn aconst_null(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::null())
}


pub fn iconst_5(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Int(5))
}

pub fn iconst_4(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Int(4))
}

pub fn iconst_3(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Int(3))
}

pub fn iconst_2(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Int(2))
}

pub fn iconst_1(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Int(1))
}

pub fn iconst_0(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Int(0))
}

pub fn dconst_1(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Double(1.0))
}

pub fn dconst_0(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Double(0.0))
}

pub fn iconst_m1(mut current_frame: StackEntryMut) {
    current_frame.push(JavaValue::Int(-1))
}

pub fn lconst(mut current_frame: StackEntryMut, i: i64) {
    current_frame.push(JavaValue::Long(i))
}

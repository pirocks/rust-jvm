use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn fconst_0(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Float(0.0));
}

pub fn fconst_1(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Float(1.0));
}


pub fn bipush(current_frame: &mut StackEntry, b: u8) {
    current_frame.push(JavaValue::Int(b as i32))
}

pub fn sipush(current_frame: &mut StackEntry, val: u16) {
    current_frame.push(JavaValue::Int(val as i32));
}


pub fn aconst_null(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Object(None))
}


pub fn iconst_5(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Int(5))
}

pub fn iconst_4(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Int(4))
}

pub fn iconst_3(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Int(3))
}

pub fn iconst_2(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Int(2))
}

pub fn iconst_1(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Int(1))
}

pub fn iconst_0(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Int(0))
}

pub fn dconst_1(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Double(1.0))
}

pub fn dconst_0(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Double(0.0))
}

pub fn iconst_m1(current_frame: &mut StackEntry) {
    current_frame.push(JavaValue::Int(-1))
}

pub fn lconst(current_frame: &mut StackEntry, i: i64) {
    current_frame.push(JavaValue::Long(i))
}

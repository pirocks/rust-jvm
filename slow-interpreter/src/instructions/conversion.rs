use rust_jvm_common::runtime_type::RuntimeType;

use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;

pub fn i2l(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Long(int as i64));
}

pub fn i2s(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Short(int as i16));
}

pub fn i2f(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Float(int as f32));
}

pub fn l2f(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let long = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Float(long as f32));
}

pub fn l2i(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let long = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Int(long as i32));
}

pub fn i2d(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Double(int as f64));
}

pub fn i2c(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(int as u16 as i32));
}

pub fn i2b(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(int as u8 as i32));
}

pub fn f2i(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    current_frame.push(JavaValue::Int(f as i32))
}

pub fn f2d(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    current_frame.push(JavaValue::Double(f as f64))
}

pub fn d2i(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    current_frame.push(JavaValue::Int(f as i32))
}

pub fn d2l(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    current_frame.push(JavaValue::Long(f as i64))
}

pub fn d2f(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    current_frame.push(JavaValue::Float(f as f32))
}

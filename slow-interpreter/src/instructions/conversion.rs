use classfile_view::view::ptype_view::PTypeView;

use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;

pub fn i2l(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Long(int as i64));
}

pub fn i2s(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Short(int as i16));
}

pub fn i2f(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Float(int as f32));
}


pub fn l2f(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let long = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Float(long as f32));
}

pub fn l2i(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let long = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Int(long as i32));
}


pub fn i2d(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Double(int as f64));
}


pub fn i2c(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(int as u8 as char as i32));
}


pub fn i2b(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let int = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(int as u8 as i32));
}


pub fn f2i(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    current_frame.push(jvm, JavaValue::Int(f as i32))
}

pub fn f2d(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    current_frame.push(jvm, JavaValue::Double(f as f64))
}

pub fn d2i(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    current_frame.push(jvm, JavaValue::Int(f as i32))
}


pub fn d2l(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    current_frame.push(jvm, JavaValue::Long(f as i64))
}

pub fn d2f(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let f = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    current_frame.push(jvm, JavaValue::Float(f as f32))
}

use std::mem::transmute;

use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;

pub fn fconst_0(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Float(0.0));
}

pub fn fconst_1(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Float(1.0));
}

pub fn fconst_2(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Float(2.0));
}


pub fn bipush(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, b: u8) {
    current_frame.push(jvm, JavaValue::Int(unsafe { transmute::<u8, i8>(b) } as i32))//todo get rid of unneeded transmute
}

pub fn sipush(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, val: u16) {
    current_frame.push(jvm, JavaValue::Int(unsafe { transmute::<u16, i16>(val) } as i32));
}


pub fn aconst_null(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::null())
}


pub fn iconst_5(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Int(5))
}

pub fn iconst_4(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Int(4))
}

pub fn iconst_3(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Int(3))
}

pub fn iconst_2(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Int(2))
}

pub fn iconst_1(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Int(1))
}

pub fn iconst_0(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Int(0))
}

pub fn dconst_1(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Double(1.0))
}

pub fn dconst_0(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Double(0.0))
}

pub fn iconst_m1(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    current_frame.push(jvm, JavaValue::Int(-1))
}

pub fn lconst(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, i: i64) {
    current_frame.push(jvm, JavaValue::Long(i))
}

use std::num::Wrapping;

use classfile_view::view::ptype_view::PTypeView;

use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;

pub fn fmul(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    current_frame.push(jvm, JavaValue::Float(value2 * value1));
}

pub fn fadd(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    current_frame.push(jvm, JavaValue::Float(value2 + value1));
}

pub fn fdiv(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    current_frame.push(jvm, JavaValue::Float(value1 / value2));
}

pub fn ddiv(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    let value1 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    current_frame.push(jvm, JavaValue::Double(value1 / value2));
}

pub fn dmul(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    let value1 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    current_frame.push(jvm, JavaValue::Double(value2 * value1));
}

pub fn dadd(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    let value1 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    current_frame.push(jvm, JavaValue::Double(value2 + value1));
}


pub fn dsub(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    let value1 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    current_frame.push(jvm, JavaValue::Double(value1 - value2));
}

pub fn fsub(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    current_frame.push(jvm, JavaValue::Float(value1 - value2));
}

pub fn lmul(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let mul_res = Wrapping(first) * Wrapping(second);
    current_frame.push(jvm, JavaValue::Long(mul_res.0))
}


pub fn lneg(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(-first))
}

pub fn land(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(first & second))
}

pub fn iand(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(first & second))
}


pub fn ixor(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(first ^ second))
}


pub fn ior(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(first | second))
}


pub fn iadd(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(((first as i64) + (second as i64)) as i32))
}

pub fn idiv(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(((value1 as i64) / (value2 as i64)) as i32))
}

pub fn imul(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int((first as i64 * second as i64) as i32))
}

pub fn ineg(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int((0 - first as i64) as i32))
}


pub fn irem(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(value1 % value2));
}


pub fn ishl(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(value1 << (value2 & 63)));
}

pub fn ishr(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(value1 >> (value2 & 63)));
}

pub fn iushr(int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let jvm = int_state.jvm;
    let mut current_frame = int_state.current_frame_mut();
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int() as u32;
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int() as u32;
    let res = value1 >> (value2 & 31);
    current_frame.push(int_state.jvm, JavaValue::Int(res as i32));
}


pub fn isub(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = Wrapping(current_frame.pop(jvm, PTypeView::IntType).unwrap_int());
    let value1 = Wrapping(current_frame.pop(jvm, PTypeView::IntType).unwrap_int());
    current_frame.push(jvm, JavaValue::Int((value1 - value2).0));
}


pub fn lsub(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let value1 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(value1 - value2));
}

pub fn lcmp(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let value1 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    if value1 == value2 {
        current_frame.push(jvm, JavaValue::Int(0))
    }
    if value1 > value2 {
        current_frame.push(jvm, JavaValue::Int(1))
    }
    if value1 < value2 {
        current_frame.push(jvm, JavaValue::Int(-1))
    }
}


pub fn ladd(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let wrapping_first = Wrapping(first);
    let wrapping_second = Wrapping(second);
    let sum = wrapping_first + wrapping_second;
    current_frame.push(jvm, JavaValue::Long(sum.0));
}

pub fn ldiv(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let value1 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(value1 / value2));
}

pub fn lrem(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let value1 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(value1 % value2));
}

pub fn lor(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(first | second));
}

pub fn lxor(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(first ^ second));
}

pub fn lshl(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(value1 << ((value2 & 0x3F) as i64)));
}


pub fn lshr(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(value1 >> ((value2 & 0x7F) as i64)));
}

pub fn lushr(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::LongType).unwrap_long() as u64;
    current_frame.push(jvm, JavaValue::Long((value1 << (value2 & 0x7F) as u64) as i64));
}

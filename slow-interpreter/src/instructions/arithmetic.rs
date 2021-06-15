use std::num::Wrapping;

use classfile_view::view::ptype_view::PTypeView;

use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::stack_entry::StackEntryMut;

pub fn fmul(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(PTypeView::FloatType).unwrap_float();
    current_frame.push(JavaValue::Float(value2 * value1));
}

pub fn fadd(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(PTypeView::FloatType).unwrap_float();
    current_frame.push(JavaValue::Float(value2 + value1));
}

pub fn fdiv(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(PTypeView::FloatType).unwrap_float();
    current_frame.push(JavaValue::Float(value1 / value2));
}

pub fn ddiv(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::DoubleType).unwrap_double();
    let value1 = current_frame.pop(PTypeView::DoubleType).unwrap_double();
    current_frame.push(JavaValue::Double(value1 / value2));
}

pub fn dmul(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::DoubleType).unwrap_double();
    let value1 = current_frame.pop(PTypeView::DoubleType).unwrap_double();
    current_frame.push(JavaValue::Double(value2 * value1));
}

pub fn dadd(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::DoubleType).unwrap_double();
    let value1 = current_frame.pop(PTypeView::DoubleType).unwrap_double();
    current_frame.push(JavaValue::Double(value2 + value1));
}


pub fn dsub(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::DoubleType).unwrap_double();
    let value1 = current_frame.pop(PTypeView::DoubleType).unwrap_double();
    current_frame.push(JavaValue::Double(value1 - value2));
}

pub fn fsub(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(PTypeView::FloatType).unwrap_float();
    current_frame.push(JavaValue::Float(value1 - value2));
}

pub fn lmul(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(PTypeView::LongType).unwrap_long();
    let mul_res = Wrapping(first) * Wrapping(second);
    current_frame.push(JavaValue::Long(mul_res.0))
}


pub fn lneg(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(-first))
}

pub fn land(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(first & second))
}

pub fn iand(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int(first & second))
}


pub fn ixor(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int(first ^ second))
}


pub fn ior(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int(first | second))
}


pub fn iadd(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int(((first as i64) + (second as i64)) as i32))
}

pub fn idiv(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int(((value1 as i64) / (value2 as i64)) as i32))
}

pub fn imul(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::IntType).unwrap_int();
    let second = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int((first as i64 * second as i64) as i32))
}

pub fn ineg(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int((0 - first as i64) as i32))
}


pub fn irem(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int(value1 % value2));
}


pub fn ishl(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int(value1 << (value2 & 63)));
}

pub fn ishr(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    current_frame.push(JavaValue::Int(value1 >> (value2 & 63)));
}

pub fn iushr<'l, 'k : 'l, 'gc_life>(int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) {
    let mut current_frame = int_state.current_frame_mut();
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int() as u32;
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int() as u32;
    let res = value1 >> (value2 & 31);
    current_frame.push(JavaValue::Int(res as i32));
}


pub fn isub(mut current_frame: StackEntryMut<'l, 'gc_life>) {
    let value2 = Wrapping(current_frame.pop(PTypeView::IntType).unwrap_int());
    let value1 = Wrapping(current_frame.pop(PTypeView::IntType).unwrap_int());
    current_frame.push(JavaValue::Int((value1 - value2).0));
}


pub fn lsub(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::LongType).unwrap_long();
    let value1 = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(value1 - value2));
}

pub fn lcmp(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::LongType).unwrap_long();
    let value1 = current_frame.pop(PTypeView::LongType).unwrap_long();
    if value1 == value2 {
        current_frame.push(JavaValue::Int(0))
    }
    if value1 > value2 {
        current_frame.push(JavaValue::Int(1))
    }
    if value1 < value2 {
        current_frame.push(JavaValue::Int(-1))
    }
}


pub fn ladd(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(PTypeView::LongType).unwrap_long();
    let wrapping_first = Wrapping(first);
    let wrapping_second = Wrapping(second);
    let sum = wrapping_first + wrapping_second;
    current_frame.push(JavaValue::Long(sum.0));
}

pub fn ldiv(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::LongType).unwrap_long();
    let value1 = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(value1 / value2));
}

pub fn lrem(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::LongType).unwrap_long();
    let value1 = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(value1 % value2));
}

pub fn lor(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(first | second));
}

pub fn lxor(mut current_frame: StackEntryMut) {
    let first = current_frame.pop(PTypeView::LongType).unwrap_long();
    let second = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(first ^ second));
}

pub fn lshl(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(value1 << ((value2 & 0x3F) as i64)));
}


pub fn lshr(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::LongType).unwrap_long();
    current_frame.push(JavaValue::Long(value1 >> ((value2 & 0x7F) as i64)));
}

pub fn lushr(mut current_frame: StackEntryMut) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::LongType).unwrap_long() as u64;
    current_frame.push(JavaValue::Long((value1 << (value2 & 0x7F) as u64) as i64));
}

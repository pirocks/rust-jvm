use std::num::Wrapping;

use classfile_view::view::ptype_view::PTypeView;

use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::runtime_type::RuntimeType;
use crate::stack_entry::StackEntryMut;

pub fn fmul(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    let value1 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    current_frame.push(JavaValue::Float(value2 * value1));
}

pub fn fadd(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    let value1 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    current_frame.push(JavaValue::Float(value2 + value1));
}

pub fn fdiv(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    let value1 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    current_frame.push(JavaValue::Float(value1 / value2));
}

pub fn ddiv(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    let value1 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    current_frame.push(JavaValue::Double(value1 / value2));
}

pub fn dmul(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    let value1 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    current_frame.push(JavaValue::Double(value2 * value1));
}

pub fn dadd(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    let value1 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    current_frame.push(JavaValue::Double(value2 + value1));
}


pub fn dsub(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    let value1 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    current_frame.push(JavaValue::Double(value1 - value2));
}

pub fn fsub(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    let value1 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    current_frame.push(JavaValue::Float(value1 - value2));
}

pub fn lmul(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let second = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let mul_res = Wrapping(first) * Wrapping(second);
    current_frame.push(JavaValue::Long(mul_res.0))
}


pub fn lneg(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(-first))
}

pub fn land(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let second = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(first & second))
}

pub fn iand(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let second = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(first & second))
}


pub fn ixor(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let second = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(first ^ second))
}


pub fn ior(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let second = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(first | second))
}


pub fn iadd(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let second = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(((first as i64) + (second as i64)) as i32))
}

pub fn idiv(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(((value1 as i64) / (value2 as i64)) as i32))
}

pub fn imul(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let second = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int((first as i64 * second as i64) as i32))
}

pub fn ineg(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int((0 - first as i64) as i32))
}


pub fn irem(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(value1 % value2));
}


pub fn ishl(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(value1 << (value2 & 63)));
}

pub fn ishr(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    current_frame.push(JavaValue::Int(value1 >> (value2 & 63)));
}

pub fn iushr(int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let mut current_frame = int_state.current_frame_mut();
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int() as u32;
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int() as u32;
    let res = value1 >> (value2 & 31);
    current_frame.push(JavaValue::Int(res as i32));
}


pub fn isub(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = Wrapping(current_frame.pop(Some(RuntimeType::IntType)).unwrap_int());
    let value1 = Wrapping(current_frame.pop(Some(RuntimeType::IntType)).unwrap_int());
    current_frame.push(JavaValue::Int((value1 - value2).0));
}


pub fn lsub(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let value1 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(value1 - value2));
}

pub fn lcmp(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let value1 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
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


pub fn ladd(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let second = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let wrapping_first = Wrapping(first);
    let wrapping_second = Wrapping(second);
    let sum = wrapping_first + wrapping_second;
    current_frame.push(JavaValue::Long(sum.0));
}

pub fn ldiv(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let value1 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(value1 / value2));
}

pub fn lrem(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let value1 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(value1 % value2));
}

pub fn lor(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let second = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(first | second));
}

pub fn lxor(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let second = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(first ^ second));
}

pub fn lshl(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(value1 << ((value2 & 0x3F) as i64)));
}


pub fn lshr(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Long(value1 >> ((value2 & 0x7F) as i64)));
}

pub fn lushr(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long() as u64;
    current_frame.push(JavaValue::Long((value1 << (value2 & 0x7F) as u64) as i64));
}

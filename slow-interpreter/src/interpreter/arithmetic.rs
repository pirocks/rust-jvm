use std::num::Wrapping;
use std::ops::Rem;

use num_traits::Zero;

use rust_jvm_common::runtime_type::RuntimeType;

use crate::better_java_stack::frames::HasFrame;
use crate::exceptions::WasException;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};
use crate::jvm_state::JVMState;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::arithmetic_exception::ArithmeticException;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub fn fmul<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    current_frame.push(InterpreterJavaValue::Float(value2 * value1));
    PostInstructionAction::Next {}
}

pub fn fadd<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    current_frame.push(InterpreterJavaValue::Float(value2 + value1));
    PostInstructionAction::Next {}
}

pub fn fdiv<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    current_frame.push(InterpreterJavaValue::Float(value1 / value2));
    PostInstructionAction::Next {}
}

pub fn ddiv<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    let value1 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    current_frame.push(InterpreterJavaValue::Double(value1 / value2));
    PostInstructionAction::Next {}
}

pub fn dmul<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    let value1 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    current_frame.push(InterpreterJavaValue::Double(value2 * value1));
    PostInstructionAction::Next {}
}

pub fn dadd<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    let value1 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    current_frame.push(InterpreterJavaValue::Double(value2 + value1));
    PostInstructionAction::Next {}
}

pub fn dsub<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    let value1 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    current_frame.push(InterpreterJavaValue::Double(value1 - value2));
    PostInstructionAction::Next {}
}

pub fn fsub<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    current_frame.push(InterpreterJavaValue::Float(value1 - value2));
    PostInstructionAction::Next {}
}

pub fn lmul<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let second = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let mul_res = Wrapping(first) * Wrapping(second);
    current_frame.push(InterpreterJavaValue::Long(mul_res.0));
    PostInstructionAction::Next {}
}

pub fn lneg<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Long(-first));
    PostInstructionAction::Next {}
}

pub fn land<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let second = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Long(((first as u64) & (second as u64)) as i64));
    PostInstructionAction::Next {}
}

pub fn iand<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let second = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int(((first as u32) & (second as u32)) as i32));
    PostInstructionAction::Next {}
}

pub fn ixor<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let second = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int((first as u32 ^ second as u32) as i32));
    PostInstructionAction::Next {}
}

pub fn ior<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let second = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int(((first as u32) | (second as u32)) as i32));
    PostInstructionAction::Next {}
}

pub fn iadd<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let second = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int(((first as i64) + (second as i64)) as i32));
    PostInstructionAction::Next {}
}

pub fn idiv<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    if value2 == 0 {
        let jvm = current_frame.inner().inner().jvm();
        let throwable = ArithmeticException::new(jvm, current_frame.inner().inner()).expect("Exception creating exception").object().cast_throwable();
        return PostInstructionAction::Exception { exception: WasException { exception_obj: throwable } };
    }
    current_frame.push(InterpreterJavaValue::Int(((value1 as i64) / (value2 as i64)) as i32));
    PostInstructionAction::Next {}
}

pub fn imul<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let second = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int((first as i64 * second as i64) as i32));
    PostInstructionAction::Next {}
}

pub fn ineg<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int((0 - first as i64) as i32));
    PostInstructionAction::Next {}
}

pub fn irem<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    if value2 == 0 {
        let jvm = current_frame.inner().inner().jvm();
        let throwable = ArithmeticException::new(jvm, current_frame.inner().inner()).expect("Exception creating exception").object().cast_throwable();
        return PostInstructionAction::Exception { exception: WasException { exception_obj: throwable } };
    }
    current_frame.push(InterpreterJavaValue::Int(value1 % value2));
    PostInstructionAction::Next {}
}

pub fn ishl<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int(value1 << (value2 & 63)));
    PostInstructionAction::Next {}
}

pub fn ishr<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int(value1 >> (value2 & 63)));
    PostInstructionAction::Next {}
}

pub fn iushr<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int() as u32;
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int() as u32;
    let res = value1 >> (value2 & 31);
    current_frame.push(InterpreterJavaValue::Int(res as i32));
    PostInstructionAction::Next {}
}

pub fn isub<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = Wrapping(current_frame.pop(RuntimeType::IntType).unwrap_int());
    let value1 = Wrapping(current_frame.pop(RuntimeType::IntType).unwrap_int());
    current_frame.push(InterpreterJavaValue::Int((value1 - value2).0));
    PostInstructionAction::Next {}
}

pub fn lsub<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let value1 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Long((Wrapping(value1) - Wrapping(value2)).0));
    PostInstructionAction::Next {}
}

pub fn lcmp<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let value1 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    if value1 == value2 {
        current_frame.push(InterpreterJavaValue::Int(0))
    }
    if value1 > value2 {
        current_frame.push(InterpreterJavaValue::Int(1))
    }
    if value1 < value2 {
        current_frame.push(InterpreterJavaValue::Int(-1))
    }
    PostInstructionAction::Next {}
}

pub fn ladd<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let second = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let wrapping_first = Wrapping(first);
    let wrapping_second = Wrapping(second);
    let sum = wrapping_first + wrapping_second;
    current_frame.push(InterpreterJavaValue::Long(sum.0));
    PostInstructionAction::Next {}
}

pub fn ldiv<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let value1 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    if value2 == 0 {
        let throwable = ArithmeticException::new(jvm, current_frame.inner().inner()).expect("Exception creating exception").object().cast_throwable();
        return PostInstructionAction::Exception { exception: WasException { exception_obj: throwable } };
    }
    current_frame.push(InterpreterJavaValue::Long(value1.wrapping_div(value2)));
    PostInstructionAction::Next {}
}

pub fn lrem<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let value1 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    if value2 == 0 {
        let jvm = current_frame.inner().inner().jvm();
        let throwable = ArithmeticException::new(jvm, current_frame.inner().inner()).expect("Exception creating exception").object().cast_throwable();
        return PostInstructionAction::Exception { exception: WasException { exception_obj: throwable } };
    }
    current_frame.push(InterpreterJavaValue::Long(value1 % value2));
    PostInstructionAction::Next {}
}

pub fn lor<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let second = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Long(first | second));
    PostInstructionAction::Next {}
}

pub fn lxor<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let first = current_frame.pop(RuntimeType::LongType).unwrap_long();
    let second = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Long(first ^ second));
    PostInstructionAction::Next {}
}

pub fn lshl<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Long(value1 << ((value2 & 0x3F) as i64)));
    PostInstructionAction::Next {}
}

pub fn lshr<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Long(value1 >> ((value2 & 0x7F) as i64)));
    PostInstructionAction::Next {}
}

pub fn lushr<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::LongType).unwrap_long() as u64;
    current_frame.push(InterpreterJavaValue::Long((value1 >> (value2 & 0x7F) as u64) as i64));
    PostInstructionAction::Next {}
}


pub fn frem<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let res = drem_impl(value2 as f64, value1 as f64) as f32;
    current_frame.push(InterpreterJavaValue::Float(res));
    PostInstructionAction::Next {}
}

pub fn fneg<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    current_frame.push(InterpreterJavaValue::Float(-val));
    PostInstructionAction::Next {}
}

pub fn drem<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_double(); //divisor
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_double();
    let res = drem_impl(value2, value1);
    current_frame.push(InterpreterJavaValue::Double(res));
    PostInstructionAction::Next {}
}

fn drem_impl(value2: f64, value1: f64) -> f64 {
    let res = if value1.is_nan() || value2.is_nan() {
        f64::NAN
    } else if value2.is_zero() || value1.is_infinite() {
        if value1.is_sign_negative() {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    } else if value2.is_infinite() {
        value1
    } else if value1.is_zero() {
        value1
    } else {
        if (value1 / value2).is_sign_negative() {
            -value1.rem(value2).abs()
        } else {
            value1.rem(value2).abs()
        }
    };
    res
}

pub fn dneg<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    current_frame.push(InterpreterJavaValue::Double(-val));
    PostInstructionAction::Next {}
}

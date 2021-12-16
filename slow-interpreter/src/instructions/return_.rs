use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JVMState};
use crate::java_values::JavaValue;

pub fn freturn<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life>) {
    let res: JavaValue<'gc_life> = interpreter_state.current_frame_mut().pop(Some(RuntimeType::FloatType));
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Float(_) => {}
        _ => panic!(),
    }

    interpreter_state.previous_frame_mut().push(res);
}

pub fn dreturn(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life>) {
    let res = interpreter_state.current_frame_mut().pop(Some(RuntimeType::DoubleType));
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Double(_) => {}
        _ => panic!(),
    }

    interpreter_state.previous_frame_mut().push(res);
}

pub fn areturn(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life>) {
    assert_ne!(interpreter_state.current_frame().operand_stack(jvm).len(), 0);
    let res = interpreter_state.pop_current_operand_stack(Some(CClassName::object().into()));
    interpreter_state.set_function_return(true);

    interpreter_state.previous_frame_mut().push(res);
}

pub fn return_(interpreter_state: &'_ mut InterpreterStateGuard<'gc_life>) {
    interpreter_state.set_function_return(true);
}

pub fn ireturn(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life>) {
    let res = interpreter_state.current_frame_mut().pop(Some(RuntimeType::IntType));
    interpreter_state.set_function_return(true);
    res.unwrap_int();

    interpreter_state.previous_frame_mut().push(res);
}

pub fn lreturn(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life>) {
    let res = interpreter_state.current_frame_mut().pop(Some(RuntimeType::LongType));
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Long(_) => {}
        _ => {
            panic!()
        }
    }

    interpreter_state.previous_frame_mut().push(res);
}
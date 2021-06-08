use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::java_values::JavaValue;

pub fn freturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop(PTypeView::FloatType);
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(res);
}

pub fn dreturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop(PTypeView::DoubleType);
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(res);
}


pub fn areturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    assert_ne!(interpreter_state.current_frame().operand_stack().len(), 0);
    let res = interpreter_state.pop_current_operand_stack(ClassName::object().into());
    interpreter_state.set_function_return(true);

    interpreter_state.previous_frame_mut().push(res);
}


pub fn return_(interpreter_state: &mut InterpreterStateGuard) {
    interpreter_state.set_function_return(true);
}


pub fn ireturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop(PTypeView::IntType);
    interpreter_state.set_function_return(true);
    res.unwrap_int();

    interpreter_state.previous_frame_mut().push(res);
}


pub fn lreturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop(PTypeView::LongType);
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Long(_) => {}
        _ => {
            panic!()
        }
    }

    interpreter_state.previous_frame_mut().push(res);
}


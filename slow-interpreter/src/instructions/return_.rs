use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::java_values::JavaValue;

pub fn freturn<'l, 'k : 'l, 'gc_life>(_jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) {
    let res = interpreter_state.current_frame_mut().pop(PTypeView::FloatType);
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(res);
}

pub fn dreturn<'l, 'k : 'l, 'gc_life>(_jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) {
    let res = interpreter_state.current_frame_mut().pop(PTypeView::DoubleType);
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(res);
}


pub fn areturn<'l, 'k : 'l, 'gc_life>(_jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) {
    assert_ne!(interpreter_state.current_frame().operand_stack().len(), 0);
    let res = interpreter_state.pop_current_operand_stack(ClassName::object().into());
    interpreter_state.set_function_return(true);

    interpreter_state.previous_frame_mut().push(res);
}


pub fn return_<'l, 'k : 'l, 'gc_life>(interpreter_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) {
    interpreter_state.set_function_return(true);
}


pub fn ireturn<'l, 'k : 'l, 'gc_life>(_jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) {
    let res = interpreter_state.current_frame_mut().pop(PTypeView::IntType);
    interpreter_state.set_function_return(true);
    res.unwrap_int();

    interpreter_state.previous_frame_mut().push(res);
}


pub fn lreturn<'l, 'k : 'l, 'gc_life>(_jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) {
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


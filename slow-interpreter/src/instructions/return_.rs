use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::java_values::JavaValue;
use crate::stack_entry::StackEntryMut;

pub fn freturn(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let mut stack_entry_mut: StackEntryMut<'gc_life> = interpreter_state.current_frame_mut();
    let res: JavaValue<'gc_life> = stack_entry_mut.pop(jvm, PTypeView::FloatType);
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(jvm, res);
}

pub fn dreturn(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let res = interpreter_state.current_frame_mut().pop(jvm, PTypeView::DoubleType);
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(jvm, res);
}


pub fn areturn(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    assert_ne!(interpreter_state.current_frame().operand_stack(jvm).len(), 0);
    let res = interpreter_state.pop_current_operand_stack(ClassName::object().into());
    interpreter_state.set_function_return(true);

    interpreter_state.previous_frame_mut().push(jvm, res);
}


pub fn return_(interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    interpreter_state.set_function_return(true);
}


pub fn ireturn(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let res = interpreter_state.current_frame_mut().pop(jvm, PTypeView::IntType);
    interpreter_state.set_function_return(true);
    res.unwrap_int();

    interpreter_state.previous_frame_mut().push(jvm, res);
}


pub fn lreturn(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let res = interpreter_state.current_frame_mut().pop(jvm, PTypeView::LongType);
    interpreter_state.set_function_return(true);
    match res {
        JavaValue::Long(_) => {}
        _ => {
            panic!()
        }
    }

    interpreter_state.previous_frame_mut().push(jvm, res);
}


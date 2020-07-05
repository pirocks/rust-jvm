use crate::{JVMState, StackEntry, InterpreterStateGuard};
use crate::java_values::JavaValue;
use std::sync::RwLockWriteGuard;

fn previous_frame<'l>(frames: &'l mut RwLockWriteGuard<Vec<StackEntry>>) -> &'l mut StackEntry {
    let len = frames.len();
    &mut frames[len - 2]
}

pub fn freturn<'l>(_jvm: &'static JVMState, interpreter_state: & mut InterpreterStateGuard) -> () {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(res);
}

pub fn dreturn<'l>(_jvm: &'static JVMState, interpreter_state: & mut InterpreterStateGuard) -> () {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(res);
}


pub fn areturn<'l>(_jvm: &'static JVMState, interpreter_state: & mut InterpreterStateGuard) -> () {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;

    interpreter_state.previous_frame_mut().push(res);
}


pub fn return_<'l>(interpreter_state: & mut InterpreterStateGuard) {
    *interpreter_state.function_return_mut() = true;
}


pub fn ireturn<'l>(_jvm: &'static JVMState, interpreter_state: & mut InterpreterStateGuard) -> () {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;
    res.unwrap_int();

    interpreter_state.previous_frame_mut().push(res);
}


pub fn lreturn<'l>(_jvm: &'static JVMState, interpreter_state: & mut InterpreterStateGuard) -> () {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;
    match res {
        JavaValue::Long(_) => {}
        _ => {
            // interpreter_state.get_current_frame().print_stack_trace();
            dbg!(res);
            panic!()
        }
    }

    interpreter_state.previous_frame_mut().push(res);
}


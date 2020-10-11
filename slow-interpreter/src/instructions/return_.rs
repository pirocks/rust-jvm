use crate::{InterpreterStateGuard, JVMState};
use crate::java_values::JavaValue;

pub fn freturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(res);
}

pub fn dreturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }

    interpreter_state.previous_frame_mut().push(res);
}


pub fn areturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;

    interpreter_state.previous_frame_mut().push(res);
}


pub fn return_(interpreter_state: &mut InterpreterStateGuard) {
    *interpreter_state.function_return_mut() = true;
}


pub fn ireturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;
    res.unwrap_int();

    interpreter_state.previous_frame_mut().push(res);
}


pub fn lreturn(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let res = interpreter_state.current_frame_mut().pop();
    *interpreter_state.function_return_mut() = true;
    match res {
        JavaValue::Long(_) => {}
        _ => {
            // interpreter_state.get_current_frame().print_stack_trace();
            // dbg!(res);
            panic!()
        }
    }

    interpreter_state.previous_frame_mut().push(res);
}


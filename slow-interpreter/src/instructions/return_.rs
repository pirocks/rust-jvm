use crate::{InterpreterState, JVMState, StackEntry};
use crate::java_values::JavaValue;
use crate::threading::JavaThread;

pub fn freturn(jvm: &JVMState, current_thread: &JavaThread, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    current_thread.interpreter_state.function_return.replace(true);
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }
    current_thread.get_previous_frame().push(res);
}

pub fn dreturn(jvm: &JVMState, current_thread: &JavaThread, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    current_thread.interpreter_state.function_return.replace(true);
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }
    current_thread.get_previous_frame().push(res);
}


pub fn areturn(jvm: &JVMState, current_thread: &JavaThread, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    current_thread.interpreter_state.function_return.replace(true);
    current_thread.get_previous_frame().push(res);
}


pub fn return_(interpreter_state: &InterpreterState) {
    *interpreter_state.function_return = true;
}


pub fn ireturn(state: &JVMState, current_thread: &JavaThread, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    *current_thread.interpreter_state.function_return.write().unwrap() = true;
    res.unwrap_int();
    current_thread.get_previous_frame().push(res);
}


pub fn lreturn(jvm: &JVMState, current_thread: &JavaThread, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    *current_thread.interpreter_state.function_return.write().unwrap() = true;
    match res {
        JavaValue::Long(_) => {}
        _ => {
            // current_frame.print_stack_trace();
            dbg!(res);
            panic!()
        }
    }
    current_thread.get_previous_frame().push(res);
}


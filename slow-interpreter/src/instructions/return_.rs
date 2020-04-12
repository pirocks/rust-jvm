use std::rc::Rc;
use crate::java_values::JavaValue;
use crate::{JVMState, StackEntry};

pub fn freturn(state: & JVMState, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    state.get_current_thread().interpreter_state.function_return.replace(true);
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }
    current_frame.last_call_stack.as_ref().unwrap().push(res);
}

pub fn dreturn(state: & JVMState, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    state.get_current_thread().interpreter_state.function_return.replace(true);
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }
    current_frame.last_call_stack.as_ref().unwrap().push(res);
}


pub fn areturn(state: & JVMState, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    state.get_current_thread().interpreter_state.function_return.replace(true);
    current_frame.last_call_stack.as_ref().unwrap().push(res);
}


pub fn return_(state: & JVMState) {
    state.get_current_thread().interpreter_state.function_return.replace(true);
}


pub fn ireturn(state: & JVMState, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    state.get_current_thread().interpreter_state.function_return.replace(true);
    match res {
        JavaValue::Int(_) => {}
        JavaValue::Short(_) => {}
        JavaValue::Byte(_) => {}
        JavaValue::Boolean(_) => {}
        JavaValue::Char(_) => {}
        _ => panic!()
    }
    current_frame.last_call_stack.as_ref().unwrap().push(res);
}


pub fn lreturn(state: & JVMState, current_frame: &StackEntry) -> () {
    let res = current_frame.pop();
    state.get_current_thread().interpreter_state.function_return.replace(true);
    match res {
        JavaValue::Long(_) => {}
        _ => {
            current_frame.print_stack_trace();
            dbg!(res);
            panic!()
        }
    }
    current_frame.last_call_stack.as_ref().unwrap().push(res);
}


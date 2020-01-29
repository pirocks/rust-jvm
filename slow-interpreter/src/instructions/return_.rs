use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn freturn(state: &mut InterpreterState, current_frame: &Rc<StackEntry>) -> () {
    let res = current_frame.pop();
    state.function_return = true;
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }
    current_frame.last_call_stack.as_ref().unwrap().push(res);
}

pub fn dreturn(state: &mut InterpreterState, current_frame: &Rc<StackEntry>) -> () {
    let res = current_frame.pop();
    state.function_return = true;
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }
    current_frame.last_call_stack.as_ref().unwrap().push(res);
}


pub fn areturn(state: &mut InterpreterState, current_frame: &Rc<StackEntry>) -> () {
    let res = current_frame.pop();
    state.function_return = true;
    current_frame.last_call_stack.as_ref().unwrap().push(res);
}


pub fn return_(state: &mut InterpreterState) {
    state.function_return = true;
}


pub fn ireturn(state: &mut InterpreterState, current_frame: &Rc<StackEntry>) -> () {
    let res = current_frame.pop();
    state.function_return = true;
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

use crate::{InterpreterState, JVMState, StackEntry};
use crate::java_values::JavaValue;
use crate::threading::JavaThread;
use std::sync::RwLockWriteGuard;

fn previous_frame<'l>(frames: &'l mut RwLockWriteGuard<Vec<StackEntry>>) -> &'l mut StackEntry {
    let len = frames.len();
    &mut frames[len - 2]
}

pub fn freturn(_jvm: &'static JVMState, current_thread: &JavaThread, current_frame: &mut StackEntry) -> () {
    let res = current_frame.pop();
    *current_thread.interpreter_state.function_return.write().unwrap() = true;
    match res {
        JavaValue::Float(_) => {}
        _ => panic!()
    }
    let mut frames = current_thread.get_frames_mut();
    previous_frame(&mut frames).push(res);
}

pub fn dreturn(_jvm: &'static JVMState, current_thread: &JavaThread, current_frame: &mut StackEntry) -> () {
    let res = current_frame.pop();
    *current_thread.interpreter_state.function_return.write().unwrap() = true;
    match res {
        JavaValue::Double(_) => {}
        _ => panic!()
    }
    let mut frames = current_thread.get_frames_mut();
    previous_frame(&mut frames).push(res);
}


pub fn areturn(_jvm: &'static JVMState, current_thread: &JavaThread, current_frame: &mut StackEntry) -> () {
    let res = current_frame.pop();
    *current_thread.interpreter_state.function_return.write().unwrap() = true;
    let mut frames = current_thread.get_frames_mut();
    previous_frame(&mut frames).push(res);
}


pub fn return_(interpreter_state: &InterpreterState) {
    *interpreter_state.function_return.write().unwrap() = true;
}


pub fn ireturn(_jvm: &'static JVMState, current_thread: &JavaThread, current_frame: &mut StackEntry) -> () {
    let res = current_frame.pop();
    *current_thread.interpreter_state.function_return.write().unwrap() = true;
    res.unwrap_int();
    let mut frames = current_thread.get_frames_mut();
    previous_frame(&mut frames).push(res);
}


pub fn lreturn(_jvm: &'static JVMState, current_thread: &JavaThread, current_frame: &mut StackEntry) -> () {
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
    let mut frames = current_thread.get_frames_mut();
    previous_frame(&mut frames).push(res);
}


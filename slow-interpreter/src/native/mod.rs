use crate::InterpreterState;
use crate::CallStackEntry;
use std::rc::Rc;
use std::sync::Arc;
use runtime_common::runtime_class::RuntimeClass;


pub fn run_native_method(
    state:&InterpreterState,
    frame: Rc<CallStackEntry>,
    classfile: Arc<RuntimeClass>,
    method_i : usize
) {

//    match method_name{
//        _ => unimplemented!("{}",method_name)
//    }
}
use crate::InterpreterState;
use crate::CallStackEntry;
use std::rc::Rc;
use crate::runtime_class::RuntimeClass;
use std::sync::Arc;




pub fn run_native_method(state:&InterpreterState,frame: Rc<CallStackEntry>,classfile: Arc<RuntimeClass>, method_i : usize) {

//    match method_name{
//        _ => unimplemented!("{}",method_name)
//    }
}
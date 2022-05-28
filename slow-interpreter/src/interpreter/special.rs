use rust_jvm_common::runtime_type::RuntimeType;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};
use crate::JVMState;

pub fn arraylength<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc>{
    let array_o = match current_frame.pop(RuntimeType::object()).unwrap_object() {
        Some(x) => x,
        None => {
            todo!()
            /*return throw_npe(jvm, int_state);*/
        }
    };
    //todo use ArrayMemoryLayout
    let len = unsafe { (array_o.as_ptr() as *const i32).read() };
    current_frame.push(InterpreterJavaValue::Int(len));
    PostInstructionAction::Next {}
}
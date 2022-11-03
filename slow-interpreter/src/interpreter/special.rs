use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::runtime_type::RuntimeType;
use crate::better_java_stack::frames::HasFrame;

use crate::interpreter::common::special::invoke_instanceof;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue, RealInterpreterStateGuard};
use crate::JVMState;

pub fn arraylength<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
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

pub fn checkcast<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, cpdtype: CPDType) -> PostInstructionAction<'gc> {
    let obj = int_state.current_frame_mut().pop(RuntimeType::object());
    if obj.unwrap_object().is_none() {
        int_state.current_frame_mut().push(obj);
        return PostInstructionAction::Next {};
    }
    int_state.current_frame_mut().push(obj);
    invoke_instanceof(jvm, int_state.current_frame_mut(), cpdtype);
    let res = int_state.current_frame_mut().pop(RuntimeType::IntType).unwrap_int();
    int_state.current_frame_mut().push(obj);
    if res == 0 {
        dbg!(obj.to_new_java_handle(jvm).unwrap_object().unwrap().runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool));
        dbg!(cpdtype.jvm_representation(&jvm.string_pool));
        int_state.inner().debug_print_stack_trace(jvm);
        todo!()
    }
    PostInstructionAction::Next {}
}
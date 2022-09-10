use rust_jvm_common::compressed_classfile::names::{CClassName};
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::{JVMState, WasException};
use crate::new_java_values::owned_casts::OwnedCastAble;


pub fn athrow<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let exception_obj = {
        let value = int_state.current_frame_mut().pop(CClassName::throwable().into());
        // let value = interpreter_state.int_state.as_mut().unwrap().call_stack.last_mut().unwrap().operand_stack.pop().unwrap();
        value.to_new_java_handle(jvm)
    };
    let allocated_handle = exception_obj.unwrap_object_nonnull();

    //todo checkcast not array
    PostInstructionAction::Exception { exception: WasException{ exception_obj: allocated_handle.cast_throwable() } }
}

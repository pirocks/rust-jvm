use rust_jvm_common::compressed_classfile::names::{CClassName};
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::{JVMState};
use another_jit_vm_ir::WasException;

pub fn athrow<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let exception_obj = {
        let value = int_state.current_frame_mut().pop(CClassName::throwable().into());
        // let value = interpreter_state.int_state.as_mut().unwrap().call_stack.last_mut().unwrap().operand_stack.pop().unwrap();
        value.to_new_java_handle(jvm)
    };
    // if jvm.debug_print_exceptions {
    println!("EXCEPTION:");
    int_state.inner().debug_print_stack_trace(jvm);
    let allocated_handle = exception_obj.unwrap_object_nonnull();
    dbg!(allocated_handle.duplicate_discouraged().cast_throwable().print_stack_trace(jvm, int_state.inner())).unwrap();
    // dbg!(exception_obj.as_njv().unwrap_normal_object().unwrap().get_var_top_level(jvm, FieldName::field_detailMessage()));
    // }

    //todo checkcast not array
    int_state.inner().set_throw(allocated_handle.into());
    PostInstructionAction::Exception { exception: WasException{} }
}

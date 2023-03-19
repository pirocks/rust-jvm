use crate::ir_compiler_common::special::IRCompilerState;
use crate::ir_compiler_common::stack_stores::StackPos;

pub(crate) fn emit_aload_n(compiler_state: &mut IRCompilerState, local_var_index: u16) {
    let reference_token = compiler_state.emit_local_load_pointer(local_var_index);
    compiler_state.emit_stack_store_pointer(StackPos::AfterFromEnd(0), reference_token)
}

pub(crate) fn emit_iload_n(compiler_state: &mut IRCompilerState, local_var_index: u16) {
    let integer_token = compiler_state.emit_local_load_integer(local_var_index);
    compiler_state.emit_stack_store_int(StackPos::AfterFromEnd(0), integer_token)
}

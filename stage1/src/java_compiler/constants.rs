use crate::ir_compiler_common::special::IRCompilerState;
use crate::ir_compiler_common::stack_stores::StackPos;

pub(crate) fn emit_iconst(compiler_state: &mut IRCompilerState, integer_const_value: i32) {
    let const_token = compiler_state.emit_constant_int(integer_const_value);
    compiler_state.emit_stack_store_int(StackPos::AfterFromEnd(0), const_token)
}


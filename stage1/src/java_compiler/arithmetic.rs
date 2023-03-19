use crate::ir_compiler_common::special::IRCompilerState;
use crate::ir_compiler_common::stack_stores::StackPos;

pub(crate) fn emit_iadd(compiler_state: &mut IRCompilerState) {
    let a = compiler_state.emit_stack_load_int(StackPos::BeforeFromEnd(0));
    let b = compiler_state.emit_stack_load_int(StackPos::BeforeFromEnd(1));
    let res = compiler_state.emit_add_integer(a, b);
    compiler_state.emit_stack_store_int(StackPos::AfterFromEnd(0), res);
}

pub(crate) fn emit_imul(compiler_state: &mut IRCompilerState) {
    let a = compiler_state.emit_stack_load_int(StackPos::BeforeFromEnd(0));
    let b = compiler_state.emit_stack_load_int(StackPos::BeforeFromEnd(1));
    let res = compiler_state.emit_mul_integer(a, b);
    compiler_state.emit_stack_store_int(StackPos::AfterFromEnd(0), res);
}


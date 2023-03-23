use crate::ir_compiler_common::special::IRCompilerState;

pub(crate) fn emit_ireturn(compiler_state: &mut IRCompilerState) {
    let res = compiler_state.current_operand_stack_tokens.pop().unwrap();
    compiler_state.emit_ir_end(res)
}

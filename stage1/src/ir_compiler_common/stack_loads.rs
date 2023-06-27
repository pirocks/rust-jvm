use crate::ir_compiler_common::{IntegerValueToken, IRCompilerState, PointerValueToken};
use crate::ir_compiler_common::stack_stores::StackPos;

impl IRCompilerState<'_> {
    pub fn emit_stack_load_pointer(&mut self, _from_end: StackPos) -> PointerValueToken {
        todo!()
    }

    pub fn emit_stack_load_int(&mut self, _from_end: StackPos) -> IntegerValueToken {
        todo!()
    }
}

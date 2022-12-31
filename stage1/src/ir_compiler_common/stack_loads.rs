use crate::ir_compiler_common::{IntegerValueToken, IRCompilerState, PointerValueToken};

impl IRCompilerState<'_> {
    pub fn emit_stack_load_pointer(&mut self, from_end: u16) -> PointerValueToken {
        todo!()
    }

    pub fn emit_stack_load_int(&mut self, from_end: u16) -> IntegerValueToken {
        todo!()
    }
}

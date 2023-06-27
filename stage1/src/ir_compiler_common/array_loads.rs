
use crate::ir_compiler_common::{IntegerValueToken, PointerValueToken};
use crate::ir_compiler_common::special::IRCompilerState;

impl IRCompilerState<'_> {
    pub fn emit_local_load_pointer(&mut self, local_var: u16) -> PointerValueToken {
        self.current_local_var_tokens[local_var as usize].unwrap_pointer()
    }

    pub fn emit_local_load_integer(&mut self, local_var: u16) -> IntegerValueToken{
        self.current_local_var_tokens[local_var as usize].unwrap_integer()
    }
}


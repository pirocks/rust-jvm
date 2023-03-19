use crate::CompilerState;
use crate::ir_compiler_common::{IntegerValue, IntegerValueToken};
use crate::ir_compiler_common::special::IRCompilerState;

impl IRCompilerState<'_>{
    pub fn emit_add_integer(&mut self, a: IntegerValueToken, b: IntegerValueToken) -> IntegerValueToken{
        todo!()
    }

    pub fn emit_mul_integer(&mut self, a: IntegerValueToken, b: IntegerValueToken) -> IntegerValueToken{
        todo!()
    }
}

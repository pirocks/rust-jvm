use crate::ir_compiler_common::IntegerValueToken;
use crate::ir_compiler_common::special::IRCompilerState;

impl IRCompilerState<'_>{
    pub fn emit_add_integer(&mut self, _a: IntegerValueToken, _b: IntegerValueToken) -> IntegerValueToken{
        todo!()
    }

    pub fn emit_mul_integer(&mut self, _a: IntegerValueToken, _b: IntegerValueToken) -> IntegerValueToken{
        todo!()
    }
}

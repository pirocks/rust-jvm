use crate::ir_compiler_common::IntegerValueToken;
use crate::ir_compiler_common::special::IRCompilerState;

impl IRCompilerState<'_>{
    pub fn emit_constant_int(&mut self, _constant: i32) -> IntegerValueToken{
        todo!()
    }
}

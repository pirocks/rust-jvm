use another_jit_vm_ir::compiler::Size;
use crate::ir_compiler_common::{DoubleValueToken, FloatValueToken, IntegerValueToken, IRCompilerState, LongValueToken, PointerValueToken};

impl IRCompilerState<'_> {
    pub fn emit_load_pointer(&mut self, pointer_pointer: PointerValueToken) -> PointerValueToken {
        todo!()
    }

    pub fn emit_load_float(&mut self, float_pointer: PointerValueToken) -> FloatValueToken {
        todo!()
    }

    pub fn emit_load_double(&mut self, double_pointer: PointerValueToken) -> DoubleValueToken {
        todo!()
    }

    pub fn emit_load_long(&mut self, long_pointer: PointerValueToken) -> LongValueToken {
        todo!()
    }

    pub fn emit_load_int(&mut self, int_pointer: PointerValueToken) -> IntegerValueToken {
        todo!()
    }

    pub fn emit_load_int_zero_extend(&mut self, int_pointer: PointerValueToken, size: Size) -> IntegerValueToken {
        todo!()
    }

    pub fn emit_load_int_sign_extend(&mut self, int_pointer: PointerValueToken, size: Size) -> IntegerValueToken {
        todo!()
    }
}

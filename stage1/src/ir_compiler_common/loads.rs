use another_jit_vm_ir::compiler::Size;
use crate::ir_compiler_common::{DoubleValueToken, FloatValueToken, IntegerValueToken, IRCompilerState, LongValueToken, PointerValueToken};

impl IRCompilerState<'_> {
    pub fn emit_load_pointer(&mut self, _pointer_pointer: PointerValueToken) -> PointerValueToken {
        todo!()
    }

    pub fn emit_load_float(&mut self, _float_pointer: PointerValueToken) -> FloatValueToken {
        todo!()
    }

    pub fn emit_load_double(&mut self, _double_pointer: PointerValueToken) -> DoubleValueToken {
        todo!()
    }

    pub fn emit_load_long(&mut self, _long_pointer: PointerValueToken) -> LongValueToken {
        todo!()
    }

    pub fn emit_load_int(&mut self, _int_pointer: PointerValueToken) -> IntegerValueToken {
        todo!()
    }

    pub fn emit_load_int_zero_extend(&mut self, _int_pointer: PointerValueToken, _size: Size) -> IntegerValueToken {
        todo!()
    }

    pub fn emit_load_int_sign_extend(&mut self, _int_pointer: PointerValueToken, _size: Size) -> IntegerValueToken {
        todo!()
    }
}

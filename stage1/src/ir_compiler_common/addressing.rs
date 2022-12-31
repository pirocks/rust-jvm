use std::num::NonZeroUsize;
use crate::ir_compiler_common::{IntegerValueToken, IRCompilerState, PointerValueToken};

impl IRCompilerState{
    pub fn emit_address_calculate_int(&mut self, pointer: PointerValueToken, index: IntegerValueToken, constant_offset: usize, constant_index_multiplier: NonZeroUsize) -> PointerValueToken{
        todo!()
    }
}


use std::num::NonZeroUsize;
use crate::ir_compiler_common::{IntegerValueToken, IRCompilerState, PointerValueToken};

impl IRCompilerState<'_>{
    pub fn emit_address_calculate_int(&mut self, _pointer: PointerValueToken, _index: IntegerValueToken, _constant_offset: usize, _constant_index_multiplier: NonZeroUsize) -> PointerValueToken{
        todo!()
    }
}


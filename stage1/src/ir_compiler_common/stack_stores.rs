use crate::ir_compiler_common::{DoubleValueToken, FloatValueToken, IntegerValueToken, LongValueToken, PointerValueToken};
use crate::ir_compiler_common::special::{IRCompilerState};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum StackPos{
    AfterFromEnd(u16),
    BeforeFromEnd(u16)
}

impl IRCompilerState<'_>{
    pub fn emit_stack_store_int(&mut self, from_end: StackPos, to_store: IntegerValueToken){
        todo!()
    }

    pub fn emit_stack_store_long(&mut self, from_end: StackPos, to_store: LongValueToken){
        todo!()
    }

    pub fn emit_stack_store_float(&mut self, from_end: StackPos, to_store: FloatValueToken){
        todo!()
    }

    pub fn emit_stack_store_double(&mut self, from_end: StackPos, to_store: DoubleValueToken){
        todo!()
    }

    pub fn emit_stack_store_pointer(&mut self, from_end: StackPos, to_store: PointerValueToken){
        todo!()
    }
}


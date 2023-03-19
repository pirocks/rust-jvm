use another_jit_vm_ir::compiler::Size;
use array_memory_layout::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use crate::ir_compiler_common::{IntegerValueToken, ONE, PointerValueToken};
use crate::ir_compiler_common::special::IRCompilerState;

impl IRCompilerState<'_> {
    pub fn emit_local_load_pointer(&mut self, local_var: u16) -> PointerValueToken{
        todo!()
    }

    pub fn emit_local_load_integer(&mut self, local_var: u16) -> IntegerValueToken{
        todo!()
    }
}


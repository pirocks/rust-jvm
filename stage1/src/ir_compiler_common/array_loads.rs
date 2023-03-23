use another_jit_vm_ir::compiler::Size;
use array_memory_layout::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use crate::ir_compiler_common::{IntegerValueToken, ONE, PointerValue, PointerValueToken, Stage1IRInstr};
use crate::ir_compiler_common::special::IRCompilerState;

impl IRCompilerState<'_> {
    pub fn emit_local_load_pointer(&mut self, local_var: u16) -> PointerValueToken {
        self.current_local_var_tokens[local_var as usize].unwrap_pointer()
    }

    pub fn emit_local_load_integer(&mut self, local_var: u16) -> IntegerValueToken{
        self.current_local_var_tokens[local_var as usize].unwrap_integer()
    }
}


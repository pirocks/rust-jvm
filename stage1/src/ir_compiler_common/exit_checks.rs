use rust_jvm_common::compressed_classfile::class_names::CClassName;
use crate::ir_compiler_common::{IntegerValueToken, IRCompilerState, PointerValueToken};

impl IRCompilerState{
    pub fn emit_npe_check(&mut self, to_check: PointerValueToken) {
        //todo to get best perf should put these after end of main function
        //since branches to this are unlikely
        todo!()
    }

    pub fn emit_array_bounds_check(&mut self, len: IntegerValueToken, int: IntegerValueToken){
        todo!()
    }

    pub fn emit_array_store_check(&mut self, interface: CClassName){
        todo!()
    }
}


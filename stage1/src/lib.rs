#![feature(const_option)]

use another_jit_vm::{IRMethodID};
use compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};
use rust_jvm_common::{ByteCodeIndex, MethodId};
use crate::ir_compiler_common::{Stage1IRInstr};
use crate::ir_compiler_common::special::IRCompilerState;
use crate::java_compiler::emit_single_instruction;

//todo fix instanceof/checkcast
//todo fix class loaders
//todo make a get object class fast path

//todo maybe an r15 offset consts makes sense here as well
pub mod native_compiler_common;
pub mod ir_compiler_common;
pub mod java_compiler;
pub mod frame_layout;
pub mod registers_state;

pub fn compile_to_ir<'vm>(resolver: &impl MethodResolver<'vm>, method_frame_data: &JavaCompilerMethodAndFrameData, method_id: MethodId, ir_method_id: IRMethodID) -> Vec<Stage1IRInstr> {
    //todo use ir emit functions
    let c_method_desc = resolver.lookup_method_desc(method_id);
    let mut compiler_state = IRCompilerState::new(method_id, ir_method_id, method_frame_data, &c_method_desc,false);
    compiler_state.emit_ir_start();
    if method_frame_data.should_synchronize {
        if method_frame_data.is_static {
            let class_object = compiler_state.emit_get_class_object();
            compiler_state.emit_monitor_enter(class_object);
        } else {
            let this_object = compiler_state.emit_load_arg_pointer(0);
            compiler_state.emit_monitor_enter(this_object);
        }
    }
    let code = resolver.get_compressed_code(method_id);
    for (i,(java_pc, instr)) in code.instructions.iter().enumerate() {
        compiler_state.notify_before_instruction(*java_pc, ByteCodeIndex(i as u16));
        emit_single_instruction(&mut compiler_state, instr);
        compiler_state.notify_after_instruction(*java_pc);
    }
    //todo returns need to handle monitor_enter_exit
    compiler_state.complete()
}


pub struct CompilerState {}

impl CompilerState {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
pub mod ir_tests{

    fn compile_and_run_instructions() {

    }

    #[test]
    pub fn test_iadd() {

    }
}

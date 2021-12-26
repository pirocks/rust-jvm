use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::CClassName;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::vm_exit_abi::IRVMExitType;
use crate::jit::ir::IRInstr;
use crate::jit::MethodResolver;

pub fn new(resolver: &MethodResolver<'vm_life>, ccn: CClassName) -> impl Iterator<Item=IRInstr> {
    match resolver.lookup_type_loaded(&(ccn).into()) {
        None => {
            array_into_iter([IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: todo!(),
                    this_method_id: todo!(),
                    return_to_bytecode_index: todo!(),
                },
            }])
        }
        Some((loaded_class, loader)) => {
            todo!()
        }
    }
}


pub fn anewarray(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    elem_type: &CPDType,
) -> impl Iterator<Item=IRInstr> {
    match resolver.lookup_type_loaded(elem_type) {
        None => {
            let restart_point = IRInstr::RestartPoint(current_instr_data.current_index);
            let cpd_type_id = resolver.get_cpdtype_id(elem_type);
            array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: cpd_type_id,
                        this_method_id: method_frame_data.current_method_id,
                        return_to_bytecode_index: current_instr_data.current_index,
                    },
                }])
        }
        Some((loaded_class, loader)) => {
            todo!()
        }
    }
}

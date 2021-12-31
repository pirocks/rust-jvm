use std::sync::Arc;

use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::loading::LoaderName;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::vm_exit_abi::IRVMExitType;
use crate::jit::ir::IRInstr;
use crate::jit::MethodResolver;
use crate::runtime_class::RuntimeClass;

pub fn putstatic(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    target_class: CClassName,
    name: FieldName,
) -> impl Iterator<Item=IRInstr> {
    let restart_point = IRInstr::RestartPoint(current_instr_data.current_index);
    match resolver.lookup_type_loaded(&target_class.into()) {
        None => {
            array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: todo!(),
                        this_method_id: method_frame_data.current_method_id,
                        return_to_bytecode_index: todo!(),
                    },
                }])
        }
        Some((rc, loader)) => {
            let field_id = resolver.get_field_id(rc, name);
            array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::PutStatic {
                        field_id,
                        value: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                    }
                }])
        }
    }
}

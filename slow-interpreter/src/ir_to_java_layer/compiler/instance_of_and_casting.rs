use another_jit_vm_ir::compiler::IRInstr;
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use gc_memory_layout_common::FramePointerOffset;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::cpdtype_table::CPDTypeID;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::jit::MethodResolver;

pub fn checkcast(resolver: &MethodResolver, method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, cpdtype: &CPDType) -> impl Iterator<Item=IRInstr> {
    let cpdtype_id = resolver.get_cpdtype_id(cpdtype);
    let frame_pointer_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    array_into_iter([checkcast_impl(cpdtype_id, frame_pointer_offset)])
}

pub(crate) fn checkcast_impl(cpdtype_id: CPDTypeID, frame_pointer_offset: FramePointerOffset) -> IRInstr {
    IRInstr::VMExit2 {
        exit_type: IRVMExitType::CheckCast {
            value: frame_pointer_offset,
            cpdtype: cpdtype_id,
        }
    }
}

pub fn instanceof(resolver: &MethodResolver, method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, cpdtype: &CPDType) -> impl Iterator<Item=IRInstr> {
    let cpdtype_id = resolver.get_cpdtype_id(cpdtype);
    array_into_iter([
        IRInstr::VMExit2 {
            exit_type: IRVMExitType::InstanceOf {
                value: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                res: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                cpdtype: cpdtype_id,
            }
        }
    ])
}

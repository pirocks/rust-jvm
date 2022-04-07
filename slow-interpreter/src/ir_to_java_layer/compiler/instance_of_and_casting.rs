use another_jit_vm::{FramePointerOffset, Register};
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use gc_memory_layout_common::memory_regions::{BaseAddressAndMask};
use rust_jvm_common::compressed_classfile::CPDType;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::jit::MethodResolver;

pub fn checkcast(resolver: &MethodResolver, method_frame_data: &JavaCompilerMethodAndFrameData, mut current_instr_data: CurrentInstructionCompilerData, cpdtype: CPDType) -> impl Iterator<Item=IRInstr> {
    let frame_pointer_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    checkcast_impl(resolver, method_frame_data, &mut current_instr_data, cpdtype, frame_pointer_offset)
}

pub(crate) fn checkcast_impl(
    resolver: &MethodResolver,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &mut CurrentInstructionCompilerData,
    cpdtype: CPDType,
    frame_pointer_offset: FramePointerOffset
) -> impl Iterator<Item=IRInstr> {
    let masks_and_address = resolver.known_addresses_for_type(cpdtype);
    let cpdtype_id = resolver.get_cpdtype_id(cpdtype);
    let mut res = vec![];
    let mask_register = Register(1);
    let ptr_register = Register(2);
    let expected_constant_register = Register(3);
    let checkcast_succeeds = current_instr_data.compiler_labeler.local_label();
    for BaseAddressAndMask { mask, base_address } in masks_and_address {
        res.push(IRInstr::LoadFPRelative {
            from: frame_pointer_offset,
            to: ptr_register,
            size: Size::pointer()
        });
        res.push(IRInstr::Const64bit { to: mask_register, const_: mask });
        res.push(IRInstr::BinaryBitAnd {
            res: ptr_register,
            a: mask_register,
            size: Size::pointer()
        });
        res.push(IRInstr::Const64bit { to: expected_constant_register, const_: base_address as usize as u64 });
        res.push(IRInstr::BranchEqual {
            a: expected_constant_register,
            b: ptr_register,
            label: checkcast_succeeds,
            size: Size::pointer()
        });
    }
    res.push(IRInstr::VMExit2 {
        exit_type: IRVMExitType::CheckCast {
            value: frame_pointer_offset,
            cpdtype: cpdtype_id,
        }
    });
    res.push(IRInstr::Label(IRLabel{ name: checkcast_succeeds }));
    res.into_iter()
}

pub fn instanceof(resolver: &MethodResolver, method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, cpdtype: CPDType) -> impl Iterator<Item=IRInstr> {
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

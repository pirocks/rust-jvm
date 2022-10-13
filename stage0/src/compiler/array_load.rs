use std::mem::size_of;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Signed, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use gc_memory_layout_common::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use rust_jvm_common::NativeJavaValue;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData};
use crate::compiler::fields::{field_type_to_register_size, runtime_type_to_size};
use crate::compiler_common::JavaCompilerMethodAndFrameData;

pub fn caload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, CPDType::CharType)
}

pub fn baload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, CPDType::ByteType)
}


pub fn iaload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, CPDType::IntType)
}

pub fn aaload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, CClassName::object().into())
}

pub fn laload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, CPDType::LongType)
}

pub fn daload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, CPDType::DoubleType)
}

pub fn faload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, CPDType::FloatType)
}

pub fn saload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, CPDType::ShortType)
}


fn array_load_impl(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, arr_type: CPDType) -> impl Iterator<Item=IRInstr> {
    let index = Register(1);
    let array_ref = Register(2);
    let array_layout = ArrayMemoryLayout::from_cpdtype(arr_type);
    assert_eq!(array_layout.len_entry_offset(), 0);//needs to be zero for this impl
    assert_eq!(array_layout.elem_0_entry_offset(), size_of::<NativeJavaValue>());
    let elem_size = array_layout.elem_size();
    let native_jv_size_register = Register(3);
    let length = Register(4);
    let res = Register(5);
    let elem_register_size = field_type_to_register_size(arr_type);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: array_ref, size: Size::pointer() },
        IRInstr::NPECheck { possibly_null: array_ref, temp_register: index, npe_exit_type: IRVMExitType::NPE { java_pc: current_instr_data.current_offset } },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: index, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: array_ref, size: Size::pointer() },
        IRInstr::Const64bit { to: native_jv_size_register, const_: elem_size as u64 },
        IRInstr::Load { to: length, from_address: array_ref, size: Size::int() },
        IRInstr::Add { res: array_ref, a: native_jv_size_register, size: Size::pointer() },
        IRInstr::BoundsCheck { length, index, size: Size::int(), exit: IRVMExitType::ArrayOutOfBounds { java_pc: current_instr_data.current_offset } },
        IRInstr::MulConst { res: index, a: elem_size as i32, size: Size::pointer(), signed: Signed::Signed },
        IRInstr::Add { res: array_ref, a: index, size: Size::pointer() },
        IRInstr::Load { to: res, from_address: array_ref, size: elem_register_size },
        IRInstr::StoreFPRelative { from: res, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: runtime_type_to_size(&arr_type.to_runtime_type().unwrap()) }
    ])
}


use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Signed, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use array_memory_layout::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use crate::compiler::{array_into_iter, CurrentInstructionCompilerData};
use crate::compiler::fields::field_type_to_register_size;
use crate::compiler_common::JavaCompilerMethodAndFrameData;

pub fn sastore(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_store_impl(method_frame_data, current_instr_data, CPDType::ShortType)
}

pub fn castore(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_store_impl(method_frame_data, current_instr_data, CPDType::CharType)
}

pub fn bastore(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_store_impl(method_frame_data, current_instr_data, CPDType::BooleanType)
}

pub fn iastore(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_store_impl(method_frame_data, current_instr_data, CPDType::IntType)
}

pub fn fastore(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_store_impl(method_frame_data, current_instr_data, CPDType::FloatType)
}

pub fn aastore(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_store_impl(method_frame_data, current_instr_data, CClassName::object().into())
}

pub fn lastore(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_store_impl(method_frame_data, current_instr_data, CPDType::LongType)
}

pub fn dastore(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_store_impl(method_frame_data, current_instr_data, CPDType::DoubleType)
}

fn array_store_impl(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, elem_type: CPDType) -> impl Iterator<Item=IRInstr> {
    let index = Register(1);
    let array_ref = Register(2);
    let array_layout = ArrayMemoryLayout::from_cpdtype(elem_type);
    assert_eq!(array_layout.len_entry_offset(), 0);//needs to be zero for this impl
    // assert_eq!(array_layout.elem_0_entry_offset(), size_of::<NativeJavaValue>());
    let elem_0_offset_register = Register(3);
    let length = Register(4);
    let value = Register(5);
    let elem_register_size = field_type_to_register_size(elem_type);
    let index_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 1);
    array_into_iter([
        /*if elem_type == CPDType::CharType {
            IRInstr::DebuggerBreakpoint
        } else {
            IRInstr::NOP
        },*/
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 2), to: array_ref, size: Size::pointer() },
        IRInstr::NPECheck { possibly_null: array_ref, temp_register: index, npe_exit_type: IRVMExitType::NPE { java_pc: current_instr_data.current_offset } },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 2), to: array_ref, size: Size::pointer() },
        IRInstr::LoadFPRelative { from: index_offset, to: index, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value, size: elem_register_size },
        IRInstr::Const64bit { to: elem_0_offset_register, const_: array_layout.elem_0_entry_offset() as u64 },
        IRInstr::Load { to: length, from_address: array_ref, size: Size::int() },//todo shouldn't this check length offset
        IRInstr::Add { res: array_ref, a: elem_0_offset_register, size: Size::pointer() },
        IRInstr::BoundsCheck { length, index, size: Size::int(), exit: IRVMExitType::ArrayOutOfBounds { java_pc: current_instr_data.current_offset, index: index_offset } },
        IRInstr::SignExtend {
            from: index,
            to: index,
            from_size: Size::int(),
            to_size: Size::pointer(),
        },
        IRInstr::MulConst { res: index, a: array_layout.elem_size().get() as i32, size: Size::pointer(), signed: Signed::Signed },
        IRInstr::Add { res: array_ref, a: index, size: Size::pointer() },
        IRInstr::Store { from: value, to_address: array_ref, size: elem_register_size }
    ])
}



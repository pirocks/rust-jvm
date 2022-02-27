use std::mem::size_of;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Signed, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use jvmti_jni_bindings::jlong;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::java_values::NativeJavaValue;

pub fn caload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, Size::char())
}

pub fn baload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, Size::byte())
}

pub fn aaload(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_load_impl(method_frame_data, current_instr_data, Size::pointer())
}

fn array_load_impl(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, elem_size: Size) -> impl Iterator<Item=IRInstr> {
    let index = Register(1);
    let array_ref = Register(2);
    assert_eq!(size_of::<jlong>(), size_of::<NativeJavaValue>());
    let native_jv_size = size_of::<jlong>();
    let native_jv_size_register = Register(3);
    let length = Register(4);
    let res = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: array_ref, size: Size::pointer() },
        IRInstr::NPECheck { possibly_null: array_ref, temp_register: index, npe_exit_type: IRVMExitType::NPE },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: index, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: array_ref, size: Size::pointer() },
        IRInstr::Const64bit { to: native_jv_size_register, const_: native_jv_size as u64 },
        IRInstr::Load { to: length, from_address: array_ref, size: Size::int() },
        IRInstr::Add { res: array_ref, a: native_jv_size_register, size: Size::pointer() },
        IRInstr::BoundsCheck { length, index, size: Size::int() },
        IRInstr::MulConst { res: index, a: native_jv_size as i32, size: Size::pointer(), signed: Signed::Unsigned },
        IRInstr::Add { res: array_ref, a: index, size: Size::pointer() },
        IRInstr::Load { to: res, from_address: array_ref, size: elem_size },
        IRInstr::StoreFPRelative { from: res, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: elem_size }
    ])
}


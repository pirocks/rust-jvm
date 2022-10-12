use another_jit_vm::{FloatRegister, Register};
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::CPDType;
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData, MethodRecompileConditions, NeedsRecompileIf};
use crate::compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};

pub fn ldc_string<'vm>(resolver: &impl MethodResolver<'vm>,
                       method_frame_data: &JavaCompilerMethodAndFrameData,
                       current_instr_data: &CurrentInstructionCompilerData,
                       restart_point_generator: &mut RestartPointGenerator,
                       recompile_conditions: &mut MethodRecompileConditions,
                       str: CompressedWtf8String) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let string_class_cpdtype = CClassName::string().into();
    match resolver.lookup_type_inited_initing(&string_class_cpdtype) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(string_class_cpdtype);
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: string_class_cpdtype });
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset,
                }
            }])
        }
        Some((_loaded_class, _loader)) => {
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::NewString {
                    res: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                    compressed_wtf8_buf: str,
                    java_pc: current_instr_data.current_offset,
                }
            }])
        }
    }
}

pub fn ldc_class<'vm>(resolver: &impl MethodResolver<'vm>,
                      method_frame_data: &JavaCompilerMethodAndFrameData,
                      current_instr_data: &CurrentInstructionCompilerData,
                      restart_point_generator: &mut RestartPointGenerator,
                      recompile_conditions: &mut MethodRecompileConditions,
                      type_: &CPDType) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let to_load_cpdtype = type_.clone();
    let cpd_type_id = resolver.get_cpdtype_id(to_load_cpdtype);
    //todo we could do this in the exit and cut down on recompilations
    match resolver.lookup_type_inited_initing(&to_load_cpdtype) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: to_load_cpdtype });
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset,
                }
            }])
        }
        Some((_loaded_class, _loader)) => {
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::NewClass {
                    res: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                    type_: cpd_type_id,
                    java_pc: current_instr_data.current_offset,
                }
            }])
        }
    }
}


pub fn ldc_float(method_frame_data: &JavaCompilerMethodAndFrameData,
                 current_instr_data: &CurrentInstructionCompilerData,
                 float: f32) -> impl Iterator<Item=IRInstr> {
    let target_offset = method_frame_data.operand_stack_entry(current_instr_data.next_index, 0);
    array_into_iter([
        IRInstr::Const32bit { to: Register(1), const_: float.to_bits() },
        IRInstr::StoreFPRelative { from: Register(1), to: target_offset, size: Size::float() },
        IRInstr::LoadFPRelativeFloat { from: target_offset, to: FloatRegister(1) }
    ])
}

pub fn ldc_integer(method_frame_data: &JavaCompilerMethodAndFrameData,
                   current_instr_data: &CurrentInstructionCompilerData,
                   integer: i32) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::Const32bit { to: Register(1), const_: integer as u32 },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }])
}

pub fn ldc_double(method_frame_data: &JavaCompilerMethodAndFrameData,
                  current_instr_data: &CurrentInstructionCompilerData,
                  float: f64) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::Const64bit { to: Register(1), const_: float.to_bits() },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::double() }])
}

pub fn ldc_long(method_frame_data: &JavaCompilerMethodAndFrameData,
                current_instr_data: &CurrentInstructionCompilerData,
                long: i64) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::Const64bit { to: Register(1), const_: long as u64 },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }])
}
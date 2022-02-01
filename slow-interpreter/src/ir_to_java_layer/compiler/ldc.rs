use std::sync::Arc;

use iced_x86::CC_b::c;
use wtf8::Wtf8Buf;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::jit::MethodResolver;
use crate::runtime_class::RuntimeClass;

pub fn ldc_string(resolver: &MethodResolver<'vm_life>,
                  method_frame_data: &JavaCompilerMethodAndFrameData,
                  current_instr_data: &CurrentInstructionCompilerData,
                  restart_point_generator: &mut RestartPointGenerator,
                  str: CompressedWtf8String) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let string_class_cpdtype = CClassName::string().into();
    match resolver.lookup_type_loaded(&string_class_cpdtype) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(&string_class_cpdtype);
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                }
            }])
        }
        Some((loaded_class, loader)) => {
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::NewString {
                    res: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                    compressed_wtf8_buf: str,
                }
            }])
        }
    }
}

pub fn ldc_class(resolver: &MethodResolver<'vm_life>,
                 method_frame_data: &JavaCompilerMethodAndFrameData,
                 current_instr_data: &CurrentInstructionCompilerData,
                 restart_point_generator: &mut RestartPointGenerator,
                 type_: &CPDType) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let to_load_cpdtype = type_.clone();
    let cpd_type_id = resolver.get_cpdtype_id(&to_load_cpdtype);
    //todo we could do this in the exit and cut down on recompilations
    match resolver.lookup_type_loaded(&to_load_cpdtype) {
        None => {
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                }
            }])
        }
        Some((loaded_class, loader)) => {
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::NewClass {
                    res: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                    type_: cpd_type_id,
                }
            }])
        }
    }
}


pub fn ldc_float(method_frame_data: &JavaCompilerMethodAndFrameData,
                 current_instr_data: &CurrentInstructionCompilerData,
                 float: f32) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::Const32bit { to: Register(1), const_: float.to_bits() },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }])
}
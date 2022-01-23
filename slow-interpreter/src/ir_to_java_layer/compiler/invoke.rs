use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use gc_memory_layout_common::{FramePointerOffset, StackframeMemoryLayout};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;

use crate::ir_to_java_layer::compiler::{array_into_iter, CompilerLabeler, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::jit::MethodResolver;

pub fn invokespecial(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    method_name: MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: &CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let class_cpdtype = CPDType::Ref(classname_ref_type.clone());
    let restart_point_id_class_load = restart_point_generator.new_restart_point();
    let restart_point_class_load = IRInstr::RestartPoint(restart_point_id_class_load);
    match resolver.lookup_type_loaded(&class_cpdtype) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(&CPDType::Ref(classname_ref_type.clone()));
            Either::Left(array_into_iter([restart_point_class_load,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::LoadClassAndRecompile {
                        class: cpd_type_id,
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id: restart_point_id_class_load,
                    },
                }]))
        }
        Some((rc, loader)) => {
            let view = rc.view();
            let (method_id, is_native) = resolver.lookup_special(class_cpdtype, method_name, descriptor.clone()).unwrap();
            let maybe_address = resolver.lookup_address(method_id);
            let restart_point_id_function_address = restart_point_generator.new_restart_point();
            let restart_point_function_address = IRInstr::RestartPoint(restart_point_id_function_address);
            Either::Right(match maybe_address {
                None => {
                    let exit_instr = IRInstr::VMExit2 {
                        exit_type: IRVMExitType::CompileFunctionAndRecompileCurrent {
                            current_method_id: method_frame_data.current_method_id,
                            target_method_id: method_id,
                            restart_point_id: restart_point_id_function_address,
                        }
                    };
                    //todo have restart point ids for matching same restart points
                    Either::Left(array_into_iter([restart_point_class_load, restart_point_function_address, exit_instr]))
                }
                Some((ir_method_id, address)) => {
                    let method_layout = resolver.lookup_method_layout(method_id);
                    if is_native {
                        todo!()
                    } else {
                        Either::Right(array_into_iter([restart_point_class_load, restart_point_function_address, IRInstr::IRCall {
                            temp_register_1: Register(1),
                            temp_register_2: Register(2),
                            current_frame_size: method_frame_data.full_frame_size(),
                            new_frame_size: method_layout.full_frame_size(),
                            new_method_id: method_id,
                            new_ir_method_id: ir_method_id,
                            target_address: address,
                        }]))
                    }
                }
            })
        }
    }
}

pub fn invokestatic(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    method_name: MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: &CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let class_as_cpdtype = CPDType::Ref(classname_ref_type.clone());
    match resolver.lookup_static(class_as_cpdtype.clone(), method_name, descriptor.clone()) {
        None => {
            let cpdtype_id = resolver.get_cpdtype_id(&class_as_cpdtype);
            Either::Left(array_into_iter([restart_point,IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpdtype_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id
                },
            }]))
        }
        Some((method_id, is_native)) => {
            Either::Right(if is_native {
                let exit_label = current_instr_data.compiler_labeler.label_at(current_instr_data.current_offset);
                let num_args = resolver.num_args(method_id);
                let arg_start_frame_offset = if num_args != 0 {
                    method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args)
                } else {
                    FramePointerOffset(usize::MAX)
                };
                array_into_iter([restart_point,IRInstr::VMExit2 {
                    exit_type: IRVMExitType::RunStaticNative {
                        method_id,
                        arg_start_frame_offset,
                        num_args,
                    },
                }])
            } else {
                todo!()
            })
        }
    }
}

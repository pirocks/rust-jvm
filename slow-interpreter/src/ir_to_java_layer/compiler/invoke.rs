use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRCallTarget, IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::{InvokeVirtualResolve, IRVMExitType};
use gc_memory_layout_common::{FramePointerOffset, StackframeMemoryLayout};
use rust_jvm_common::classfile::InstructionInfo::jsr;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::MethodId;

use crate::ir_to_java_layer::compiler::{array_into_iter, ByteCodeIndex, CompilerLabeler, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
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
            let maybe_address = resolver.lookup_ir_method_id_and_address(method_id);
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
                    if is_native {
                        todo!()
                    } else {
                        let target_method_layout = resolver.lookup_method_layout(method_id);
                        let num_args = descriptor.arg_types.len();
                        let arg_from_to_offsets = virtual_and_special_arg_offsets(resolver, method_frame_data, &current_instr_data, descriptor, method_id);
                        Either::Right(array_into_iter([restart_point_class_load, restart_point_function_address, IRInstr::IRCall {
                            temp_register_1: Register(1),
                            temp_register_2: Register(2),
                            arg_from_to_offsets,
                            return_value: if let CompressedParsedDescriptorType::VoidType = descriptor.return_type {
                                None
                            } else {
                                Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                            },
                            target_address: IRCallTarget::Constant {
                                address,
                                new_frame_size: target_method_layout.full_frame_size(),
                                method_id,
                                ir_method_id,
                            },
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
            Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpdtype_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                },
            }]))
        }
        Some((method_id, is_native)) => {
            Either::Right(if is_native {
                let exit_label = current_instr_data.compiler_labeler.label_at(current_instr_data.current_offset);
                let num_args = resolver.num_args(method_id);
                let arg_start_frame_offset = if num_args != 0 {
                    Some(method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args - 1))
                } else {
                    None
                };
                array_into_iter([restart_point, IRInstr::VMExit2 {
                    exit_type: IRVMExitType::RunStaticNative {
                        method_id,
                        arg_start_frame_offset,
                        res_pointer_offset: if descriptor.return_type.is_void() {
                            None
                        } else {
                            Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                        },
                        num_args,
                    },
                }])
            } else {
                todo!()
            })
        }
    }
}


pub fn invokevirtual(
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
    match resolver.lookup_virtual(CPDType::Ref(classname_ref_type.clone()), method_name, descriptor.clone()) {
        None => {
            Either::Left(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: todo!(),
                        this_method_id: todo!(),
                        restart_point_id: todo!(),
                    },
                }]))
        }
        Some((method_id, is_native)) => {
            Either::Right(if is_native {
                todo!()
            } else {
                // todo have a vm exit which performs the lookup
                // investigate ways of making IRcall work for variable targets,
                // and investigate size of table for invokevirtual without tagging.
                let num_args = descriptor.arg_types.len();
                let arg_from_to_offsets = virtual_and_special_arg_offsets(resolver, method_frame_data, &current_instr_data, descriptor, method_id);
                array_into_iter([restart_point,
                    IRInstr::VMExit2 {
                        exit_type: IRVMExitType::InvokeVirtualResolve {
                            object_ref: method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args as u16),
                            inheritance_method_id: resolver.lookup_inheritance_method_id(method_id),
                            target_method_id: method_id
                        }
                    },
                    IRInstr::IRCall {
                        temp_register_1: Register(1),
                        temp_register_2: Register(2),
                        arg_from_to_offsets,
                        return_value: Some(method_frame_data.operand_stack_entry(current_instr_data.next_index,0)),
                        target_address: IRCallTarget::Variable {
                            address: InvokeVirtualResolve::ADDRESS_RES,
                            ir_method_id: InvokeVirtualResolve::IR_METHOD_ID_RES,
                            method_id: InvokeVirtualResolve::METHOD_ID_RES,
                            new_frame_size: InvokeVirtualResolve::NEW_FRAME_SIZE_RES,
                        },
                    }])
            })
        }
    }
}

fn virtual_and_special_arg_offsets(resolver: &MethodResolver<'vm_life>, method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, descriptor: &CMethodDescriptor, target_method_id: MethodId) -> Vec<(FramePointerOffset, FramePointerOffset)> {
    let target_method_layout = resolver.lookup_method_layout(target_method_id);
    let num_args = descriptor.arg_types.len();
    let mut arg_from_to_offsets = vec![];
    for (i, arg_type) in descriptor.arg_types.iter().enumerate() {
        let from = method_frame_data.operand_stack_entry(current_instr_data.current_index, (num_args - i - 1) as u16);
        let to = target_method_layout.local_var_entry(ByteCodeIndex(0), i as u16);
        arg_from_to_offsets.push((from, to))
    }
    let object_ref_from = method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args as u16);
    let object_ref_to = target_method_layout.local_var_entry(ByteCodeIndex(0), 0);
    arg_from_to_offsets.push((object_ref_from, object_ref_to));
    arg_from_to_offsets
}
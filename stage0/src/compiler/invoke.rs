use std::mem::size_of;

use itertools::Either;

use another_jit_vm::{FramePointerOffset, Register};
use another_jit_vm_ir::compiler::{IRCallTarget, IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use another_jit_vm_ir::vm_exit_abi::register_structs::{InvokeInterfaceResolve, InvokeVirtualResolve};
use gc_memory_layout_common::layout::{FRAME_HEADER_END_OFFSET, FrameHeader};
use jvmti_jni_bindings::jlong;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::method_shape::MethodShape;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData, MethodRecompileConditions, NeedsRecompileIf};
use crate::compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};

pub fn invokespecial<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    method_name: MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let class_cpdtype = classname_ref_type.to_cpdtype();
    let restart_point_id_class_load = restart_point_generator.new_restart_point();
    let restart_point_class_load = IRInstr::RestartPoint(restart_point_id_class_load);
    let restart_point_id_function_address = restart_point_generator.new_restart_point();
    let restart_point_function_address = IRInstr::RestartPoint(restart_point_id_function_address);
    match resolver.lookup_type_inited_initing(&class_cpdtype) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(classname_ref_type.to_cpdtype());
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: class_cpdtype });
            Either::Left(array_into_iter([restart_point_class_load,
                restart_point_function_address,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::LoadClassAndRecompile {
                        class: cpd_type_id,
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id: restart_point_id_class_load,
                        java_pc: current_instr_data.current_offset,
                    },
                    skipable_exit_id: None
                }]))
        }
        Some((_rc, _loader)) => {
            let (method_id, _is_native) = resolver.lookup_special(&class_cpdtype, method_name, descriptor.clone()).unwrap();
            let maybe_address = resolver.lookup_ir_method_id_and_address(method_id);
            match maybe_address {
                None => {
                    recompile_conditions.add_condition(NeedsRecompileIf::FunctionCompiled { method_id });
                    let exit_instr = IRInstr::VMExit2 {
                        exit_type: IRVMExitType::CompileFunctionAndRecompileCurrent {
                            current_method_id: method_frame_data.current_method_id,
                            target_method_id: method_id,
                            restart_point_id: restart_point_id_function_address,
                            java_pc: current_instr_data.current_offset,
                        },
                        skipable_exit_id: None
                    };
                    //todo have restart point ids for matching same restart points
                    Either::Left(array_into_iter([restart_point_class_load, restart_point_function_address, exit_instr]))
                }
                Some((ir_method_id, address)) => {
                    recompile_conditions.add_condition(NeedsRecompileIf::FunctionRecompiled { function_method_id: method_id, current_ir_method_id: ir_method_id });
                    let arg_from_to_offsets = virtual_and_special_arg_offsets(method_frame_data, &current_instr_data, descriptor);
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
                            method_id
                        },
                        current_frame_size: method_frame_data.full_frame_size(),
                    }]))
                }
            }
        }
    }
}

pub fn invokestatic<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    method_name: MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: &CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let class_init_restart_point = IRInstr::RestartPoint(restart_point_id);
    let restart_point_id_function_address = restart_point_generator.new_restart_point();
    let restart_point_function_address = IRInstr::RestartPoint(restart_point_id_function_address);
    let class_as_cpdtype = classname_ref_type.to_cpdtype();
    match resolver.lookup_static(class_as_cpdtype.clone(), method_name, descriptor.clone()) {
        None => {
            let cpdtype_id = resolver.get_cpdtype_id(class_as_cpdtype);
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: class_as_cpdtype });
            Either::Left(array_into_iter([class_init_restart_point,
                restart_point_function_address,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: cpdtype_id,
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                        java_pc: current_instr_data.current_offset,
                        edit_action: None,
                        skipable_exit_id: None
                    },
                    skipable_exit_id: None
                }]))
        }
        Some((method_id, _is_native)) => {
            let arg_from_to_offsets = static_arg_offsets(method_frame_data, &current_instr_data, descriptor);
            Either::Right(match resolver.lookup_ir_method_id_and_address(method_id) {
                None => {
                    recompile_conditions.add_condition(NeedsRecompileIf::FunctionCompiled { method_id });
                    let exit_instr = IRInstr::VMExit2 {
                        exit_type: IRVMExitType::CompileFunctionAndRecompileCurrent {
                            current_method_id: method_frame_data.current_method_id,
                            target_method_id: method_id,
                            restart_point_id: restart_point_id_function_address,
                            java_pc: current_instr_data.current_offset,
                        },
                        skipable_exit_id: None
                    };
                    //todo have restart point ids for matching same restart points
                    Either::Left(array_into_iter([class_init_restart_point,
                        restart_point_function_address,
                        exit_instr]))
                }
                Some((ir_method_id, address)) => {
                    recompile_conditions.add_condition(NeedsRecompileIf::FunctionRecompiled { function_method_id: method_id, current_ir_method_id: ir_method_id });
                    Either::Right(array_into_iter([class_init_restart_point,
                        restart_point_function_address,
                        IRInstr::IRCall {
                            temp_register_1: Register(1),
                            temp_register_2: Register(2),
                            arg_from_to_offsets,
                            return_value: if descriptor.return_type.is_void() {
                                None
                            } else {
                                Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                            },
                            target_address: IRCallTarget::Constant {
                                address,
                                method_id
                            },
                            current_frame_size: method_frame_data.full_frame_size(),
                        }]))
                }
            })
        }
    }
}


pub fn invokevirtual<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    method_name: MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let after_call_restart_point_id = restart_point_generator.new_restart_point();
    let after_call_restart_point = IRInstr::RestartPoint(after_call_restart_point_id);
    let target_class_type = classname_ref_type.to_cpdtype();
    let target_class_type_id = resolver.get_cpdtype_id(target_class_type);

    if resolver.lookup_type_inited_initing(&target_class_type).is_none() {}
    let rc = match resolver.lookup_type_inited_initing(&target_class_type) {
        Some((rc, _)) => {
            rc
        }
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: target_class_type });
            //todo this should never happen?
            return Either::Left(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: target_class_type_id,
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                        java_pc: current_instr_data.current_offset,
                        edit_action: None,
                        skipable_exit_id: None
                    },
                    skipable_exit_id: None
                }, after_call_restart_point]));
        }
    };
    let num_args = descriptor.arg_types.len();

    // todo investigate size of table for invokevirtual without tagging.
    let arg_from_to_offsets = virtual_and_special_arg_offsets(method_frame_data, &current_instr_data, descriptor);
    //todo fix the generated lookup
    return Either::Right(array_into_iter([restart_point,
        IRInstr::VTableLookupOrExit {
            resolve_exit: IRVMExitType::InvokeVirtualResolve {
                object_ref: method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args as u16),
                method_shape_id: resolver.lookup_method_shape(MethodShape { name: method_name, desc: descriptor.clone() }),
                method_number: resolver.lookup_method_number(rc, MethodShape { name: method_name, desc: descriptor.clone() }),
                native_restart_point: after_call_restart_point_id,
                native_return_offset: if descriptor.return_type.is_void() {
                    None
                } else {
                    Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                },
                java_pc: current_instr_data.current_offset,
            }
        },
        IRInstr::IRCall {
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            arg_from_to_offsets,
            return_value: if descriptor.return_type.is_void() {
                None
            } else {
                Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
            },
            target_address: IRCallTarget::Variable {
                address: InvokeVirtualResolve::ADDRESS_RES,
            },
            current_frame_size: method_frame_data.full_frame_size(),
        },
        after_call_restart_point]));
}

pub fn invoke_interface<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    method_name: &MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: &CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let num_args = descriptor.arg_types.len() as u16;

    let target_class_cpdtype = classname_ref_type.to_cpdtype();
    let cpdtype_id = resolver.get_cpdtype_id(target_class_cpdtype);
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let after_call_restart_point_id = restart_point_generator.new_restart_point();
    let after_call_restart_point = IRInstr::RestartPoint(after_call_restart_point_id);
    match resolver.lookup_interface(&target_class_cpdtype, *method_name, descriptor.clone()) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: target_class_cpdtype });//todo this could be part of method resolver so that stuff always gets recompiled as needed
            Either::Right(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: cpdtype_id,
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                        java_pc: current_instr_data.current_offset,
                        edit_action: None,
                        skipable_exit_id: None
                    },
                    skipable_exit_id: None
                }, after_call_restart_point]))
        }
        Some((target_method_id, _is_native)) => {
            Either::Left(
                array_into_iter([
                    restart_point,
                    IRInstr::VMExit2 {
                        exit_type: IRVMExitType::InvokeInterfaceResolve {
                            object_ref: method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args),
                            target_method_id,
                            native_restart_point: after_call_restart_point_id,
                            native_return_offset: if descriptor.return_type.is_void() {
                                None
                            } else {
                                Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                            },
                            java_pc: current_instr_data.current_offset,
                        },
                        skipable_exit_id: None
                    },
                    IRInstr::IRCall {
                        temp_register_1: Register(1),
                        temp_register_2: Register(2),
                        arg_from_to_offsets: virtual_and_special_arg_offsets(method_frame_data, &current_instr_data, descriptor),
                        return_value: if descriptor.return_type.is_void() {
                            None
                        } else {
                            Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                        },
                        target_address: IRCallTarget::Variable {
                            address: InvokeInterfaceResolve::ADDRESS_RES,
                            // ir_method_id: InvokeInterfaceResolve::IR_METHOD_ID_RES,
                            // method_id: InvokeInterfaceResolve::METHOD_ID_RES,
                            // new_frame_size: InvokeInterfaceResolve::NEW_FRAME_SIZE_RES,
                        },
                        current_frame_size: method_frame_data.full_frame_size(),
                    }
                    , after_call_restart_point])
            )
        }
    }
}

fn virtual_and_special_arg_offsets<'vm>(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, descriptor: &CMethodDescriptor) -> Vec<(FramePointerOffset, FramePointerOffset)> {
    let num_args = descriptor.arg_types.len();
    let mut arg_from_to_offsets = vec![];
    let mut local_var_i = 0;
    for (operand_stack_i, arg_type) in descriptor.arg_types.iter().enumerate() {
        let from = method_frame_data.operand_stack_entry(current_instr_data.current_index, (num_args - operand_stack_i - 1) as u16);
        assert_eq!(size_of::<FrameHeader>(), FRAME_HEADER_END_OFFSET);
        let to = FramePointerOffset(FRAME_HEADER_END_OFFSET + (local_var_i as u16 + 1) as usize * size_of::<jlong>());
        arg_from_to_offsets.push((from, to));
        match arg_type {
            CompressedParsedDescriptorType::LongType |
            CompressedParsedDescriptorType::DoubleType => {
                local_var_i += 2;
            }
            _ => {
                local_var_i += 1;
            }
        }
    }
    let object_ref_from = method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args as u16);
    let object_ref_to = FramePointerOffset(FRAME_HEADER_END_OFFSET);
    arg_from_to_offsets.push((object_ref_from, object_ref_to));
    arg_from_to_offsets
}


fn static_arg_offsets<'vm>(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, descriptor: &CMethodDescriptor) -> Vec<(FramePointerOffset, FramePointerOffset)> {
    let num_args = descriptor.arg_types.len();
    let mut arg_from_to_offsets = vec![];
    let mut local_var_i = 0;
    //todo this is jank needs to better
    for (operand_stack_i, arg_type) in descriptor.arg_types.iter().enumerate() {
        let from = method_frame_data.operand_stack_entry(current_instr_data.current_index, (num_args - operand_stack_i - 1) as u16);
        let to = FramePointerOffset(FRAME_HEADER_END_OFFSET + local_var_i * size_of::<jlong>());
        arg_from_to_offsets.push((from, to));
        match arg_type {
            CompressedParsedDescriptorType::LongType |
            CompressedParsedDescriptorType::DoubleType => {
                local_var_i += 2;
            }
            _ => {
                local_var_i += 1;
            }
        }
    }
    arg_from_to_offsets
}
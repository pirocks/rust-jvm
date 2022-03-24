use std::ffi::c_void;
use std::mem::size_of;

use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRCallTarget, IRInstr, RestartPointGenerator};
use another_jit_vm_ir::ir_stack::FRAME_HEADER_END_OFFSET;
use another_jit_vm_ir::IRMethodID;
use another_jit_vm_ir::vm_exit_abi::{InvokeInterfaceResolve, InvokeVirtualResolve, IRVMExitType};
use gc_memory_layout_common::{FramePointerOffset, StackframeMemoryLayout};
use jvmti_jni_bindings::jlong;
use rust_jvm_common::classfile::InstructionInfo::jsr;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::method_shape::MethodShape;
use rust_jvm_common::{ByteCodeIndex, MethodId};

use crate::ir_to_java_layer::compiler::{array_into_iter, CompilerLabeler, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData, MethodRecompileConditions, NeedsRecompileIf};
use crate::jit::MethodResolver;

pub fn invokespecial<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    method_name: MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: &CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let class_cpdtype = CPDType::Ref(classname_ref_type.clone());
    let restart_point_id_class_load = restart_point_generator.new_restart_point();
    let restart_point_class_load = IRInstr::RestartPoint(restart_point_id_class_load);
    let restart_point_id_function_address = restart_point_generator.new_restart_point();
    let restart_point_function_address = IRInstr::RestartPoint(restart_point_id_function_address);
    match resolver.lookup_type_inited_initing(&class_cpdtype) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(&CPDType::Ref(classname_ref_type.clone()));
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: class_cpdtype });
            Either::Left(array_into_iter([restart_point_class_load,
                restart_point_function_address,
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
            let (method_id, is_native) = resolver.lookup_special(&class_cpdtype, method_name, descriptor.clone()).unwrap();
            let num_args = descriptor.arg_types.len();
            Either::Right(if is_native {
                //todo if ever native methods get compiled this will need a recompile check as well maybe
                Either::Left(array_into_iter([
                    restart_point_class_load,
                    restart_point_function_address,
                    IRInstr::VMExit2 {
                        exit_type: IRVMExitType::RunNativeSpecial {
                            method_id,
                            arg_start_frame_offset: method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args as u16),
                            res_pointer_offset: if CompressedParsedDescriptorType::VoidType == descriptor.return_type {
                                None
                            } else {
                                Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                            },
                            num_args: num_args as u16,
                        }
                    }
                ]))
            } else {
                let maybe_address = resolver.lookup_ir_method_id_and_address(method_id);
                Either::Right(match maybe_address {
                    None => {
                        recompile_conditions.add_condition(NeedsRecompileIf::FunctionCompiled { method_id });
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
                        recompile_conditions.add_condition(NeedsRecompileIf::FunctionRecompiled { function_method_id: method_id, current_ir_method_id: ir_method_id });
                        let target_method_layout = resolver.lookup_method_layout(method_id);
                        let arg_from_to_offsets = virtual_and_special_arg_offsets(resolver, method_frame_data, &current_instr_data, descriptor);
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
                            current_frame_size: method_frame_data.full_frame_size(),
                        }]))
                    }
                })
            })
        }
    }
}

pub fn invokestatic<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
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
    let class_as_cpdtype = CPDType::Ref(classname_ref_type.clone());
    match resolver.lookup_static(class_as_cpdtype.clone(), method_name, descriptor.clone()) {
        None => {
            let cpdtype_id = resolver.get_cpdtype_id(&class_as_cpdtype);
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: class_as_cpdtype });
            Either::Left(array_into_iter([class_init_restart_point,
                restart_point_function_address,
                IRInstr::VMExit2 {
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
                //todo if ever native methods get compiled this will need a recompile check as well maybe
                Either::Left(array_into_iter([class_init_restart_point,
                    restart_point_function_address,
                    IRInstr::VMExit2 {
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
                    }]))
            } else {
                let num_args = descriptor.arg_types.len();
                let arg_from_to_offsets = static_arg_offsets(resolver, method_frame_data, &current_instr_data, descriptor, method_id);
                let target_method_layout = resolver.lookup_method_layout(method_id);
                Either::Right(match resolver.lookup_ir_method_id_and_address(method_id) {
                    None => {
                        recompile_conditions.add_condition(NeedsRecompileIf::FunctionCompiled { method_id });
                        let exit_instr = IRInstr::VMExit2 {
                            exit_type: IRVMExitType::CompileFunctionAndRecompileCurrent {
                                current_method_id: method_frame_data.current_method_id,
                                target_method_id: method_id,
                                restart_point_id: restart_point_id_function_address,
                            }
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
                                    ir_method_id,
                                    method_id,
                                    new_frame_size: target_method_layout.full_frame_size(),
                                },
                                current_frame_size: method_frame_data.full_frame_size(),
                            }]))
                    }
                })
            })
        }
    }
}


pub fn invokevirtual<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    method_name: MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: &CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let after_call_restart_point_id = restart_point_generator.new_restart_point();
    let after_call_restart_point = IRInstr::RestartPoint(after_call_restart_point_id);
    let target_class_type = CPDType::Ref(classname_ref_type.clone());
    let target_class_type_id = resolver.get_cpdtype_id(&target_class_type);

    if resolver.lookup_type_inited_initing(&target_class_type).is_none() {
        recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: target_class_type });
        //todo this should never happen?
        return Either::Left(array_into_iter([restart_point,
            IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: target_class_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                },
            }, after_call_restart_point]));
    }

    let num_args = descriptor.arg_types.len();
    /*if let Some(method_id) = resolver.lookup_native_virtual(target_class_type.clone(), method_name, descriptor.clone()) {
        let arg_start_frame_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args as u16);
        let res_pointer_offset = if descriptor.return_type != CompressedParsedDescriptorType::VoidType {
            Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
        } else {
            None
        };
        return Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
            exit_type: IRVMExitType::RunNativeVirtual {
                method_id,
                arg_start_frame_offset,
                res_pointer_offset,
                num_args: num_args as u16,
            }
        }]));
    }*/

    let method_shape_id = resolver.lookup_virtual(target_class_type, method_name, descriptor.clone());
    // todo investigate size of table for invokevirtual without tagging.
    let arg_from_to_offsets = virtual_and_special_arg_offsets(resolver, method_frame_data, &current_instr_data, descriptor);
    return Either::Right(array_into_iter([restart_point,
        IRInstr::VMExit2 {
            exit_type: IRVMExitType::InvokeVirtualResolve {
                object_ref: method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args as u16),
                method_shape_id: resolver.lookup_method_shape(MethodShape { name: method_name, desc: descriptor.clone() }),
                native_restart_point: after_call_restart_point_id,
                native_return_offset: if descriptor.return_type.is_void() {
                    None
                } else {
                    Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                },
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
                ir_method_id: InvokeVirtualResolve::IR_METHOD_ID_RES,
                method_id: InvokeVirtualResolve::METHOD_ID_RES,
                new_frame_size: InvokeVirtualResolve::NEW_FRAME_SIZE_RES,
            },
            current_frame_size: method_frame_data.full_frame_size(),
        },
        after_call_restart_point]));
}

pub fn invoke_interface(
    resolver: &MethodResolver,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    method_name: &MethodName,
    descriptor: &CMethodDescriptor,
    classname_ref_type: &CPRefType,
) -> impl Iterator<Item=IRInstr> {
    let num_args = descriptor.arg_types.len() as u16;

    let target_class_cpdtype = CPDType::Ref(classname_ref_type.clone());
    let cpdtype_id = resolver.get_cpdtype_id(&target_class_cpdtype);
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
                    },
                },after_call_restart_point]))
        }
        Some((target_method_id, is_native)) => {
            Either::Left(/*if is_native {
                let string_pool = &resolver.jvm.string_pool;
                Either::Left(array_into_iter([
                    restart_point,
                    IRInstr::VMExit2 {
                        exit_type: IRVMExitType::RunNativeSpecial {
                            method_id: target_method_id,
                            arg_start_frame_offset: method_frame_data.operand_stack_entry(current_instr_data.current_index, num_args as u16),
                            res_pointer_offset: if CompressedParsedDescriptorType::VoidType == descriptor.return_type {
                                None
                            } else {
                                Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                            },
                            num_args: num_args as u16,
                        }
                    },
                    after_call_restart_point
                ]))
            } else {*/
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
                            }
                        }
                    },
                    IRInstr::IRCall {
                        temp_register_1: Register(1),
                        temp_register_2: Register(2),
                        arg_from_to_offsets: virtual_and_special_arg_offsets(resolver, method_frame_data, &current_instr_data, descriptor),
                        return_value: if descriptor.return_type.is_void() {
                            None
                        } else {
                            Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
                        },
                        target_address: IRCallTarget::Variable {
                            address: InvokeInterfaceResolve::ADDRESS_RES,
                            ir_method_id: InvokeInterfaceResolve::IR_METHOD_ID_RES,
                            method_id: InvokeInterfaceResolve::METHOD_ID_RES,
                            new_frame_size: InvokeInterfaceResolve::NEW_FRAME_SIZE_RES,
                        },
                        current_frame_size: method_frame_data.full_frame_size(),
                    }
                ,after_call_restart_point])
            /*}*/)
        }
    }
}

fn virtual_and_special_arg_offsets<'vm_life>(resolver: &MethodResolver<'vm_life>, method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, descriptor: &CMethodDescriptor) -> Vec<(FramePointerOffset, FramePointerOffset)> {
    // let target_method_layout = resolver.lookup_method_layout(target_method_id);
    let num_args = descriptor.arg_types.len();
    let mut arg_from_to_offsets = vec![];
    let mut local_var_i = 0;
    for (operand_stack_i, arg_type) in descriptor.arg_types.iter().enumerate() {
        let from = method_frame_data.operand_stack_entry(current_instr_data.current_index, (num_args - operand_stack_i - 1) as u16);
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


fn static_arg_offsets<'vm_life>(resolver: &MethodResolver<'vm_life>, method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, descriptor: &CMethodDescriptor, target_method_id: MethodId) -> Vec<(FramePointerOffset, FramePointerOffset)> {
    let target_method_layout = resolver.lookup_method_layout(target_method_id);
    let num_args = descriptor.arg_types.len();
    let mut arg_from_to_offsets = vec![];
    let mut local_var_i = 0;
    //todo this is jank needs to better
    for (operand_stack_i, arg_type) in descriptor.arg_types.iter().enumerate() {
        let from = method_frame_data.operand_stack_entry(current_instr_data.current_index, (num_args - operand_stack_i - 1) as u16);
        let to = target_method_layout.local_var_entry(ByteCodeIndex(0), local_var_i as u16);
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
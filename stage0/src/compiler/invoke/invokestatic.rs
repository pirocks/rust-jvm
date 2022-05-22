use std::ptr::null;
use itertools::Either;
use another_jit_vm::{FramePointerOffset, Register};
use another_jit_vm_ir::changeable_const::ChangeableConstID;
use another_jit_vm_ir::compiler::{IRCallTarget, IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::{IRVMEditAction, IRVMExitType};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::MethodId;
use crate::compiler::{array_into_iter, CurrentInstructionCompilerData, MethodRecompileConditions, NeedsRecompileIf};
use crate::compiler::invoke::static_arg_offsets;
use crate::compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum MethodIDOrChangeableConst {
    MethodId(MethodId),
    ChangeableConst {
        initial_address: u64,
        target_address: ChangeableConstID,
    },
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

    let class_as_cpdtype = classname_ref_type.to_cpdtype();
    match resolver.lookup_static(class_as_cpdtype.clone(), method_name, descriptor.clone()) {
        None => {
            let cpdtype_id = resolver.get_cpdtype_id(class_as_cpdtype);
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: class_as_cpdtype });
            let initial_address = 0;
            let changeable_function_address_const_id = resolver.new_changeable_const64(initial_address);
            let changeable_function_address = MethodIDOrChangeableConst::ChangeableConst { initial_address, target_address: changeable_function_address_const_id };
            let after_exit = static_ir_call_given_methodid(
                resolver,
                restart_point_generator,
                method_frame_data,
                &current_instr_data,
                recompile_conditions,
                descriptor,
                changeable_function_address,
            );
            let init_skipable_exit_id = resolver.new_skipable_exit_id();
            Either::Left(array_into_iter([class_init_restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: cpdtype_id,
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                        java_pc: current_instr_data.current_offset,
                        edit_action: Some(IRVMEditAction::StaticFunctionRecompileFromInitClass {
                            skipable_exit: init_skipable_exit_id,
                            changeable_function_address_const_id,
                            method_name,
                            descriptor: descriptor.clone(),
                            classname_ref_type: *classname_ref_type,
                        }),
                    },
                    skipable_exit_id: Some(init_skipable_exit_id),
                }]).chain(after_exit))
        }
        Some((method_id, _is_native)) => {
            Either::Right(array_into_iter([class_init_restart_point]).chain(
                static_ir_call_given_methodid(
                    resolver,
                    restart_point_generator,
                    method_frame_data,
                    &current_instr_data,
                    recompile_conditions,
                    descriptor,
                    MethodIDOrChangeableConst::MethodId(method_id),
                )))
        }
    }
}


fn static_ir_call_given_methodid<'vm>(
    resolver: &impl MethodResolver<'vm>,
    restart_point_generator: &mut RestartPointGenerator,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    recompile_conditions: &mut MethodRecompileConditions,
    descriptor: &CMethodDescriptor,
    method_id: MethodIDOrChangeableConst,
) -> impl Iterator<Item=IRInstr> {
    let arg_from_to_offsets = static_arg_offsets(method_frame_data, &current_instr_data, descriptor);
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point_function = IRInstr::RestartPoint(restart_point_id);
    match method_id {
        MethodIDOrChangeableConst::MethodId(method_id) => {
            match resolver.lookup_ir_method_id_and_address(method_id) {
                None => {
                    recompile_conditions.add_condition(NeedsRecompileIf::FunctionCompiled { method_id });

                    let function_recompile_skipable_exit_id = resolver.new_skipable_exit_id();
                    let after_exit = static_ir_call_impl(method_frame_data,
                                                         &current_instr_data,
                                                         descriptor,
                                                         IRCallTarget::Constant { address: null(), method_id },
                                                         arg_from_to_offsets);
                    let exit_instr = IRInstr::VMExit2 {
                        exit_type: IRVMExitType::CompileFunctionAndRecompileCurrent {
                            current_method_id: method_frame_data.current_method_id,
                            target_method_id: method_id,
                            restart_point_id,
                            java_pc: current_instr_data.current_offset,
                            edit_action: Some(IRVMEditAction::FunctionRecompileAndCallLocationUpdate { method_id, skipable_exit: function_recompile_skipable_exit_id }),
                        },
                        skipable_exit_id: Some(function_recompile_skipable_exit_id),
                    };
                    //todo have restart point ids for matching same restart points
                    Either::Left(array_into_iter([
                        restart_point_function,
                        exit_instr])
                        .chain(after_exit))
                }
                Some((ir_method_id, address)) => {
                    recompile_conditions.add_condition(NeedsRecompileIf::FunctionRecompiled { function_method_id: method_id, current_ir_method_id: ir_method_id });
                    Either::Right(
                        array_into_iter([restart_point_function]).chain(
                            static_ir_call_impl(
                                method_frame_data,
                                &current_instr_data,
                                descriptor,
                                IRCallTarget::Constant { address, method_id },
                                arg_from_to_offsets,
                            )))
                }
            }
        }
        MethodIDOrChangeableConst::ChangeableConst { initial_address, target_address } => {
            recompile_conditions.add_condition(NeedsRecompileIf::ChangeableConstChanged { from: initial_address, changeable_const: target_address });
            Either::Right(array_into_iter([restart_point_function]).chain(static_ir_call_impl(
                method_frame_data,
                &current_instr_data,
                descriptor,
                IRCallTarget::UnRegistered { changeable_const: target_address },
                arg_from_to_offsets,
            )))
        }
    }
}

fn static_ir_call_impl(
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    descriptor: &CMethodDescriptor,
    target_address: IRCallTarget,
    arg_from_to_offsets: Vec<(FramePointerOffset, FramePointerOffset)>,
) -> impl Iterator<Item=IRInstr> {
    let ir_call = IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        arg_from_to_offsets,
        return_value: if descriptor.return_type.is_void() {
            None
        } else {
            Some(method_frame_data.operand_stack_entry(current_instr_data.next_index, 0))
        },
        //todo needs to call the method target modifier register directly somehow in skippable vmexit for this
        target_address,
        current_frame_size: method_frame_data.full_frame_size(),
    };
    array_into_iter([ir_call])
}

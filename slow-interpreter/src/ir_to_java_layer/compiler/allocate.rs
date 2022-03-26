use std::num::NonZeroU8;
use itertools::Either;

use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use rust_jvm_common::classfile::Atype;
use rust_jvm_common::compressed_classfile::{CPDType};
use rust_jvm_common::compressed_classfile::names::CClassName;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData, MethodRecompileConditions, NeedsRecompileIf};
use crate::jit::MethodResolver;

pub fn new<'vm_life>(resolver: &MethodResolver<'vm_life>,
                     method_frame_data: &JavaCompilerMethodAndFrameData,
                     current_instr_data: &CurrentInstructionCompilerData,
                     restart_point_generator: &mut RestartPointGenerator,
                     recompile_conditions: &mut MethodRecompileConditions,
                     ccn: CClassName) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let cpd_type_id = resolver.get_cpdtype_id(ccn.into());
    match resolver.lookup_type_inited_initing(&(ccn).into()) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: ccn.clone().into() });
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                },
            }])
        }
        Some((loaded_class, loader)) => {
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::AllocateObject {
                    class_type: cpd_type_id,
                    res: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                }
            }])
        }
    }
}


pub fn anewarray<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    elem_type: &CPDType,
) -> impl Iterator<Item=IRInstr> {
    let array_type = CPDType::array(*elem_type/*CPRefType::Array(box elem_type.clone())*/);
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_inited_initing(&array_type) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(array_type);
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: array_type });
            Either::Left(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: cpd_type_id,
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                    },
                }]))
        }
        Some((loaded_class, loader)) => {
            // runtime_class_to_allocated_object_type(&loaded_class,loader,todo!(),todo!());
            //todo allocation should be done in vm exit
            let array_type = resolver.get_cpdtype_id(array_type);
            let arr_len = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
            let arr_res = method_frame_data.operand_stack_entry(current_instr_data.next_index, 0);
            Either::Right(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::AllocateObjectArray_ {
                        array_type,
                        arr_len,
                        arr_res,
                    }
                }]))
        }
    }
}


pub fn newarray<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    elem_type: &Atype,
) -> impl Iterator<Item=IRInstr> {
    anewarray(resolver, method_frame_data, current_instr_data, restart_point_generator, recompile_conditions,&match elem_type {
        Atype::TBoolean => CPDType::BooleanType,
        Atype::TChar => CPDType::CharType,
        Atype::TFloat => CPDType::FloatType,
        Atype::TDouble => CPDType::DoubleType,
        Atype::TByte => CPDType::ByteType,
        Atype::TShort => CPDType::ShortType,
        Atype::TInt => CPDType::IntType,
        Atype::TLong => CPDType::LongType,
    })
}


pub fn multianewarray<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    array_type: CPDType,
    num_arrays: NonZeroU8
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_inited_initing(&array_type) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(array_type);
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: array_type });
            Either::Left(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: cpd_type_id,
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                    },
                }]))
        }
        Some((loaded_class, loader)) => {
            // runtime_class_to_allocated_object_type(&loaded_class,loader,todo!(),todo!());
            //todo allocation should be done in vm exit
            let elem_type = array_type.unwrap_ref_type().recursively_unwrap_array_type();
            let array_elem_type = resolver.get_cpdtype_id(elem_type.to_cpdtype());
            let arr_len_start = method_frame_data.operand_stack_entry(current_instr_data.current_index, (num_arrays.get() - 1) as u16);
            let arr_res = method_frame_data.operand_stack_entry(current_instr_data.next_index, 0);
            Either::Right(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::MultiAllocateObjectArray_ {
                        array_elem_type,
                        num_arrays,
                        arr_len_start,
                        arr_res,
                    }
                }]))
        }
    }
}
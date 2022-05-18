use std::iter;
use std::mem::size_of;

use itertools::{Either};

use another_jit_vm::{Register};
use another_jit_vm_ir::changeable_const::ChangeableConstID;
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator, Size};
use another_jit_vm_ir::vm_exit_abi::{IRVMEditAction, IRVMExitType};
use jvmti_jni_bindings::jlong;
use runtime_class_stuff::{RuntimeClassClass};
use runtime_class_stuff::field_numbers::FieldNumber;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData, MethodRecompileConditions, NeedsRecompileIf};
use crate::compiler::instance_of_and_casting::checkcast_impl;
use crate::compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};

pub const fn field_type_to_register_size(cpd_type: CPDType) -> Size {
    match cpd_type {
        CPDType::BooleanType => Size::Byte,
        CPDType::ByteType => Size::byte(),
        CPDType::ShortType => Size::short(),
        CPDType::CharType => Size::X86Word,
        CPDType::IntType => Size::int(),
        CPDType::LongType => Size::X86QWord,
        CPDType::FloatType => Size::float(),
        CPDType::DoubleType => Size::X86QWord,
        CPDType::VoidType => panic!(),
        CPDType::Class(_) | CPDType::Array { .. } => Size::pointer()
    }
}

pub const fn runtime_type_to_size(rtype: &RuntimeType) -> Size {
    match rtype {
        RuntimeType::IntType => Size::int(),
        RuntimeType::FloatType => Size::float(),
        RuntimeType::DoubleType => Size::double(),
        RuntimeType::LongType => Size::long(),
        RuntimeType::Ref(_) => Size::pointer(),
        RuntimeType::TopType => panic!()
    }
}
/*
struct PutFieldRecomp {
    field_type: CPDType,
    field_number: FieldNumber,
    class_type: CPDType,
}

pub fn calc_putfield_size_needed() -> usize {
    let mut assembler = CodeAssembler::new(64).unwrap();
    let mut labels = HashMap::new();
    let mut restart_points = HashMap::new();
    for ir_instr in put_field_impl() {
        single_ir_to_native(&mut assembler,&ir_instr,&mut labels, &mut restart_points,IRInstructIndex(0),true)
    }
    let res = assembler.assemble(0).unwrap();
    res.len()
}*/

//todo have single changeable/skippable putfield ir instruction, easier to change, give it an id that can be updated as needed
// need mapping loaded class to modified putfield.
// need offsets for where needs to be modified
pub fn putfield<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    mut current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    target_class: CClassName,
    name: FieldName,
    known_target_type: CPDType,
) -> impl Iterator<Item=IRInstr> {
    //todo turn this into a skipable vmexit
    let cpd_type = (target_class).into();
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let cpd_type_id_obj = resolver.get_cpdtype_id(cpd_type);
    let skipable_exit_id = resolver.new_skipable_exit_id();
    match resolver.lookup_type_inited_initing(&cpd_type) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: cpd_type });
            let field_number_const_id = resolver.new_changeable_const64(u64::MAX);
            let after_exit = put_field_impl(resolver,
                                            method_frame_data,
                                            &mut current_instr_data,
                                            cpd_type,
                                            restart_point.clone(),
                                            known_target_type,
                                            field_number_const_id,
                                            known_target_type);
            Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id_obj,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset,
                    edit_action: Some(IRVMEditAction::PutField { field_number_id: field_number_const_id, name }),
                    skipable_exit_id: Some(skipable_exit_id)
                },
                skipable_exit_id: Some(skipable_exit_id),
            }]).chain(after_exit))
        }
        Some((rc, _)) => {
            let (field_number, field_type) = recursively_find_field_number_and_type(rc.unwrap_class_class(), name);
            let field_number = (field_number.0 as usize * size_of::<jlong>()) as u64;

            known_target_type_matches_field_type(known_target_type, field_type);
            let field_number_const_id = resolver.new_changeable_const64(field_number);
            Either::Right(put_field_impl(resolver, method_frame_data, &mut current_instr_data, cpd_type, restart_point, field_type, field_number_const_id, known_target_type))
        }
    }
}

fn put_field_impl<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &mut CurrentInstructionCompilerData,
    cpd_type: CPDType,
    restart_point: IRInstr,
    field_type: CPDType,
    field_number_const_id: ChangeableConstID,
    known_target_type: CPDType,
) -> impl Iterator<Item=IRInstr> {
    let class_ref_register = Register(1);
    let to_put_value = Register(2);
    let offset = Register(3);
    let object_ptr_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 1);
    let to_put_value_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    let field_size = field_type_to_register_size(field_type);
    assert_eq!(field_size, field_type_to_register_size(known_target_type));
    array_into_iter([restart_point]).chain(if field_type.try_unwrap_class_type().is_some() && resolver.debug_checkcast_assertions() {
        Either::Left(checkcast_impl(resolver, current_instr_data, field_type, to_put_value_offset))
    } else {
        Either::Right(iter::empty())
    })
        .chain(if resolver.debug_checkcast_assertions() {
            Either::Left(checkcast_impl(resolver, current_instr_data, cpd_type, object_ptr_offset))
        } else {
            Either::Right(iter::empty())
        })
        .chain(array_into_iter([
            IRInstr::LoadFPRelative {
                from: object_ptr_offset,
                to: class_ref_register,
                size: Size::pointer(),
            },
            IRInstr::NPECheck {
                possibly_null: class_ref_register,
                temp_register: to_put_value,
                npe_exit_type: IRVMExitType::NPE {
                    java_pc: current_instr_data.current_offset
                },
            },
            IRInstr::LoadFPRelative {
                from: to_put_value_offset,
                to: to_put_value,
                size: field_size,
            },
            IRInstr::LoadFPRelative {
                from: object_ptr_offset,
                to: class_ref_register,
                size: Size::pointer(),
            },
            IRInstr::ChangeableConst64bit {
                to: offset,
                const_id: field_number_const_id,
            },
            IRInstr::Add {
                res: class_ref_register,
                a: offset,
                size: Size::pointer(),
            },
            IRInstr::Store {
                to_address: class_ref_register,
                from: to_put_value,
                size: field_size,
            }
        ]))
}

fn known_target_type_matches_field_type(known_target_type: CPDType, field_type: CPDType) {
    match known_target_type.to_runtime_type() {
        None => {
            if let Some(_) = field_type.to_runtime_type() {
                panic!()
            }
        }
        Some(known_target_type) => {
            match field_type.to_runtime_type() {
                None => { panic!() }
                Some(field_type) => {
                    match field_type {
                        RuntimeType::Ref(_) => {
                            match known_target_type {
                                RuntimeType::Ref(_) => {}
                                _ => {
                                    assert_eq!(field_type, known_target_type)
                                }
                            }
                        }
                        _ => {
                            assert_eq!(field_type, known_target_type)
                        }
                    }
                }
            }
        }
    }
}


pub fn getfield<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    mut current_instr_data: CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    target_class: CClassName,
    name: FieldName,
) -> impl Iterator<Item=IRInstr> {
    let cpd_type = (target_class).into();
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let obj_cpd_type_id = resolver.get_cpdtype_id(cpd_type);
    match resolver.lookup_type_inited_initing(&cpd_type) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: cpd_type });
            Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: obj_cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset,
                    edit_action: None,
                    skipable_exit_id: None
                },
                skipable_exit_id: None,
            }]))
        }
        Some((rc, _)) => {
            let (field_number, field_type) = recursively_find_field_number_and_type(rc.unwrap_class_class(), name);
            let class_ref_register = Register(1);
            let to_get_value = Register(2);
            let offset = Register(3);
            let object_ptr_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
            let to_get_value_offset = method_frame_data.operand_stack_entry(current_instr_data.next_index, 0);
            let field_size = field_type_to_register_size(field_type);
            Either::Right(array_into_iter([
                restart_point]).chain(
                if resolver.debug_checkcast_assertions() {
                    Either::Right(checkcast_impl(resolver, &mut current_instr_data, cpd_type, object_ptr_offset))
                } else {
                    Either::Left(iter::empty())
                }
            ).chain(array_into_iter([
                IRInstr::LoadFPRelative {
                    from: object_ptr_offset,
                    to: class_ref_register,
                    size: Size::pointer(),
                },
                IRInstr::NPECheck {
                    possibly_null: class_ref_register,
                    temp_register: to_get_value,
                    npe_exit_type: IRVMExitType::NPE { java_pc: current_instr_data.current_offset },
                },
                IRInstr::LoadFPRelative {
                    from: object_ptr_offset,
                    to: class_ref_register,
                    size: Size::pointer(),
                },
                IRInstr::Const64bit { to: offset, const_: (field_number.0 as usize * size_of::<jlong>()) as u64 },
                IRInstr::Add { res: class_ref_register, a: offset, size: Size::pointer() },
                IRInstr::Load { from_address: class_ref_register, to: to_get_value, size: field_size },
                IRInstr::StoreFPRelative { from: to_get_value, to: to_get_value_offset, size: runtime_type_to_size(&field_type.to_runtime_type().unwrap()) }
            ])).chain(if field_type.try_unwrap_class_type().is_some() && resolver.debug_checkcast_assertions() {
                Either::Left(checkcast_impl(resolver, &mut current_instr_data, field_type, to_get_value_offset))
            } else {
                Either::Right(array_into_iter([]))
            }))
        }
    }
}

pub fn recursively_find_field_number_and_type(rc: &RuntimeClassClass, name: FieldName) -> (FieldNumber, CPDType) {
    match rc.field_numbers.get(&name) {
        Some(x) => *x,
        None => recursively_find_field_number_and_type(rc.parent.as_ref().unwrap().unwrap_class_class(), name),
    }
}
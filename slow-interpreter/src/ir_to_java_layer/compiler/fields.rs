use std::iter;
use std::mem::size_of;

use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use jvmti_jni_bindings::jlong;
use runtime_class_stuff::{RuntimeClassClass};
use runtime_class_stuff::field_numbers::FieldNumber;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData, MethodRecompileConditions, NeedsRecompileIf};
use crate::ir_to_java_layer::compiler::instance_of_and_casting::checkcast_impl;
use crate::jit::MethodResolver;

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

pub fn putfield<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
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
    let cpd_type_id_obj = resolver.get_cpdtype_id(cpd_type);
    match resolver.lookup_type_inited_initing(&cpd_type) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: cpd_type });
            Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id_obj,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset
                }
            }]))
        }
        Some((rc, _)) => {
            let string_pool = &resolver.jvm.string_pool;
            let (field_number, field_type) = recursively_find_field_number_and_type(rc.unwrap_class_class(), name);
            let class_ref_register = Register(1);
            let to_put_value = Register(2);
            let offset = Register(3);
            let object_ptr_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 1);
            let to_put_value_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
            let field_size = field_type_to_register_size(field_type);
            Either::Right(array_into_iter([restart_point]).chain(if field_type.try_unwrap_class_type().is_some() && resolver.debug_checkcast_assertions() {
                Either::Left(checkcast_impl(resolver, method_frame_data, &mut current_instr_data, field_type, to_put_value_offset))
            } else {
                Either::Right(iter::empty())
            })
                .chain(if resolver.debug_checkcast_assertions() {
                    Either::Left(checkcast_impl(resolver, method_frame_data, &mut current_instr_data, cpd_type, object_ptr_offset))
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
                        npe_exit_type: IRVMExitType::NPE { java_pc: current_instr_data.current_offset },
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
                    IRInstr::Const64bit { to: offset, const_: (field_number.0 as usize * size_of::<jlong>()) as u64 },
                    IRInstr::Add { res: class_ref_register, a: offset, size: Size::pointer() },
                    IRInstr::Store { to_address: class_ref_register, from: to_put_value, size: field_size }
                ])))
        }
    }
}


pub fn getfield<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
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
                    java_pc: current_instr_data.current_offset
                }
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
                    Either::Right(checkcast_impl(resolver, method_frame_data, &mut current_instr_data, cpd_type, object_ptr_offset))
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
                Either::Left(checkcast_impl(resolver, method_frame_data, &mut current_instr_data, field_type, to_get_value_offset))
            } else {
                Either::Right(array_into_iter([]))
            }))
        }
    }
}

fn recursively_find_field_number_and_type(rc: &RuntimeClassClass, name: FieldName) -> (FieldNumber, CPDType) {
    match rc.field_numbers.get(&name) {
        Some(x) => *x,
        None => recursively_find_field_number_and_type(rc.parent.as_ref().unwrap().unwrap_class_class(), name),
    }
}
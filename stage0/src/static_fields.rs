use std::ops::Deref;

use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_descriptors::CFieldDescriptor;
use rust_jvm_common::compressed_classfile::field_names::FieldName;

use crate::{array_into_iter, MethodRecompileConditions, NeedsRecompileIf};
use crate::fields::{field_type_to_register_size, runtime_type_to_size};
use compiler_common::{CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData, MethodResolver};

pub fn putstatic<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    target_class: CClassName,
    field_name: FieldName,
    desc: CFieldDescriptor,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_inited_initing(&target_class.into()) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: target_class.into() });
            Either::Left(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: resolver.get_cpdtype_id(target_class.into()),
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                        java_pc: current_instr_data.current_offset,
                    },
                }]))
        }
        Some((rc, _loader)) => {
            let rc = rc.deref();
            let (_, address, field_cpdtype) = resolver.resolve_static_field(rc, field_name);
            let to_put = Register(5);
            let raw_ptr = address.as_ptr();
            assert_eq!(field_cpdtype, desc.0);
            let size = runtime_type_to_size(&field_cpdtype.to_runtime_type().unwrap());
            let static_field_pointer = Register(1);
            assert_ne!(static_field_pointer, to_put);

            Either::Right(array_into_iter([
                restart_point,
                IRInstr::LoadFPRelative {
                    from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                    to: to_put,
                    size: field_type_to_register_size(desc.0).lengthen_runtime_type(),
                },
                IRInstr::Const64bit {
                    to: static_field_pointer,
                    const_: raw_ptr as u64,
                },
                IRInstr::Store {
                    to_address: static_field_pointer,
                    from: to_put,
                    size,
                }
            ]))
        }
    }
}


pub fn getstatic<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    target_class: CClassName,
    field_name: FieldName,
    desc: CFieldDescriptor,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_inited_initing(&target_class.into()) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: target_class.into() });
            Either::Left(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: resolver.get_cpdtype_id(target_class.into()),
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                        java_pc: current_instr_data.current_offset,
                    },
                }]))
        }
        Some((rc, _loader)) => {
            let rc = rc.deref();
            let (_, address, field_cpdtype) = resolver.resolve_static_field(rc, field_name);
            let raw_ptr = address.as_ptr();
            assert_eq!(field_cpdtype, desc.0);
            let size = runtime_type_to_size(&field_cpdtype.to_runtime_type().unwrap()).lengthen_runtime_type();
            let static_field_pointer = Register(1);
            let static_field_value = Register(2);
            Either::Right(array_into_iter([restart_point,
                IRInstr::Const64bit {
                    to: static_field_pointer,
                    const_: raw_ptr as u64,
                },
                IRInstr::Load {
                    to: static_field_value,
                    from_address: static_field_pointer,
                    size,
                },
                IRInstr::StoreFPRelative {
                    from: static_field_value,
                    to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                    size: Size::X86QWord,
                }
            ]))
        }
    }
}

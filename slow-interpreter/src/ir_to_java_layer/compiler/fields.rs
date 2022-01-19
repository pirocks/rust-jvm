use std::mem::size_of;

use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use jvmti_jni_bindings::jlong;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::java::lang::reflect::field::Field;
use crate::jit::MethodResolver;

pub fn putfield(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    target_class: CClassName,
    name: FieldName,
) -> impl Iterator<Item=IRInstr> {
    let cpd_type = (target_class).into();
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_loaded(&cpd_type) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(&cpd_type);
            Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                }
            }]))
        }
        Some((rc, _)) => {
            let (field_number, field_type) = rc.unwrap_class_class().field_numbers.get(&name).unwrap();
            let class_ref_register = Register(1);
            let to_put_value = Register(2);
            let offset = Register(3);
            let object_ptr_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 1);
            let to_put_value_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
            Either::Right(array_into_iter([
                restart_point,
                IRInstr::DebuggerBreakpoint,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::LogFramePointerOffsetValue {
                        value_string: "",
                        value: object_ptr_offset,
                    }
                },
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::LogFramePointerOffsetValue {
                        value_string: "",
                        value: to_put_value_offset,
                    }
                },
                IRInstr::LoadFPRelative {
                    from: object_ptr_offset,
                    to: class_ref_register,
                },
                IRInstr::NPECheck {
                    possibly_null: class_ref_register,
                    temp_register: to_put_value,
                    npe_exit_type: IRVMExitType::NPE,
                },
                IRInstr::LoadFPRelative {
                    from: to_put_value_offset,
                    to: to_put_value,
                },
                IRInstr::LoadFPRelative {
                    from: object_ptr_offset,
                    to: class_ref_register,
                },
                IRInstr::Const64bit { to: offset, const_: (field_number * size_of::<jlong>()) as u64 },
                IRInstr::Add { res: class_ref_register, a: offset },
                IRInstr::Store { to_address: class_ref_register, from: to_put_value }
            ]))
        }
    }
}

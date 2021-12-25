use std::mem::size_of;

use itertools::Either;

use another_jit_vm::Register;
use jvmti_jni_bindings::jlong;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::vm_exit_abi::IRVMExitType;
use crate::java::lang::reflect::field::Field;
use crate::jit::ir::IRInstr;
use crate::jit::MethodResolver;

pub fn putfield(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    target_class: CClassName,
    name: FieldName
) -> impl Iterator<Item=IRInstr> {
    let cpd_type = (target_class).into();
    match resolver.lookup_type_loaded(&cpd_type) {
        None => {
            Either::Left(array_into_iter([IRInstr::VMExit2 { exit_type: IRVMExitType::InitClassAndRecompile { class: cpd_type } }]))
        }
        Some((rc, _)) => {
            let (field_number, field_type) = rc.unwrap_class_class().field_numbers.get(&name).unwrap();
            let class_ref_register = Register(1);
            let to_put_value = Register(2);
            let offset = Register(3);
            Either::Right(array_into_iter([
                IRInstr::LoadFPRelative {
                    from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1),
                    to: class_ref_register,
                },
                IRInstr::NPECheck {
                    possibly_null: class_ref_register,
                    temp_register: to_put_value,
                    npe_exit_type: IRVMExitType::NPE,
                },
                IRInstr::LoadFPRelative {
                    from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                    to: to_put_value,
                },
                IRInstr::Const64bit { to: offset, const_: (field_number * size_of::<jlong>()) as u64 },
                IRInstr::Add { res: class_ref_register, a: offset },
                IRInstr::Store { to_address: class_ref_register, from: to_put_value }
            ]))
        }
    }
}
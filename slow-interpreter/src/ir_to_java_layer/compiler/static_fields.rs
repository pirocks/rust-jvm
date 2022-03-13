use std::sync::Arc;
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;

use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::loading::LoaderName;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData, MethodRecompileConditions, NeedsRecompileIf};
use crate::jit::MethodResolver;
use crate::runtime_class::RuntimeClass;

pub fn putstatic<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    target_class: CClassName,
    name: FieldName,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_inited_initing(&target_class.into()) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: target_class.into() });
            array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: resolver.get_cpdtype_id(&target_class.into()),
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id
                    },
                }])
        }
        Some((rc, loader)) => {
            let field_id = resolver.get_field_id(rc, name);
            array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::PutStatic {
                        field_id,
                        value: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                    }
                }])
        }
    }
}


pub fn getstatic<'vm_life>(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    target_class: CClassName,
    name: FieldName,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_inited_initing(&target_class.into()) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: target_class.into() });
            array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::InitClassAndRecompile {
                        class: resolver.get_cpdtype_id(&target_class.into()),
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id
                    },
                }])
        }
        Some((rc, loader)) => {
            let rc_type = resolver.get_cpdtype_id(&rc.cpdtype());
            array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::GetStatic {
                        field_name: name,
                        rc_type,
                        res_value: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                    }
                }])
        }
    }
}

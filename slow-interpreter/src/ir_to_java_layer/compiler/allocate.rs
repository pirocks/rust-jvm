use itertools::Either;

use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::jit::MethodResolver;
use crate::jit::state::runtime_class_to_allocated_object_type;

pub fn new(resolver: &MethodResolver<'vm_life>,
           method_frame_data: &JavaCompilerMethodAndFrameData,
           current_instr_data: &CurrentInstructionCompilerData,
           restart_point_generator: &mut RestartPointGenerator,
           ccn: CClassName) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_loaded(&(ccn).into()) {
        None => {
            array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: todo!(),
                    this_method_id: todo!(),
                    restart_point_id,
                },
            }])
        }
        Some((loaded_class, loader)) => {
            todo!()
        }
    }
}


pub fn anewarray(
    resolver: &MethodResolver<'vm_life>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    restart_point_generator: &mut RestartPointGenerator,
    elem_type: &CPDType,
) -> impl Iterator<Item=IRInstr> {
    let array_type = CPDType::Ref(CPRefType::Array(box elem_type.clone()));
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_loaded(&array_type) {
        None => {
            let cpd_type_id = resolver.get_cpdtype_id(&array_type);
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
            let array_type = resolver.get_cpdtype_id(&array_type);
            let arr_len = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
            let arr_res = method_frame_data.operand_stack_entry(current_instr_data.next_index, 0);
            Either::Right(array_into_iter([restart_point,
                IRInstr::VMExit2 { exit_type: IRVMExitType::NPE },
                // IRInstr::VMExit2 { exit_type: IRVMExitType::LogWholeFrame {} },
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

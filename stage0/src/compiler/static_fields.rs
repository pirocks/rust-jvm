use std::mem::size_of;

use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, RestartPointGenerator, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use runtime_class_stuff::RuntimeClassClass;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::NativeJavaValue;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData, MethodRecompileConditions, NeedsRecompileIf};
use crate::compiler::fields::{runtime_type_to_size};
use crate::compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};

pub fn putstatic<'vm>(
    resolver: &impl MethodResolver<'vm>,
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
                        class: resolver.get_cpdtype_id(target_class.into()),
                        this_method_id: method_frame_data.current_method_id,
                        restart_point_id,
                        java_pc: current_instr_data.current_offset,
                    },
                }])
        }
        Some((rc, _loader)) => {
            let field_id = resolver.get_field_id(rc, name);
            array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::PutStatic {
                        field_id,
                        value: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                        java_pc: current_instr_data.current_offset,
                    }
                }])
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
            // let rc_type = resolver.get_cpdtype_id(rc.cpdtype());
            // Either::Right(array_into_iter([restart_point,
            //     IRInstr::VMExit2 {
            //         exit_type: IRVMExitType::GetStatic {
            //             field_name,
            //             rc_type,
            //             res_value: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
            //             java_pc: current_instr_data.current_offset,
            //         }
            //     }]))
            let class_class = rc.unwrap_class_class();
            match get_static_from_class_class(method_frame_data, current_instr_data, field_name, restart_point.clone(), class_class){
                None => {
                    panic!()
                }
                Some(res) => {
                    return Either::Right(res)
                }
            }
        }
    }
}

fn get_static_from_class_class<'vm>(
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    field_name: FieldName,
    restart_point: IRInstr,
    class_class: &RuntimeClassClass<'vm>
) -> Option<impl Iterator<Item=IRInstr>> {
    let static_field_number_and_field_type = match class_class.static_field_numbers.get(&field_name) {
        Some(static_field_number_and_field_type) => static_field_number_and_field_type,
        None => {
            if let Some(parent) = class_class.parent.as_ref() {
                if let Some(res) = get_static_from_class_class(method_frame_data,current_instr_data, field_name, restart_point.clone(), parent.unwrap_class_class()){
                    return Some(res)
                }
            }
            for interface_class in class_class.interfaces.iter() {
                if let Some(res) = get_static_from_class_class(method_frame_data,current_instr_data, field_name, restart_point.clone(), interface_class.unwrap_class_class()){
                    return Some(res)
                }
            }
            return  None
        },
    };
    let raw_ptr = class_class.static_vars.raw_ptr();
    let static_number = static_field_number_and_field_type.static_number.0;
    let field_cpdtype = static_field_number_and_field_type.cpdtype;
    let size = runtime_type_to_size(&field_cpdtype.to_runtime_type().unwrap());
    let static_field_pointer = Register(1);
    let static_field_value = Register(2);
    Some(array_into_iter([restart_point,
        IRInstr::Const64bit {
            to: static_field_pointer,
            const_: raw_ptr as u64,
        },
        IRInstr::AddConst {
            res: static_field_pointer,
            a: (static_number as usize * size_of::<NativeJavaValue<'vm>>()) as i32,
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

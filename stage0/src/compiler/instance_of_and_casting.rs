use itertools::Either;

use another_jit_vm::{FramePointerOffset, Register};
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, RestartPointGenerator, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use rust_jvm_common::compressed_classfile::CPDType;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData, MethodRecompileConditions, NeedsRecompileIf};
use crate::compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};

pub fn checkcast<'vm>(
    resolver: &impl MethodResolver<'vm>,
    recompile_conditions: &mut MethodRecompileConditions,
    restart_point_generator: &mut RestartPointGenerator,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    mut current_instr_data: CurrentInstructionCompilerData,
    cpdtype: CPDType,
) -> impl Iterator<Item=IRInstr> {
    let frame_pointer_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    checkcast_impl(resolver, method_frame_data, recompile_conditions, restart_point_generator, &mut current_instr_data, cpdtype, frame_pointer_offset)
}

pub fn checkcast_impl<'vm>(
    resolver: &impl MethodResolver<'vm>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    recompile_conditions: &mut MethodRecompileConditions,
    restart_point_generator: &mut RestartPointGenerator,
    current_instr_data: &mut CurrentInstructionCompilerData,
    cpdtype: CPDType,
    frame_pointer_offset: FramePointerOffset,
) -> impl Iterator<Item=IRInstr> {
    let cpdtype_id = resolver.get_cpdtype_id(cpdtype);
    let mut res = vec![];
    let ptr_register = Register(2);
    let checkcast_succeeds = current_instr_data.compiler_labeler.local_label();
    res.push(IRInstr::LoadFPRelative {
        from: frame_pointer_offset,
        to: ptr_register,
        size: Size::pointer(),
    });
    let zero_register = Register(3);
    res.push(IRInstr::Const64bit { to: zero_register, const_: 0 });
    res.push(IRInstr::BranchEqual {
        a: ptr_register,
        label: checkcast_succeeds,
        size: Size::pointer(),
        b: zero_register
    });


    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_inited_initing(&cpdtype) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: cpdtype.into() });
            res.extend([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpdtype_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset,
                },
            }]);
        }
        Some((rc, _loader_name)) => {
            let exit_type = IRVMExitType::CheckCast {
                value: frame_pointer_offset,
                cpdtype: cpdtype_id,
                java_pc: current_instr_data.current_offset,
            };
            if !rc.view().is_interface() && !rc.cpdtype().is_array() && !rc.cpdtype().is_primitive() {
                if let Some(inheritance_tree_vec) = rc.unwrap_class_class().inheritance_tree_vec.as_ref() {
                    res.extend(array_into_iter([restart_point,
                        IRInstr::Const32bit { to: Register(1), const_: 1 },
                        IRInstr::InstanceOfClass {
                            inheritance_path: inheritance_tree_vec.clone(),
                            object_ref: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                            return_val: Register(1),
                            instance_of_exit: exit_type,
                        },
                        IRInstr::BranchEqualVal {
                            a: Register(1),
                            const_: 1,
                            label: checkcast_succeeds,
                            size: Size::int(),
                        },
                        IRInstr::VMExit2 { exit_type: IRVMExitType::Todo {} }
                    ]));
                }
            } else if rc.view().is_interface() {
                res.extend(array_into_iter([restart_point,
                    IRInstr::InstanceOfInterface {
                        target_interface_id: resolver.lookup_interface_class_id(rc.cpdtype()),
                        object_ref: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                        return_val: Register(1),
                    },
                    IRInstr::BranchEqualVal {
                        a: Register(1),
                        const_: 1,
                        label: checkcast_succeeds,
                        size: Size::int(),
                    },
                    IRInstr::VMExit2 { exit_type: IRVMExitType::Todo {} }
                ]))
            } else {
                res.push(IRInstr::VMExit2 {
                    exit_type
                });
            }
        }
    }

    res.push(IRInstr::Label(IRLabel { name: checkcast_succeeds }));
    res.into_iter()
}

pub fn instanceof<'vm>(
    resolver: &impl MethodResolver<'vm>,
    restart_point_generator: &mut RestartPointGenerator,
    recompile_conditions: &mut MethodRecompileConditions,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    cpdtype: CPDType,
) -> impl Iterator<Item=IRInstr> {
    let cpdtype_id = resolver.get_cpdtype_id(cpdtype);
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    match resolver.lookup_type_inited_initing(&cpdtype) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: cpdtype.into() });
            Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpdtype_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset,
                },
            }]))
        }
        Some((rc, _loader_name)) => {
            let exit_type = IRVMExitType::InstanceOf {
                value: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                res: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                cpdtype: cpdtype_id,
                java_pc: current_instr_data.current_offset,
            };
            if !rc.view().is_interface() && !rc.cpdtype().is_array() && !rc.cpdtype().is_primitive() {
                if let Some(inheritance_tree_vec) = rc.unwrap_class_class().inheritance_tree_vec.as_ref() {
                    return Either::Right(Either::Left(array_into_iter([restart_point,
                        IRInstr::InstanceOfClass {
                            inheritance_path: inheritance_tree_vec.clone(),
                            object_ref: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                            return_val: Register(1),
                            instance_of_exit: exit_type,
                        },
                        IRInstr::StoreFPRelative {
                            from: Register(1),
                            to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                            size: Size::int(),
                        }
                    ])));
                }
            };
            if rc.view().is_interface() {
                return Either::Right(Either::Right(array_into_iter([restart_point,
                    IRInstr::InstanceOfInterface {
                        target_interface_id: resolver.lookup_interface_class_id(rc.cpdtype()),
                        object_ref: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
                        return_val: Register(1),
                    },
                    IRInstr::StoreFPRelative {
                        from: Register(1),
                        to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0),
                        size: Size::int(),
                    }
                ])));
            }
            Either::Left(array_into_iter([restart_point,
                IRInstr::VMExit2 {
                    exit_type
                }
            ]))
        }
    }
}

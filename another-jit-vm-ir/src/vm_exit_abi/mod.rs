use std::collections::HashSet;
use std::num::NonZeroU8;

use iced_x86::code_asm::{CodeAssembler, CodeLabel, qword_ptr, rax, rbp};

use another_jit_vm::Register;
use gc_memory_layout_common::memory_regions::FramePointerOffset;
use rust_jvm_common::{ByteCodeOffset, FieldId, MethodId};
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::method_shape::MethodShapeID;
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::compiler::RestartPointID;
use crate::vm_exit_abi::register_structs::{AllocateObject, AllocateObjectArray, CheckCast, CompileFunctionAndRecompileCurrent, ExitRegisterStruct, GetStatic, InitClassAndRecompile, InstanceOf, InvokeInterfaceResolve, InvokeVirtualResolve, LoadClassAndRecompile, LogFramePointerOffsetValue, LogWholeFrame, MonitorEnter, MonitorExit, MultiAllocateArray, NewClass, NewString, PutStatic, RunNativeSpecial, RunNativeVirtual, RunStaticNative, Throw, TopLevelReturn, TraceInstructionAfter, TraceInstructionBefore};
use crate::vm_exit_abi::runtime_input::RawVMExitType;

pub mod register_structs;
pub mod runtime_input;

#[derive(Debug, Clone)]
pub enum IRVMExitType {
    AllocateObjectArray_ {
        array_type: CPDTypeID,
        arr_len: FramePointerOffset,
        arr_res: FramePointerOffset,
    },
    MultiAllocateObjectArray_ {
        array_elem_type: CPDTypeID,
        num_arrays: NonZeroU8,
        arr_len_start: FramePointerOffset,
        arr_res: FramePointerOffset,
    },
    AllocateObject {
        class_type: CPDTypeID,
        res: FramePointerOffset,
    },
    NewString {
        res: FramePointerOffset,
        compressed_wtf8_buf: CompressedWtf8String,
    },
    NewClass {
        res: FramePointerOffset,
        type_: CPDTypeID,
    },
    NPE,
    LoadClassAndRecompile {
        class: CPDTypeID,
        this_method_id: MethodId,
        restart_point_id: RestartPointID,
    },
    InitClassAndRecompile {
        class: CPDTypeID,
        this_method_id: MethodId,
        restart_point_id: RestartPointID,
    },
    RunStaticNative {
        //todo should I actually use these args?
        method_id: MethodId,
        arg_start_frame_offset: Option<FramePointerOffset>,
        res_pointer_offset: Option<FramePointerOffset>,
        num_args: u16,
    },
    RunNativeVirtual {
        method_id: MethodId,
        arg_start_frame_offset: FramePointerOffset,
        res_pointer_offset: Option<FramePointerOffset>,
        num_args: u16,
    },
    RunNativeSpecial {
        method_id: MethodId,
        arg_start_frame_offset: FramePointerOffset,
        res_pointer_offset: Option<FramePointerOffset>,
        num_args: u16,
    },
    CompileFunctionAndRecompileCurrent {
        current_method_id: MethodId,
        target_method_id: MethodId,
        restart_point_id: RestartPointID,
    },
    TopLevelReturn,
    Todo,
    InstanceOf {
        value: FramePointerOffset,
        res: FramePointerOffset,
        cpdtype: CPDTypeID,
    },
    CheckCast {
        value: FramePointerOffset,
        cpdtype: CPDTypeID,
    },
    PutStatic {
        field_id: FieldId,
        value: FramePointerOffset,
    },
    GetStatic {
        field_name: FieldName,
        rc_type: CPDTypeID,
        res_value: FramePointerOffset,
    },
    LogFramePointerOffsetValue {
        value_string: &'static str,
        value: FramePointerOffset,
    },
    LogWholeFrame {},
    TraceInstructionBefore {
        method_id: MethodId,
        offset: ByteCodeOffset,
    },
    TraceInstructionAfter {
        method_id: MethodId,
        offset: ByteCodeOffset,
    },
    InvokeVirtualResolve {
        object_ref: FramePointerOffset,
        method_shape_id: MethodShapeID,
        native_restart_point: RestartPointID,
        native_return_offset: Option<FramePointerOffset>,
    },
    InvokeInterfaceResolve {
        object_ref: FramePointerOffset,
        target_method_id: MethodId,
        native_restart_point: RestartPointID,
        native_return_offset: Option<FramePointerOffset>,
    },
    MonitorEnter {
        obj: FramePointerOffset
    },
    MonitorExit {
        obj: FramePointerOffset
    },
    Throw {
        to_throw_obj_offset: FramePointerOffset
    },
}

impl IRVMExitType {
    pub fn gen_assembly(&self, assembler: &mut CodeAssembler, after_exit_label: &mut CodeLabel, registers: &HashSet<Register>) {
        match self {
            IRVMExitType::AllocateObjectArray_ { array_type, arr_len, arr_res } => {
                // assembler.int3().unwrap();
                assembler.mov(rax, RawVMExitType::AllocateObjectArray as u64).unwrap();
                assembler.mov(AllocateObjectArray::TYPE.to_native_64(), array_type.0 as u64).unwrap();
                assembler.mov(AllocateObjectArray::LEN.to_native_64(), rbp - arr_len.0).unwrap();
                assembler.lea(AllocateObjectArray::RES_PTR.to_native_64(), rbp - arr_res.0).unwrap();
                assembler.lea(AllocateObjectArray::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::LoadClassAndRecompile { class, this_method_id, restart_point_id } => {
                assembler.mov(rax, RawVMExitType::LoadClassAndRecompile as u64).unwrap();
                assembler.mov(LoadClassAndRecompile::CPDTYPE_ID.to_native_64(), class.0 as u64).unwrap();
                assembler.mov(LoadClassAndRecompile::TO_RECOMPILE.to_native_64(), *this_method_id as u64).unwrap();
                assembler.mov(LoadClassAndRecompile::RESTART_POINT_ID.to_native_64(), restart_point_id.0 as u64).unwrap();
            }
            IRVMExitType::RunStaticNative { method_id, arg_start_frame_offset, res_pointer_offset, num_args } => {
                assert!(registers.contains(&RunStaticNative::METHODID));
                assert!(registers.contains(&RunStaticNative::RESTART_IP));
                assert!(registers.contains(&RunStaticNative::NUM_ARGS));
                assert!(registers.contains(&RunStaticNative::RES));
                assert!(registers.contains(&RunStaticNative::ARG_START));
                assembler.mov(rax, RawVMExitType::RunStaticNative as u64).unwrap();
                assembler.mov(RunStaticNative::METHODID.to_native_64(), *method_id as u64).unwrap();
                assembler.lea(RunStaticNative::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
                match arg_start_frame_offset {
                    None => {
                        assembler.mov(RunStaticNative::ARG_START.to_native_64(), 0u64).unwrap();
                    }
                    Some(arg_start_frame_offset) => {
                        assembler.lea(RunStaticNative::ARG_START.to_native_64(), rbp - arg_start_frame_offset.0).unwrap();
                    }
                }
                assembler.mov(RunStaticNative::NUM_ARGS.to_native_64(), *num_args as u64).unwrap();
                if let Some(res_pointer_offset) = res_pointer_offset {
                    assembler.lea(RunStaticNative::RES.to_native_64(), rbp - res_pointer_offset.0).unwrap();
                }
                // assembler.mov(RunStaticNative::RES.to_native_64(),).unwrap()
            }
            IRVMExitType::TopLevelReturn => {
                assembler.mov(TopLevelReturn::RES.to_native_64(), rax).unwrap();
                assembler.mov(rax, RawVMExitType::TopLevelReturn as u64).unwrap();
            }
            IRVMExitType::CompileFunctionAndRecompileCurrent { current_method_id, target_method_id, restart_point_id, } => {
                assembler.mov(rax, RawVMExitType::CompileFunctionAndRecompileCurrent as u64).unwrap();
                assembler.mov(CompileFunctionAndRecompileCurrent::RESTART_POINT_ID.to_native_64(), restart_point_id.0 as u64).unwrap();
                assembler.mov(CompileFunctionAndRecompileCurrent::CURRENT.to_native_64(), *current_method_id as u64).unwrap();
                assembler.mov(CompileFunctionAndRecompileCurrent::TO_RECOMPILE.to_native_64(), *target_method_id as u64).unwrap();
            }
            IRVMExitType::InitClassAndRecompile { class, this_method_id, restart_point_id, } => {
                assembler.mov(rax, RawVMExitType::InitClassAndRecompile as u64).unwrap();
                assembler.mov(InitClassAndRecompile::CPDTYPE_ID.to_native_64(), class.0 as u64).unwrap();
                assembler.mov(InitClassAndRecompile::TO_RECOMPILE.to_native_64(), *this_method_id as u64).unwrap();
                assembler.mov(InitClassAndRecompile::RESTART_POINT_ID.to_native_64(), restart_point_id.0 as u64).unwrap();
            }
            IRVMExitType::NPE => {
                assembler.mov(rax, RawVMExitType::NPE as u64).unwrap();
            }
            IRVMExitType::PutStatic { field_id, value } => {
                assembler.mov(rax, RawVMExitType::PutStatic as u64).unwrap();
                assembler.lea(PutStatic::VALUE_PTR.to_native_64(), rbp - value.0).unwrap();
                assembler.mov(PutStatic::FIELD_ID.to_native_64(), *field_id as u64).unwrap();
                assembler.lea(PutStatic::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::LogFramePointerOffsetValue { value, value_string: _ } => {
                assembler.mov(rax, RawVMExitType::LogFramePointerOffsetValue as u64).unwrap();
                assembler.mov(LogFramePointerOffsetValue::VALUE.to_native_64(), rbp - value.0).unwrap();
                assembler.lea(LogFramePointerOffsetValue::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::LogWholeFrame { .. } => {
                assembler.mov(rax, RawVMExitType::LogWholeFrame as u64).unwrap();
                assembler.lea(LogWholeFrame::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::TraceInstructionBefore { method_id, offset } => {
                assembler.mov(rax, RawVMExitType::TraceInstructionBefore as u64).unwrap();
                assembler.mov(TraceInstructionBefore::METHOD_ID.to_native_64(), *method_id as u64).unwrap();
                assembler.mov(TraceInstructionBefore::BYTECODE_OFFSET.to_native_64(), offset.0 as u64).unwrap();
                assembler.lea(TraceInstructionBefore::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::TraceInstructionAfter { method_id, offset } => {
                assembler.mov(rax, RawVMExitType::TraceInstructionAfter as u64).unwrap();
                assembler.mov(TraceInstructionAfter::METHOD_ID.to_native_64(), *method_id as u64).unwrap();
                assembler.mov(TraceInstructionAfter::BYTECODE_OFFSET.to_native_64(), offset.0 as u64).unwrap();
                assembler.lea(TraceInstructionAfter::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::AllocateObject { class_type, res } => {
                assembler.mov(rax, RawVMExitType::AllocateObject as u64).unwrap();
                assembler.lea(AllocateObject::RES_PTR.to_native_64(), rbp - res.0).unwrap();
                assembler.mov(AllocateObject::TYPE.to_native_64(), class_type.0 as u64).unwrap();
                assembler.lea(AllocateObject::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap()
            }
            IRVMExitType::NewString { res, compressed_wtf8_buf } => {
                assembler.mov(rax, RawVMExitType::NewString as u64).unwrap();
                assembler.mov(NewString::COMPRESSED_WTF8.to_native_64(), compressed_wtf8_buf.0 as u64).unwrap();
                assembler.lea(NewString::RES.to_native_64(), rbp - res.0).unwrap();
                assembler.lea(NewString::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::NewClass { res, type_ } => {
                assembler.mov(rax, RawVMExitType::NewClass as u64).unwrap();
                assembler.mov(NewClass::CPDTYPE_ID.to_native_64(), type_.0 as u64).unwrap();
                assembler.lea(NewClass::RES.to_native_64(), rbp - res.0).unwrap();
                assembler.lea(NewClass::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::MonitorEnter { obj } => {
                assembler.mov(rax, RawVMExitType::MonitorEnter as u64).unwrap();
                assembler.mov(MonitorEnter::OBJ_ADDR.to_native_64(), rbp - obj.0).unwrap();
                assembler.lea(MonitorEnter::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::MonitorExit { obj } => {
                assembler.mov(rax, RawVMExitType::MonitorExit as u64).unwrap();
                assembler.mov(MonitorExit::OBJ_ADDR.to_native_64(), rbp - obj.0).unwrap();
                assembler.lea(MonitorExit::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::Throw { to_throw_obj_offset } => {
                assembler.mov(rax, RawVMExitType::Throw as u64).unwrap();
                assembler.lea(Throw::EXCEPTION_PTR.to_native_64(), rbp - to_throw_obj_offset.0).unwrap()
            }
            IRVMExitType::GetStatic { field_name, rc_type, res_value } => {
                assembler.mov(rax, RawVMExitType::GetStatic as u64).unwrap();
                assembler.lea(GetStatic::RES_VALUE_PTR.to_native_64(), rbp - res_value.0).unwrap();
                assembler.mov(GetStatic::FIELD_NAME.to_native_64(), field_name.0.id.0 as u64).unwrap();
                assembler.mov(GetStatic::CPDTYPE_ID.to_native_64(), rc_type.0 as u64).unwrap();
                assembler.lea(GetStatic::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::Todo => {
                assembler.mov(rax, RawVMExitType::Todo as u64).unwrap()
            }
            IRVMExitType::InstanceOf { value, res, cpdtype } => {
                assembler.mov(rax, RawVMExitType::InstanceOf as u64).unwrap();
                assembler.lea(InstanceOf::RES_VALUE_PTR.to_native_64(), rbp - res.0).unwrap();
                assembler.lea(InstanceOf::VALUE_PTR.to_native_64(), rbp - value.0).unwrap();
                assembler.mov(InstanceOf::CPDTYPE_ID.to_native_64(), cpdtype.0 as u64).unwrap();
                assembler.lea(InstanceOf::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::CheckCast { value, cpdtype } => {
                assembler.mov(rax, RawVMExitType::CheckCast as u64).unwrap();
                assembler.lea(CheckCast::VALUE_PTR.to_native_64(), rbp - value.0).unwrap();
                assembler.mov(CheckCast::CPDTYPE_ID.to_native_64(), cpdtype.0 as u64).unwrap();
                assembler.lea(CheckCast::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::RunNativeVirtual { method_id, arg_start_frame_offset, res_pointer_offset, num_args: _ } => {
                assembler.mov(rax, RawVMExitType::RunNativeVirtual as u64).unwrap();
                assembler.lea(RunNativeVirtual::ARG_START.to_native_64(), rbp - arg_start_frame_offset.0).unwrap();
                match res_pointer_offset {
                    None => {
                        assembler.xor(RunNativeVirtual::RES_PTR.to_native_64(), RunNativeVirtual::RES_PTR.to_native_64()).unwrap();
                    }
                    Some(res_pointer_offset) => {
                        assembler.lea(RunNativeVirtual::RES_PTR.to_native_64(), rbp - res_pointer_offset.0).unwrap();
                    }
                }
                assembler.mov(RunNativeVirtual::METHODID.to_native_64(), *method_id as u64).unwrap();
                assembler.lea(RunNativeVirtual::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::RunNativeSpecial { method_id, arg_start_frame_offset, res_pointer_offset, num_args: _ } => {
                assembler.mov(rax, RawVMExitType::RunNativeSpecial as u64).unwrap();
                assembler.lea(RunNativeSpecial::ARG_START.to_native_64(), rbp - arg_start_frame_offset.0).unwrap();
                match res_pointer_offset {
                    None => {
                        assembler.xor(RunNativeSpecial::RES_PTR.to_native_64(), RunNativeSpecial::RES_PTR.to_native_64()).unwrap();
                    }
                    Some(res_pointer_offset) => {
                        assembler.lea(RunNativeSpecial::RES_PTR.to_native_64(), rbp - res_pointer_offset.0).unwrap();
                    }
                }
                assembler.mov(RunNativeSpecial::METHODID.to_native_64(), *method_id as u64).unwrap();
                assembler.lea(RunNativeSpecial::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::InvokeInterfaceResolve { object_ref, target_method_id, native_restart_point, native_return_offset } => {
                assembler.mov(rax, RawVMExitType::InvokeInterfaceResolve as u64).unwrap();
                assembler.lea(InvokeInterfaceResolve::OBJECT_REF.to_native_64(), rbp - object_ref.0).unwrap();
                assembler.mov(InvokeInterfaceResolve::TARGET_METHOD_ID.to_native_64(), *target_method_id as u64).unwrap();
                assembler.lea(InvokeInterfaceResolve::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
                assembler.mov(InvokeInterfaceResolve::NATIVE_RESTART_POINT.to_native_64(), native_restart_point.0).unwrap();
                match native_return_offset {
                    None => {
                        assembler.mov(InvokeInterfaceResolve::NATIVE_RETURN_PTR.to_native_64(), u64::MAX).unwrap();
                    }
                    Some(native_return_offset) => {
                        assembler.lea(InvokeInterfaceResolve::NATIVE_RETURN_PTR.to_native_64(), rbp - native_return_offset.0).unwrap();
                    }
                }
            }
            IRVMExitType::InvokeVirtualResolve { object_ref, method_shape_id, native_restart_point, native_return_offset } => {
                assembler.mov(rax, RawVMExitType::InvokeVirtualResolve as u64).unwrap();
                assembler.lea(InvokeVirtualResolve::OBJECT_REF_PTR.to_native_64(), rbp - object_ref.0).unwrap();
                assembler.lea(InvokeVirtualResolve::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
                assembler.mov(InvokeVirtualResolve::METHOD_SHAPE_ID.to_native_64(), method_shape_id.0).unwrap();
                assembler.mov(InvokeVirtualResolve::NATIVE_RESTART_POINT.to_native_64(), native_restart_point.0).unwrap();
                match native_return_offset {
                    None => {
                        assembler.mov(InvokeVirtualResolve::NATIVE_RETURN_PTR.to_native_64(), u64::MAX).unwrap();
                    }
                    Some(native_return_offset) => {
                        assembler.lea(InvokeVirtualResolve::NATIVE_RETURN_PTR.to_native_64(), rbp - native_return_offset.0).unwrap();
                    }
                }
            }
            IRVMExitType::MultiAllocateObjectArray_ { array_elem_type, num_arrays, arr_len_start, arr_res } => {
                assembler.mov(rax, RawVMExitType::MultiAllocateObjectArray as u64).unwrap();
                assembler.lea(MultiAllocateArray::LEN_START.to_native_64(), rbp - arr_len_start.0).unwrap();
                assembler.lea(MultiAllocateArray::RES_PTR.to_native_64(), rbp - arr_res.0).unwrap();
                assembler.mov(MultiAllocateArray::ELEM_TYPE.to_native_64(), array_elem_type.0 as u64).unwrap();
                assembler.mov(MultiAllocateArray::NUM_ARRAYS.to_native_64(), num_arrays.get() as u64).unwrap();
                assembler.lea(MultiAllocateArray::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
        }
    }

    pub fn to_register_struct(&self) -> impl ExitRegisterStruct {
        match self {
            IRVMExitType::AllocateObjectArray_ { .. } => {
                todo!()
            }
            IRVMExitType::MultiAllocateObjectArray_ { .. } => {
                todo!()
            }
            IRVMExitType::AllocateObject { .. } => {
                todo!()
            }
            IRVMExitType::NewString { .. } => {
                todo!()
            }
            IRVMExitType::NewClass { .. } => {
                todo!()
            }
            IRVMExitType::NPE => {
                todo!()
            }
            IRVMExitType::LoadClassAndRecompile { .. } => {
                todo!()
            }
            IRVMExitType::InitClassAndRecompile { .. } => {
                todo!()
            }
            IRVMExitType::RunStaticNative { .. } => {
                todo!()
            }
            IRVMExitType::RunNativeVirtual { .. } => {
                todo!()
            }
            IRVMExitType::RunNativeSpecial { .. } => {
                todo!()
            }
            IRVMExitType::CompileFunctionAndRecompileCurrent { .. } => {
                todo!()
            }
            IRVMExitType::TopLevelReturn => {
                todo!()
            }
            IRVMExitType::Todo => {
                todo!()
            }
            IRVMExitType::InstanceOf { .. } => {
                todo!()
            }
            IRVMExitType::CheckCast { .. } => {
                todo!()
            }
            IRVMExitType::PutStatic { .. } => {
                todo!()
            }
            IRVMExitType::GetStatic { .. } => {
                todo!()
            }
            IRVMExitType::LogFramePointerOffsetValue { .. } => {
                todo!()
            }
            IRVMExitType::LogWholeFrame { .. } => {
                todo!()
            }
            IRVMExitType::TraceInstructionBefore { .. } => {
                todo!()
            }
            IRVMExitType::TraceInstructionAfter { .. } => {
                todo!()
            }
            IRVMExitType::InvokeVirtualResolve { .. } => {
                InvokeVirtualResolve{}
            }
            IRVMExitType::InvokeInterfaceResolve { .. } => {
                todo!()
            }
            IRVMExitType::MonitorEnter { .. } => {
                todo!()
            }
            IRVMExitType::MonitorExit { .. } => {
                todo!()
            }
            IRVMExitType::Throw { .. } => {
                todo!()
            }
        }
    }

    pub fn registers_to_save(&self) -> HashSet<Register> {
        let res = match self {
            IRVMExitType::AllocateObjectArray_ { .. } => {
                AllocateObjectArray::all_registers()
            }
            IRVMExitType::MultiAllocateObjectArray_ { .. } => {
                MultiAllocateArray::all_registers()
            }
            IRVMExitType::AllocateObject { .. } => {
                AllocateObject::all_registers()
            }
            IRVMExitType::NewString { .. } => {
                NewString::all_registers()
            }
            IRVMExitType::NewClass { .. } => {
                NewClass::all_registers()
            }
            IRVMExitType::NPE => {
                HashSet::from([Register(0)])
            }
            IRVMExitType::LoadClassAndRecompile { .. } => {
                HashSet::from([Register(0), LoadClassAndRecompile::TO_RECOMPILE, LoadClassAndRecompile::CPDTYPE_ID, LoadClassAndRecompile::RESTART_POINT_ID])
            }
            IRVMExitType::InitClassAndRecompile { .. } => {
                HashSet::from([Register(0), InitClassAndRecompile::TO_RECOMPILE, InitClassAndRecompile::CPDTYPE_ID, InitClassAndRecompile::RESTART_POINT_ID])
            }
            IRVMExitType::RunStaticNative { .. } => {
                HashSet::from([Register(0), RunStaticNative::RES, RunStaticNative::RESTART_IP, RunStaticNative::ARG_START, RunStaticNative::METHODID, RunStaticNative::NUM_ARGS])
            }
            IRVMExitType::RunNativeVirtual { .. } => {
                todo!()
            }
            IRVMExitType::RunNativeSpecial { .. } => {
                RunNativeSpecial::all_registers()
            }
            IRVMExitType::CompileFunctionAndRecompileCurrent { .. } => {
                CompileFunctionAndRecompileCurrent::all_registers()
            }
            IRVMExitType::TopLevelReturn => {
                HashSet::from([Register(0), TopLevelReturn::RES])
            }
            IRVMExitType::Todo => {
                HashSet::from([Register(0)])
            }
            IRVMExitType::InstanceOf { .. } => {
                InstanceOf::all_registers()
            }
            IRVMExitType::CheckCast { .. } => {
                CheckCast::all_registers()
            }
            IRVMExitType::PutStatic { .. } => {
                PutStatic::all_registers()
            }
            IRVMExitType::GetStatic { .. } => {
                GetStatic::all_registers()
            }
            IRVMExitType::LogFramePointerOffsetValue { .. } => {
                todo!()
            }
            IRVMExitType::LogWholeFrame { .. } => {
                todo!()
            }
            IRVMExitType::TraceInstructionBefore { .. } => {
                todo!()
            }
            IRVMExitType::TraceInstructionAfter { .. } => {
                todo!()
            }
            IRVMExitType::InvokeVirtualResolve { .. } => {
                InvokeVirtualResolve::all_registers()
            }
            IRVMExitType::InvokeInterfaceResolve { .. } => {
                InvokeInterfaceResolve::all_registers()
            }
            IRVMExitType::MonitorEnter { .. } => {
                MonitorEnter::all_registers()
            }
            IRVMExitType::MonitorExit { .. } => {
                MonitorExit::all_registers()
            }
            IRVMExitType::Throw { .. } => {
                Throw::all_registers()
            }
        };
        assert!(res.contains(&Register(0)));
        res
    }
}

pub enum VMExitTypeWithArgs {
    LoadClassAndRecompile(LoadClassAndRecompile),
    RunStaticNative(RunStaticNative),
    TopLevelReturn,
}



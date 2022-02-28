use std::ffi::c_void;
use std::ptr::{NonNull, null_mut};
use std::sync::RwLock;

use bimap::BiHashMap;
use iced_x86::code_asm::{CodeAssembler, CodeLabel, qword_ptr, rax, rbp};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use add_only_static_vec::AddOnlyId;

use another_jit_vm::Register;
use another_jit_vm::saved_registers_utils::SavedRegistersWithIP;
use gc_memory_layout_common::FramePointerOffset;
use rust_jvm_common::{ByteCodeOffset, FieldId, MethodId};
use rust_jvm_common::compressed_classfile::{CompressedClassfileString, CPDType};
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::method_shape::MethodShapeID;
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::compiler::RestartPointID;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct LoadClassAndRecompileStaticArgsID(usize);

pub struct VMExitStaticArgs {
    load_class_and_recompile: RwLock<BiHashMap<LoadClassAndRecompileStaticArgsID, LoadClassAndRecompileStaticArgs>>,
}

impl VMExitStaticArgs {
    pub fn new() -> VMExitStaticArgs {
        Self {
            load_class_and_recompile: Default::default()
        }
    }

    pub fn register_new_load_class_and_recompile(&self, static_args: LoadClassAndRecompileStaticArgs) -> LoadClassAndRecompileStaticArgsID {
        let mut guard = self.load_class_and_recompile.write().unwrap();
        return match guard.get_by_right(&static_args) {
            None => {
                let len = guard.len();
                let new_id = LoadClassAndRecompileStaticArgsID(len);
                guard.insert(new_id, static_args);
                new_id
            }
            Some(id) => {
                *id
            }
        };
    }
}

pub struct AllocateObjectArray;

impl AllocateObjectArray {
    pub const LEN: Register = Register(2);
    pub const TYPE: Register = Register(3);
    pub const RES_PTR: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
}

pub struct AllocateObject;

impl AllocateObject {
    pub const TYPE: Register = Register(3);
    pub const RES_PTR: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
}


pub struct RunStaticNative;

impl RunStaticNative {
    pub const RES: Register = Register(1);
    pub const ARG_START: Register = Register(2);
    //pointer to first(highest address) arg
    pub const NUM_ARGS: Register = Register(3);
    //num args
    pub const METHODID: Register = Register(4);
    //methodid
    pub const RESTART_IP: Register = Register(5);
}


pub struct RunNativeVirtual;

impl RunNativeVirtual {
    pub const RES_PTR: Register = Register(2);
    pub const ARG_START: Register = Register(3);
    pub const METHODID: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
}


pub struct RunNativeSpecial;

impl RunNativeSpecial {
    pub const RES_PTR: Register = Register(2);
    pub const ARG_START: Register = Register(3);
    pub const METHODID: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
}

pub struct TopLevelReturn;

impl TopLevelReturn {
    pub const RES: Register = Register(2);
}

pub struct PutStatic;

impl PutStatic {
    pub const FIELD_ID: Register = Register(2);
    pub const VALUE_PTR: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
}


pub struct GetStatic;

impl GetStatic {
    pub const FIELD_NAME: Register = Register(2);
    pub const RES_VALUE_PTR: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const CPDTYPE_ID: Register = Register(5);
}

pub struct Throw;

impl Throw {
    pub const EXCEPTION_PTR: Register = Register(2);
}

pub struct InitClassAndRecompile;

impl InitClassAndRecompile {
    pub const CPDTYPE_ID: Register = Register(2);
    pub const TO_RECOMPILE: Register = Register(3);
    pub const RESTART_POINT_ID: Register = Register(4);
}

pub struct CompileFunctionAndRecompileCurrent;

impl CompileFunctionAndRecompileCurrent {
    pub const CURRENT: Register = Register(2);
    pub const TO_RECOMPILE: Register = Register(3);
    pub const RESTART_POINT_ID: Register = Register(4);
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct LoadClassAndRecompileStaticArgs {
    class_type: CPDType,
}

pub struct LoadClassAndRecompile;

impl LoadClassAndRecompile {
    pub const CPDTYPE_ID: Register = Register(2);
    pub const TO_RECOMPILE: Register = Register(3);
    pub const RESTART_POINT_ID: Register = Register(4);
}

pub struct LogFramePointerOffsetValue;

impl LogFramePointerOffsetValue {
    pub const VALUE: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    // pub const STRING_MESSAGE: Register = Register(4);
}

pub struct LogWholeFrame;

impl LogWholeFrame {
    pub const RESTART_IP: Register = Register(2);
}

pub struct TraceInstructionBefore;

impl TraceInstructionBefore {
    pub const METHOD_ID: Register = Register(2);
    pub const BYTECODE_OFFSET: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
}

pub struct TraceInstructionAfter;

impl TraceInstructionAfter {
    pub const METHOD_ID: Register = Register(2);
    pub const BYTECODE_OFFSET: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
}

pub struct BeforeReturn;

impl BeforeReturn {
    pub const FRAME_SIZE: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
}

pub struct NewString;

impl NewString {
    pub const COMPRESSED_WTF8: Register = Register(2);
    pub const RES: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
}

pub struct NewClass;

impl NewClass {
    pub const CPDTYPE_ID: Register = Register(2);
    pub const RES: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
}

pub struct InstanceOf;

impl InstanceOf {
    pub const VALUE_PTR: Register = Register(2);
    pub const RES_VALUE_PTR: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const CPDTYPE_ID: Register = Register(5);
}

pub struct CheckCast;

impl CheckCast {
    pub const VALUE_PTR: Register = Register(2);
    pub const RESTART_IP: Register = Register(4);
    pub const CPDTYPE_ID: Register = Register(5);
}


pub struct InvokeVirtualResolve;

impl InvokeVirtualResolve {
    pub const OBJECT_REF_PTR: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    pub const ADDRESS_RES: Register = Register(4);
    pub const IR_METHOD_ID_RES: Register = Register(5);
    pub const METHOD_ID_RES: Register = Register(6);
    pub const NEW_FRAME_SIZE_RES: Register = Register(7);
    pub const METHOD_SHAPE_ID: Register = Register(8);
    pub const NATIVE_RESTART_POINT: Register = Register(9);
    pub const NATIVE_RETURN_PTR: Register = Register(5);
}

pub struct InvokeInterfaceResolve;

impl InvokeInterfaceResolve {
    pub const OBJECT_REF: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    pub const ADDRESS_RES: Register = Register(4);
    pub const IR_METHOD_ID_RES: Register = Register(5);
    pub const METHOD_ID_RES: Register = Register(6);
    pub const NEW_FRAME_SIZE_RES: Register = Register(7);
    pub const TARGET_METHOD_ID: Register = Register(8);
}

pub struct MonitorEnter;

impl MonitorEnter {
    pub const OBJ_ADDR: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
}

pub struct MonitorExit;

impl MonitorExit {
    pub const OBJ_ADDR: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
}

#[derive(Debug, Clone)]
pub enum IRVMExitType {
    AllocateObjectArray_ {
        array_type: CPDTypeID,
        arr_len: FramePointerOffset,
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
        native_return_offset: FramePointerOffset,
    },
    InvokeInterfaceResolve {
        object_ref: FramePointerOffset,
        target_method_id: MethodId,
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
    pub fn gen_assembly(&self, assembler: &mut CodeAssembler, after_exit_label: &mut CodeLabel, registers: Vec<Register>) {
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
            IRVMExitType::InvokeVirtualResolve { object_ref, method_shape_id, native_restart_point, native_return_offset } => {
                assembler.mov(rax, RawVMExitType::InvokeVirtualResolve as u64).unwrap();
                assembler.lea(InvokeVirtualResolve::OBJECT_REF_PTR.to_native_64(), rbp - object_ref.0).unwrap();
                assembler.lea(InvokeVirtualResolve::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
                assembler.mov(InvokeVirtualResolve::METHOD_SHAPE_ID.to_native_64(), method_shape_id.0).unwrap();
                assembler.mov(InvokeVirtualResolve::NATIVE_RESTART_POINT.to_native_64(), native_restart_point.0).unwrap();
                assembler.lea(InvokeVirtualResolve::NATIVE_RETURN_PTR.to_native_64(), rbp - native_return_offset.0).unwrap();
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
            IRVMExitType::InvokeInterfaceResolve { object_ref, target_method_id } => {
                assembler.mov(rax, RawVMExitType::InvokeInterfaceResolve as u64).unwrap();
                assembler.lea(InvokeInterfaceResolve::OBJECT_REF.to_native_64(), rbp - object_ref.0).unwrap();
                assembler.mov(InvokeInterfaceResolve::TARGET_METHOD_ID.to_native_64(), *target_method_id as u64).unwrap();
                assembler.lea(InvokeInterfaceResolve::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
        }
    }
}

pub enum VMExitTypeWithArgs {
    LoadClassAndRecompile(LoadClassAndRecompile),
    RunStaticNative(RunStaticNative),
    TopLevelReturn,
}


#[derive(FromPrimitive)]
#[repr(u64)]
pub enum RawVMExitType {
    AllocateObjectArray = 1,
    AllocateObject,
    LoadClassAndRecompile,
    InitClassAndRecompile,
    RunStaticNative,
    TopLevelReturn,
    CompileFunctionAndRecompileCurrent,
    NPE,
    PutStatic,
    GetStatic,
    LogFramePointerOffsetValue,
    LogWholeFrame,
    TraceInstructionBefore,
    TraceInstructionAfter,
    NewString,
    NewClass,
    InvokeVirtualResolve,
    InvokeInterfaceResolve,
    MonitorEnter,
    MonitorExit,
    Throw,
    InstanceOf,
    CheckCast,
    RunNativeVirtual,
    RunNativeSpecial,
    Todo,
}


#[derive(Debug)]
pub enum RuntimeVMExitInput {
    AllocateObjectArray {
        type_: CPDTypeID,
        len: i32,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
    },
    AllocateObject {
        type_: CPDTypeID,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
    },
    AllocatePrimitiveArray {
        type_: CPDType,
        len: u64,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
    },
    LoadClassAndRecompile {
        class_type: CPDType,
        // todo static args?
        restart_point: RestartPointID,
    },
    InitClassAndRecompile {
        class_type: CPDTypeID,
        current_method_id: MethodId,
        restart_point: RestartPointID,
        rbp: *const c_void,
    },
    RunStaticNative {
        method_id: MethodId,
        arg_start: *mut c_void,
        num_args: u16,
        res_ptr: *mut c_void,
        return_to_ptr: *mut c_void,
    },
    NPE {},
    TopLevelReturn {
        return_value: u64
    },
    CompileFunctionAndRecompileCurrent {
        current_method_id: MethodId,
        to_recompile: MethodId,
        restart_point: RestartPointID,
    },
    PutStatic {
        value_ptr: *mut c_void,
        field_id: FieldId,
        return_to_ptr: *const c_void,
    },
    Throw {
        exception_obj_ptr: *const c_void
    },
    GetStatic {
        res_value_ptr: *mut c_void,
        field_name: FieldName,
        cpdtype_id: CPDTypeID,
        return_to_ptr: *const c_void,
    },
    LogFramePointerOffsetValue {
        value: u64,
        return_to_ptr: *const c_void,
        // str_message: &'static str
    },
    LogWholeFrame {
        return_to_ptr: *const c_void,
    },
    TraceInstructionBefore {
        method_id: MethodId,
        bytecode_offset: ByteCodeOffset,
        return_to_ptr: *const c_void,
    },
    TraceInstructionAfter {
        method_id: MethodId,
        bytecode_offset: ByteCodeOffset,
        return_to_ptr: *const c_void,
    },
    NewString {
        return_to_ptr: *const c_void,
        res: *mut c_void,
        compressed_wtf8: CompressedWtf8String,
    },
    NewClass {
        return_to_ptr: *const c_void,
        res: *mut c_void,
        type_: CPDTypeID,
    },
    InvokeVirtualResolve {
        return_to_ptr: *const c_void,
        object_ref_ptr: *const c_void,
        method_shape_id: MethodShapeID,
        native_method_restart_point: RestartPointID,
        native_method_res: *mut c_void
    },
    InvokeInterfaceResolve {
        return_to_ptr: *const c_void,
        object_ref: *const c_void,
        target_method_id: MethodId,
    },
    MonitorEnter {
        obj_ptr: *const c_void,
        return_to_ptr: *const c_void,
    },
    MonitorExit {
        obj_ptr: *const c_void,
        return_to_ptr: *const c_void,
    },
    InstanceOf {
        res: *mut c_void,
        value: *const c_void,
        cpdtype_id: CPDTypeID,
        return_to_ptr: *const c_void,
    },
    CheckCast {
        value: *const c_void,
        cpdtype_id: CPDTypeID,
        return_to_ptr: *const c_void,
    },
    RunNativeVirtual {
        res_ptr: *mut c_void,
        arg_start: *const c_void,
        method_id: MethodId,
        return_to_ptr: *const c_void,
    },
    RunNativeSpecial {
        res_ptr: *mut c_void,
        arg_start: *const c_void,
        method_id: MethodId,
        return_to_ptr: *const c_void,
    },
}

impl RuntimeVMExitInput {
    pub fn from_register_state(register_state: &SavedRegistersWithIP) -> Self {
        let raw_vm_exit_type: RawVMExitType = RawVMExitType::from_u64(register_state.saved_registers_without_ip.rax as u64).unwrap();
        match raw_vm_exit_type {
            RawVMExitType::AllocateObjectArray => {
                RuntimeVMExitInput::AllocateObjectArray {
                    type_: CPDTypeID(register_state.saved_registers_without_ip.get_register(AllocateObjectArray::TYPE) as u32),
                    len: register_state.saved_registers_without_ip.get_register(AllocateObjectArray::LEN) as i32,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(AllocateObjectArray::RESTART_IP) as *const c_void,
                    res_address: register_state.saved_registers_without_ip.get_register(AllocateObjectArray::RES_PTR) as *mut NonNull<c_void>,
                }
            }
            RawVMExitType::LoadClassAndRecompile => todo!(),
            RawVMExitType::RunStaticNative => {
                RuntimeVMExitInput::RunStaticNative {
                    method_id: register_state.saved_registers_without_ip.get_register(RunStaticNative::METHODID) as MethodId,
                    arg_start: register_state.saved_registers_without_ip.get_register(RunStaticNative::ARG_START) as *mut c_void,
                    num_args: register_state.saved_registers_without_ip.get_register(RunStaticNative::NUM_ARGS) as u16,
                    res_ptr: register_state.saved_registers_without_ip.get_register(RunStaticNative::RES) as *mut c_void,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunStaticNative::RESTART_IP) as *mut c_void,
                }
            }
            RawVMExitType::TopLevelReturn => {
                RuntimeVMExitInput::TopLevelReturn {
                    return_value: register_state.saved_registers_without_ip.get_register(TopLevelReturn::RES)
                }
            }
            RawVMExitType::CompileFunctionAndRecompileCurrent => {
                RuntimeVMExitInput::CompileFunctionAndRecompileCurrent {
                    current_method_id: register_state.saved_registers_without_ip.get_register(CompileFunctionAndRecompileCurrent::CURRENT) as MethodId,
                    to_recompile: register_state.saved_registers_without_ip.get_register(CompileFunctionAndRecompileCurrent::TO_RECOMPILE) as MethodId,
                    restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(CompileFunctionAndRecompileCurrent::RESTART_POINT_ID)),
                }
            }
            RawVMExitType::NPE => {
                RuntimeVMExitInput::NPE {}
            }
            RawVMExitType::PutStatic => {
                RuntimeVMExitInput::PutStatic {
                    value_ptr: register_state.saved_registers_without_ip.get_register(PutStatic::VALUE_PTR) as *mut c_void,
                    field_id: register_state.saved_registers_without_ip.get_register(PutStatic::FIELD_ID) as FieldId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(PutStatic::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::InitClassAndRecompile => {
                RuntimeVMExitInput::InitClassAndRecompile {
                    class_type: CPDTypeID(register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::CPDTYPE_ID) as u32),
                    current_method_id: register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::TO_RECOMPILE) as MethodId,
                    restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::RESTART_POINT_ID)),
                    rbp: register_state.saved_registers_without_ip.rbp,
                }
            }
            RawVMExitType::LogFramePointerOffsetValue => {
                RuntimeVMExitInput::LogFramePointerOffsetValue {
                    value: register_state.saved_registers_without_ip.get_register(LogFramePointerOffsetValue::VALUE) as u64,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(LogFramePointerOffsetValue::RESTART_IP) as *const c_void,
                    // str_message: register_state.saved_registers_without_ip.get_register(LogFramePointerOffsetValue::STRING_MESSAGE)
                }
            }
            RawVMExitType::LogWholeFrame => {
                RuntimeVMExitInput::LogWholeFrame {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(LogWholeFrame::RESTART_IP) as *const c_void
                }
            }
            RawVMExitType::TraceInstructionBefore => {
                RuntimeVMExitInput::TraceInstructionBefore {
                    method_id: register_state.saved_registers_without_ip.get_register(TraceInstructionBefore::METHOD_ID) as MethodId,
                    bytecode_offset: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(TraceInstructionBefore::BYTECODE_OFFSET) as u16),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(TraceInstructionBefore::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::TraceInstructionAfter => {
                RuntimeVMExitInput::TraceInstructionAfter {
                    method_id: register_state.saved_registers_without_ip.get_register(TraceInstructionAfter::METHOD_ID) as MethodId,
                    bytecode_offset: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(TraceInstructionAfter::BYTECODE_OFFSET) as u16),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(TraceInstructionAfter::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::AllocateObject => {
                RuntimeVMExitInput::AllocateObject {
                    type_: CPDTypeID(register_state.saved_registers_without_ip.get_register(AllocateObject::TYPE) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(AllocateObject::RESTART_IP) as *const c_void,
                    res_address: register_state.saved_registers_without_ip.get_register(AllocateObject::RES_PTR) as *mut NonNull<c_void>,
                }
            }
            RawVMExitType::NewString => {
                RuntimeVMExitInput::NewString {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(NewString::RESTART_IP) as *const c_void,
                    res: register_state.saved_registers_without_ip.get_register(NewString::RES) as *mut c_void,
                    compressed_wtf8: CompressedWtf8String(register_state.saved_registers_without_ip.get_register(NewString::COMPRESSED_WTF8) as usize),
                }
            }
            RawVMExitType::NewClass => {
                RuntimeVMExitInput::NewClass {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(NewClass::RESTART_IP) as *const c_void,
                    res: register_state.saved_registers_without_ip.get_register(NewClass::RES) as *mut c_void,
                    type_: CPDTypeID(register_state.saved_registers_without_ip.get_register(NewClass::CPDTYPE_ID) as u32),
                }
            }
            RawVMExitType::InvokeVirtualResolve => {
                let native_method_res = register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::NATIVE_RETURN_PTR) as *mut c_void;
                assert_ne!(native_method_res, null_mut());
                RuntimeVMExitInput::InvokeVirtualResolve {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::RESTART_IP) as *const c_void,
                    method_shape_id: MethodShapeID(register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::METHOD_SHAPE_ID) as u64),
                    object_ref_ptr: register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::OBJECT_REF_PTR) as *const c_void,
                    native_method_restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::NATIVE_RESTART_POINT)),
                    native_method_res
                }
            }
            RawVMExitType::MonitorEnter => {
                RuntimeVMExitInput::MonitorEnter {
                    obj_ptr: register_state.saved_registers_without_ip.get_register(MonitorEnter::OBJ_ADDR) as *const c_void,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(MonitorEnter::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::MonitorExit => {
                RuntimeVMExitInput::MonitorExit {
                    obj_ptr: register_state.saved_registers_without_ip.get_register(MonitorExit::OBJ_ADDR) as *const c_void,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(MonitorExit::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::Throw => {
                RuntimeVMExitInput::Throw {
                    exception_obj_ptr: register_state.saved_registers_without_ip.get_register(Throw::EXCEPTION_PTR) as *const c_void
                }
            }
            RawVMExitType::GetStatic => {
                RuntimeVMExitInput::GetStatic {
                    res_value_ptr: register_state.saved_registers_without_ip.get_register(GetStatic::RES_VALUE_PTR) as *mut c_void,
                    field_name: FieldName(CompressedClassfileString{ id: AddOnlyId(register_state.saved_registers_without_ip.get_register(GetStatic::FIELD_NAME) as u32) }),
                    cpdtype_id: CPDTypeID(register_state.saved_registers_without_ip.get_register(GetStatic::CPDTYPE_ID) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(GetStatic::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::InstanceOf => {
                RuntimeVMExitInput::InstanceOf {
                    res: register_state.saved_registers_without_ip.get_register(InstanceOf::RES_VALUE_PTR) as *mut c_void,
                    value: register_state.saved_registers_without_ip.get_register(InstanceOf::VALUE_PTR) as *const c_void,
                    cpdtype_id: CPDTypeID(register_state.saved_registers_without_ip.get_register(InstanceOf::CPDTYPE_ID) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(InstanceOf::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::CheckCast => {
                RuntimeVMExitInput::CheckCast {
                    value: register_state.saved_registers_without_ip.get_register(CheckCast::VALUE_PTR) as *const c_void,
                    cpdtype_id: CPDTypeID(register_state.saved_registers_without_ip.get_register(CheckCast::CPDTYPE_ID) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(CheckCast::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::RunNativeVirtual => {
                RuntimeVMExitInput::RunNativeVirtual {
                    res_ptr: register_state.saved_registers_without_ip.get_register(RunNativeVirtual::RES_PTR) as *mut c_void,
                    arg_start: register_state.saved_registers_without_ip.get_register(RunNativeVirtual::ARG_START) as *const c_void,
                    method_id: register_state.saved_registers_without_ip.get_register(RunNativeVirtual::METHODID) as MethodId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunNativeVirtual::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::RunNativeSpecial => {
                let arg_start = register_state.saved_registers_without_ip.get_register(RunNativeSpecial::ARG_START) as *const c_void;
                RuntimeVMExitInput::RunNativeSpecial {
                    res_ptr: register_state.saved_registers_without_ip.get_register(RunNativeSpecial::RES_PTR) as *mut c_void,
                    arg_start,
                    method_id: register_state.saved_registers_without_ip.get_register(RunNativeSpecial::METHODID) as MethodId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunNativeSpecial::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::Todo => {
                todo!()
            }
            RawVMExitType::InvokeInterfaceResolve => {
                RuntimeVMExitInput::InvokeInterfaceResolve {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::RESTART_IP) as *const c_void,
                    object_ref: register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::OBJECT_REF) as *const c_void,
                    target_method_id: register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::TARGET_METHOD_ID) as MethodId,
                }
            }
        }
    }
}


pub enum RuntimeVMExitOutput {
    Allocate {}
}

impl RuntimeVMExitOutput {}


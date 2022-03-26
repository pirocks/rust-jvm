use std::ffi::c_void;
use std::ptr::{NonNull, null_mut};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use add_only_static_vec::AddOnlyId;
use another_jit_vm::saved_registers_utils::SavedRegistersWithIP;
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::method_shape::MethodShapeID;
use rust_jvm_common::{ByteCodeOffset, FieldId, MethodId};
use rust_jvm_common::compressed_classfile::{CompressedClassfileString, CPDType};
use rust_jvm_common::compressed_classfile::names::FieldName;
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;
use crate::RestartPointID;
use crate::vm_exit_abi::register_structs::{AllocateObject, AllocateObjectArray, CheckCast, CompileFunctionAndRecompileCurrent, GetStatic, InitClassAndRecompile, InstanceOf, InvokeInterfaceResolve, InvokeVirtualResolve, LogFramePointerOffsetValue, LogWholeFrame, MonitorEnter, MonitorExit, MultiAllocateArray, NewClass, NewString, PutStatic, RunNativeSpecial, RunNativeVirtual, RunStaticNative, Throw, TopLevelReturn, TraceInstructionAfter, TraceInstructionBefore};

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum RawVMExitType {
    AllocateObjectArray = 1,
    MultiAllocateObjectArray,
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
    MultiAllocateArray {
        elem_type: CPDTypeID,
        num_arrays: u8,
        len_start: *const i64,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
    },
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
        native_method_res: *mut c_void,
    },
    InvokeInterfaceResolve {
        return_to_ptr: *const c_void,
        native_method_restart_point: RestartPointID,
        native_method_res: *mut c_void,
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
                    native_method_res,
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
                    field_name: FieldName(CompressedClassfileString { id: AddOnlyId(register_state.saved_registers_without_ip.get_register(GetStatic::FIELD_NAME) as u32) }),
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
                    native_method_restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::NATIVE_RESTART_POINT)),
                    native_method_res: register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::NATIVE_RETURN_PTR) as *mut c_void,
                    object_ref: register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::OBJECT_REF) as *const c_void,
                    target_method_id: register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::TARGET_METHOD_ID) as MethodId,
                }
            }
            RawVMExitType::MultiAllocateObjectArray => {
                RuntimeVMExitInput::MultiAllocateArray {
                    elem_type: CPDTypeID(register_state.saved_registers_without_ip.get_register(MultiAllocateArray::ELEM_TYPE) as u32),
                    len_start: register_state.saved_registers_without_ip.get_register(MultiAllocateArray::LEN_START) as *const i64,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(MultiAllocateArray::RESTART_IP) as *const c_void,
                    res_address: register_state.saved_registers_without_ip.get_register(MultiAllocateArray::RES_PTR) as *mut NonNull<c_void>,
                    num_arrays: register_state.saved_registers_without_ip.get_register(MultiAllocateArray::NUM_ARRAYS) as u8,
                }
            }
        }
    }
}

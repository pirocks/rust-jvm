use std::ffi::c_void;
use std::ptr::{NonNull, null_mut};
use nonnull_const::NonNullConst;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use add_only_static_vec::AddOnlyId;
use another_jit_vm::Register;
use another_jit_vm::saved_registers_utils::SavedRegistersWithIP;
use method_table::interface_table::InterfaceID;
use runtime_class_stuff::method_numbers::MethodNumber;
use rust_jvm_common::{ByteCodeOffset, FieldId, MethodId};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileString;


use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::method_shape::MethodShapeID;
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::RestartPointID;
use crate::vm_exit_abi::register_structs::{AllocateObject, AllocateObjectArray, AllocateObjectArrayIntrinsic, ArrayOutOfBounds, AssertInstanceOf, CheckCast, CheckCastFailure, CompileFunctionAndRecompileCurrent, GetStatic, InitClassAndRecompile, InstanceOf, InvokeInterfaceResolve, InvokeVirtualResolve, LogFramePointerOffsetValue, LogWholeFrame, MonitorEnter, MonitorEnterRegister, MonitorExit, MultiAllocateArray, NewClass, NewClassRegister, NewString, NPE, PutStatic, RunInterpreted, RunNativeSpecial, RunNativeVirtual, RunStaticNative, RunStaticNativeNew, Throw, Todo, TopLevelReturn, TraceInstructionAfter, TraceInstructionBefore};

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
    CheckCastFailure,
    ArrayOutOfBounds,
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
    AssertInstanceOf,
    NewClassRegister,
    MonitorEnterRegister,
    MonitorExitRegister,
    CheckCast,
    RunNativeVirtual,
    RunNativeSpecial,
    Todo,
    RunStaticNativeNew,
    RunSpecialNativeNew,
    RunInterpreted,
    AllocateObjectArrayIntrinsic,
}


#[derive(FromPrimitive)]
#[repr(u64)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TodoCase {
    CheckcastFailure,
    CheckcastFailure2,
    TableSwitchFallthrough,
    ArrayCopyFailure,
    Other
}

#[derive(Debug)]
pub enum RuntimeVMExitInput {
    MultiAllocateArray {
        elem_type: CPDTypeID,
        num_arrays: u8,
        len_start: *const i64,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
        pc: ByteCodeOffset,
    },
    AllocateObjectArray {
        type_: CPDTypeID,
        len: i32,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
        pc: ByteCodeOffset,
    },
    AllocateObjectArrayIntrinsic {
        type_: CPDTypeID,
        len: i32,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
    },
    AllocateObject {
        type_: CPDTypeID,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
        pc: ByteCodeOffset,
    },
    AllocatePrimitiveArray {
        type_: CPDType,
        len: u64,
        return_to_ptr: *const c_void,
        res_address: *mut NonNull<c_void>,
        pc: ByteCodeOffset,
    },
    LoadClassAndRecompile {
        class_type: CPDType,
        // todo static args?
        restart_point: RestartPointID,
        pc: ByteCodeOffset,
    },
    InitClassAndRecompile {
        class_type: CPDTypeID,
        current_method_id: MethodId,
        restart_point: RestartPointID,
        pc: ByteCodeOffset,
    },
    RunStaticNative {
        method_id: MethodId,
        arg_start: *mut c_void,
        num_args: u16,
        res_ptr: *mut c_void,
        return_to_ptr: *mut c_void,
        pc: ByteCodeOffset,
    },
    NPE {
        pc: ByteCodeOffset
    },
    CheckCastFailure {
        pc: ByteCodeOffset
    },
    ArrayOutOfBounds {
        pc: ByteCodeOffset,
        index: NonNullConst<c_void>
    },
    TopLevelReturn {
        return_value: u64,
    },
    CompileFunctionAndRecompileCurrent {
        current_method_id: MethodId,
        to_recompile: MethodId,
        restart_point: RestartPointID,
        pc: ByteCodeOffset,
    },
    PutStatic {
        value_ptr: *mut c_void,
        field_id: FieldId,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    Throw {
        exception_obj_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    GetStatic {
        res_value_ptr: *mut c_void,
        field_name: FieldName,
        cpdtype_id: CPDTypeID,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    LogFramePointerOffsetValue {
        value: u64,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
        // str_message: &'static str
    },
    LogWholeFrame {
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    TraceInstructionBefore {
        method_id: MethodId,
        bytecode_offset: ByteCodeOffset,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    TraceInstructionAfter {
        method_id: MethodId,
        bytecode_offset: ByteCodeOffset,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    NewString {
        return_to_ptr: *const c_void,
        res: *mut c_void,
        compressed_wtf8: CompressedWtf8String,
        pc: ByteCodeOffset,
    },
    NewClass {
        return_to_ptr: *const c_void,
        res: *mut c_void,
        type_: CPDTypeID,
        pc: ByteCodeOffset,
    },
    NewClassRegister {
        return_to_ptr: *const c_void,
        res: Register,
        type_: CPDTypeID,
        pc: ByteCodeOffset,
    },
    InvokeVirtualResolve {
        return_to_ptr: *const c_void,
        object_ref_ptr: *const c_void,
        method_shape_id: MethodShapeID,
        method_number: u32,
        native_method_restart_point: RestartPointID,
        native_method_res: *mut c_void,
        pc: ByteCodeOffset,
    },
    InvokeInterfaceResolve {
        return_to_ptr: *const c_void,
        native_method_restart_point: RestartPointID,
        native_method_res: *mut c_void,
        object_ref: *const c_void,
        method_shape_id: MethodShapeID,
        method_number: MethodNumber,
        interface_id: InterfaceID,
        pc: ByteCodeOffset,
    },
    MonitorEnter {
        obj_ptr: *const c_void,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    MonitorEnterRegister {
        obj_ptr: *const c_void,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    MonitorExit {
        obj_ptr: *const c_void,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    MonitorExitRegister {
        obj_ptr: *const c_void,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    InstanceOf {
        res: *mut c_void,
        value: *const c_void,
        cpdtype_id: CPDTypeID,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    AssertInstanceOf {
        res: *mut c_void,
        value: *const c_void,
        cpdtype_id: CPDTypeID,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
        expected: bool,
    },
    CheckCast {
        value: *const c_void,
        cpdtype_id: CPDTypeID,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    RunNativeVirtual {
        res_ptr: *mut c_void,
        arg_start: *const c_void,
        method_id: MethodId,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    RunNativeSpecial {
        res_ptr: *mut c_void,
        arg_start: *const c_void,
        method_id: MethodId,
        return_to_ptr: *const c_void,
        pc: ByteCodeOffset,
    },
    RunNativeSpecialNew {
        method_id: MethodId,
        return_to_ptr: *const c_void,
    },
    RunNativeStaticNew {
        method_id: MethodId,
        // pc: ByteCodeOffset,
        return_to_ptr: *const c_void,
    },
    RunInterpreted {
        method_id: MethodId,
        return_to_ptr: *const c_void,
    },
    Todo {
        pc: ByteCodeOffset,
        todo_case: TodoCase,
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
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(AllocateObjectArray::JAVA_PC) as u16),
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
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(RunStaticNative::JAVA_PC) as u16),
                }
            }
            RawVMExitType::TopLevelReturn => {
                RuntimeVMExitInput::TopLevelReturn {
                    return_value: register_state.saved_registers_without_ip.get_register(TopLevelReturn::RES),
                }
            }
            RawVMExitType::CompileFunctionAndRecompileCurrent => {
                RuntimeVMExitInput::CompileFunctionAndRecompileCurrent {
                    current_method_id: register_state.saved_registers_without_ip.get_register(CompileFunctionAndRecompileCurrent::CURRENT) as MethodId,
                    to_recompile: register_state.saved_registers_without_ip.get_register(CompileFunctionAndRecompileCurrent::TO_RECOMPILE) as MethodId,
                    restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(CompileFunctionAndRecompileCurrent::RESTART_POINT_ID)),
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(CompileFunctionAndRecompileCurrent::JAVA_PC) as u16),
                }
            }
            RawVMExitType::NPE => {
                RuntimeVMExitInput::NPE {
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(NPE::JAVA_PC) as u16)
                }
            }
            RawVMExitType::CheckCastFailure => {
                RuntimeVMExitInput::CheckCastFailure {
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(CheckCastFailure::JAVA_PC) as u16)
                }
            }
            RawVMExitType::PutStatic => {
                RuntimeVMExitInput::PutStatic {
                    value_ptr: register_state.saved_registers_without_ip.get_register(PutStatic::VALUE_PTR) as *mut c_void,
                    field_id: register_state.saved_registers_without_ip.get_register(PutStatic::FIELD_ID) as FieldId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(PutStatic::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(PutStatic::JAVA_PC) as u16),
                }
            }
            RawVMExitType::InitClassAndRecompile => {
                RuntimeVMExitInput::InitClassAndRecompile {
                    class_type: CPDTypeID(register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::CPDTYPE_ID) as u32),
                    current_method_id: register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::TO_RECOMPILE) as MethodId,
                    restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::RESTART_POINT_ID)),
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::JAVA_PC) as u16),
                }
            }
            RawVMExitType::LogFramePointerOffsetValue => {
                RuntimeVMExitInput::LogFramePointerOffsetValue {
                    value: register_state.saved_registers_without_ip.get_register(LogFramePointerOffsetValue::VALUE) as u64,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(LogFramePointerOffsetValue::RESTART_IP) as *const c_void,
                    // str_message: register_state.saved_registers_without_ip.get_register(LogFramePointerOffsetValue::STRING_MESSAGE)
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(LogFramePointerOffsetValue::JAVA_PC) as u16),
                }
            }
            RawVMExitType::LogWholeFrame => {
                RuntimeVMExitInput::LogWholeFrame {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(LogWholeFrame::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(LogWholeFrame::JAVA_PC) as u16),
                }
            }
            RawVMExitType::TraceInstructionBefore => {
                RuntimeVMExitInput::TraceInstructionBefore {
                    method_id: register_state.saved_registers_without_ip.get_register(TraceInstructionBefore::METHOD_ID) as MethodId,
                    bytecode_offset: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(TraceInstructionBefore::BYTECODE_OFFSET) as u16),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(TraceInstructionBefore::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(TraceInstructionBefore::JAVA_PC) as u16),
                }
            }
            RawVMExitType::TraceInstructionAfter => {
                RuntimeVMExitInput::TraceInstructionAfter {
                    method_id: register_state.saved_registers_without_ip.get_register(TraceInstructionAfter::METHOD_ID) as MethodId,
                    bytecode_offset: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(TraceInstructionAfter::BYTECODE_OFFSET) as u16),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(TraceInstructionAfter::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(TraceInstructionAfter::JAVA_PC) as u16),
                }
            }
            RawVMExitType::AllocateObject => {
                RuntimeVMExitInput::AllocateObject {
                    type_: CPDTypeID(register_state.saved_registers_without_ip.get_register(AllocateObject::TYPE) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(AllocateObject::RESTART_IP) as *const c_void,
                    res_address: register_state.saved_registers_without_ip.get_register(AllocateObject::RES_PTR) as *mut NonNull<c_void>,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(AllocateObject::JAVA_PC) as u16),
                }
            }
            RawVMExitType::NewString => {
                RuntimeVMExitInput::NewString {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(NewString::RESTART_IP) as *const c_void,
                    res: register_state.saved_registers_without_ip.get_register(NewString::RES) as *mut c_void,
                    compressed_wtf8: CompressedWtf8String(register_state.saved_registers_without_ip.get_register(NewString::COMPRESSED_WTF8) as usize),
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(NewString::JAVA_PC) as u16),
                }
            }
            RawVMExitType::NewClass => {
                RuntimeVMExitInput::NewClass {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(NewClass::RESTART_IP) as *const c_void,
                    res: register_state.saved_registers_without_ip.get_register(NewClass::RES) as *mut c_void,
                    type_: CPDTypeID(register_state.saved_registers_without_ip.get_register(NewClass::CPDTYPE_ID) as u32),
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(NewClass::JAVA_PC) as u16),
                }
            }
            RawVMExitType::NewClassRegister => {
                RuntimeVMExitInput::NewClassRegister {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(NewClassRegister::RESTART_IP) as *const c_void,
                    res: NewClassRegister::RES,
                    type_: CPDTypeID(register_state.saved_registers_without_ip.get_register(NewClassRegister::CPDTYPE_ID) as u32),
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(NewClassRegister::JAVA_PC) as u16),
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
                    method_number: register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::METHOD_NUMBER) as u32,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::JAVA_PC) as u16),
                }
            }
            RawVMExitType::MonitorEnter => {
                RuntimeVMExitInput::MonitorEnter {
                    obj_ptr: register_state.saved_registers_without_ip.get_register(MonitorEnter::OBJ_ADDR) as *const c_void,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(MonitorEnter::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(MonitorEnter::JAVA_PC) as u16),
                }
            }
            RawVMExitType::MonitorExit => {
                RuntimeVMExitInput::MonitorExit {
                    obj_ptr: register_state.saved_registers_without_ip.get_register(MonitorExit::OBJ_ADDR) as *const c_void,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(MonitorExit::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(MonitorExit::JAVA_PC) as u16),
                }
            }
            RawVMExitType::MonitorEnterRegister => {
                RuntimeVMExitInput::MonitorEnterRegister {
                    obj_ptr: register_state.saved_registers_without_ip.get_register(MonitorEnterRegister::OBJ) as *const c_void,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(MonitorEnterRegister::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(MonitorEnterRegister::JAVA_PC) as u16),
                }
            }
            RawVMExitType::MonitorExitRegister => {
                RuntimeVMExitInput::MonitorExitRegister {
                    obj_ptr: register_state.saved_registers_without_ip.get_register(MonitorEnterRegister::OBJ) as *const c_void,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(MonitorEnterRegister::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(MonitorEnterRegister::JAVA_PC) as u16),
                }
            }
            RawVMExitType::Throw => {
                RuntimeVMExitInput::Throw {
                    exception_obj_ptr: register_state.saved_registers_without_ip.get_register(Throw::EXCEPTION_PTR) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(Throw::JAVA_PC) as u16),
                }
            }
            RawVMExitType::GetStatic => {
                RuntimeVMExitInput::GetStatic {
                    res_value_ptr: register_state.saved_registers_without_ip.get_register(GetStatic::RES_VALUE_PTR) as *mut c_void,
                    field_name: FieldName(CompressedClassfileString { id: AddOnlyId(register_state.saved_registers_without_ip.get_register(GetStatic::FIELD_NAME) as u32) }),
                    cpdtype_id: CPDTypeID(register_state.saved_registers_without_ip.get_register(GetStatic::CPDTYPE_ID) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(GetStatic::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(GetStatic::JAVA_PC) as u16),
                }
            }
            RawVMExitType::InstanceOf => {
                RuntimeVMExitInput::InstanceOf {
                    res: register_state.saved_registers_without_ip.get_register(InstanceOf::RES_VALUE_PTR) as *mut c_void,
                    value: register_state.saved_registers_without_ip.get_register(InstanceOf::VALUE_PTR) as *const c_void,
                    cpdtype_id: CPDTypeID(register_state.saved_registers_without_ip.get_register(InstanceOf::CPDTYPE_ID) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(InstanceOf::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(InstanceOf::JAVA_PC) as u16),
                }
            }
            RawVMExitType::CheckCast => {
                RuntimeVMExitInput::CheckCast {
                    value: register_state.saved_registers_without_ip.get_register(CheckCast::VALUE_PTR) as *const c_void,
                    cpdtype_id: CPDTypeID(register_state.saved_registers_without_ip.get_register(CheckCast::CPDTYPE_ID) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(CheckCast::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(CheckCast::JAVA_PC) as u16),
                }
            }
            RawVMExitType::RunNativeVirtual => {
                RuntimeVMExitInput::RunNativeVirtual {
                    res_ptr: register_state.saved_registers_without_ip.get_register(RunNativeVirtual::RES_PTR) as *mut c_void,
                    arg_start: register_state.saved_registers_without_ip.get_register(RunNativeVirtual::ARG_START) as *const c_void,
                    method_id: register_state.saved_registers_without_ip.get_register(RunNativeVirtual::METHODID) as MethodId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunNativeVirtual::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(RunNativeVirtual::JAVA_PC) as u16),
                }
            }
            RawVMExitType::RunNativeSpecial => {
                let arg_start = register_state.saved_registers_without_ip.get_register(RunNativeSpecial::ARG_START) as *const c_void;
                RuntimeVMExitInput::RunNativeSpecial {
                    res_ptr: register_state.saved_registers_without_ip.get_register(RunNativeSpecial::RES_PTR) as *mut c_void,
                    arg_start,
                    method_id: register_state.saved_registers_without_ip.get_register(RunNativeSpecial::METHODID) as MethodId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunNativeSpecial::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(RunNativeSpecial::JAVA_PC) as u16),
                }
            }
            RawVMExitType::Todo => {
                RuntimeVMExitInput::Todo {
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(Todo::JAVA_PC) as u16),
                    todo_case: TodoCase::from_u64(register_state.saved_registers_without_ip.get_register(Todo::TODO_CASE) as u64).unwrap(),
                }
            }
            RawVMExitType::InvokeInterfaceResolve => {
                RuntimeVMExitInput::InvokeInterfaceResolve {
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::RESTART_IP) as *const c_void,
                    native_method_restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::NATIVE_RESTART_POINT)),
                    native_method_res: register_state.saved_registers_without_ip.get_register(InvokeVirtualResolve::NATIVE_RETURN_PTR) as *mut c_void,
                    object_ref: register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::OBJECT_REF) as *const c_void,
                    method_shape_id: MethodShapeID(register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::METHOD_SHAPE_ID) as u64),
                    method_number: MethodNumber(register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::METHOD_NUMBER) as u32),
                    interface_id: InterfaceID(register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::INTERFACE_ID) as u32),
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(InvokeInterfaceResolve::JAVA_PC) as u16),
                }
            }
            RawVMExitType::MultiAllocateObjectArray => {
                RuntimeVMExitInput::MultiAllocateArray {
                    elem_type: CPDTypeID(register_state.saved_registers_without_ip.get_register(MultiAllocateArray::ELEM_TYPE) as u32),
                    len_start: register_state.saved_registers_without_ip.get_register(MultiAllocateArray::LEN_START) as *const i64,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(MultiAllocateArray::RESTART_IP) as *const c_void,
                    res_address: register_state.saved_registers_without_ip.get_register(MultiAllocateArray::RES_PTR) as *mut NonNull<c_void>,
                    num_arrays: register_state.saved_registers_without_ip.get_register(MultiAllocateArray::NUM_ARRAYS) as u8,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(MultiAllocateArray::JAVA_PC) as u16),
                }
            }
            RawVMExitType::RunStaticNativeNew => {
                RuntimeVMExitInput::RunNativeStaticNew {
                    method_id: register_state.saved_registers_without_ip.get_register(RunStaticNativeNew::METHOD_ID) as MethodId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunStaticNativeNew::RETURN_TO_PTR) as *const c_void,
                    // pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(RunStaticNativeNew::JAVA_PC) as u16),
                }
            }
            RawVMExitType::RunSpecialNativeNew => {
                RuntimeVMExitInput::RunNativeSpecialNew {
                    method_id: register_state.saved_registers_without_ip.get_register(RunStaticNativeNew::METHOD_ID) as MethodId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunStaticNativeNew::RETURN_TO_PTR) as *const c_void,
                }
            }
            RawVMExitType::RunInterpreted => {
                RuntimeVMExitInput::RunInterpreted {
                    method_id: register_state.saved_registers_without_ip.get_register(RunInterpreted::METHOD_ID) as MethodId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunInterpreted::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::AssertInstanceOf => {
                RuntimeVMExitInput::AssertInstanceOf {
                    res: register_state.saved_registers_without_ip.get_register(AssertInstanceOf::RES_VALUE_PTR) as *mut c_void,
                    value: register_state.saved_registers_without_ip.get_register(AssertInstanceOf::VALUE_PTR) as *const c_void,
                    cpdtype_id: CPDTypeID(register_state.saved_registers_without_ip.get_register(AssertInstanceOf::CPDTYPE_ID) as u32),
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(AssertInstanceOf::RESTART_IP) as *const c_void,
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(AssertInstanceOf::JAVA_PC) as u16),
                    expected: register_state.saved_registers_without_ip.get_register(AssertInstanceOf::FAST_INSTANCE_OF_RES) as u64 != 0,
                }
            }
            RawVMExitType::ArrayOutOfBounds => {
                RuntimeVMExitInput::ArrayOutOfBounds {
                    pc: ByteCodeOffset(register_state.saved_registers_without_ip.get_register(ArrayOutOfBounds::JAVA_PC) as u16),
                    index: NonNullConst::new(register_state.saved_registers_without_ip.get_register(ArrayOutOfBounds::INDEX) as *const c_void).unwrap()
                }
            }
            RawVMExitType::AllocateObjectArrayIntrinsic => {
                unsafe {
                    RuntimeVMExitInput::AllocateObjectArrayIntrinsic {
                        type_: CPDTypeID((register_state.saved_registers_without_ip.get_register(AllocateObjectArrayIntrinsic::TYPE) as *const u32).read()),
                        len: (register_state.saved_registers_without_ip.get_register(AllocateObjectArrayIntrinsic::LEN) as *const i32).read(),
                        return_to_ptr: register_state.saved_registers_without_ip.get_register(AllocateObjectArrayIntrinsic::RESTART_IP) as *const c_void,
                        res_address: register_state.saved_registers_without_ip.get_register(AllocateObjectArrayIntrinsic::RES_PTR) as *mut NonNull<c_void>,
                    }
                }
            }
        }
    }

    pub fn exiting_pc(&self) -> Option<ByteCodeOffset> {
        match self {
            RuntimeVMExitInput::MultiAllocateArray { pc, .. } => Some(*pc),
            RuntimeVMExitInput::AllocateObjectArray { pc, .. } => Some(*pc),
            RuntimeVMExitInput::AllocateObject { pc, .. } => Some(*pc),
            RuntimeVMExitInput::AllocatePrimitiveArray { pc, .. } => Some(*pc),
            RuntimeVMExitInput::LoadClassAndRecompile { pc, .. } => Some(*pc),
            RuntimeVMExitInput::InitClassAndRecompile { pc, .. } => Some(*pc),
            RuntimeVMExitInput::RunStaticNative { pc, .. } => Some(*pc),
            RuntimeVMExitInput::NPE { pc, .. } => Some(*pc),
            RuntimeVMExitInput::TopLevelReturn { .. } => None,
            RuntimeVMExitInput::CompileFunctionAndRecompileCurrent { pc, .. } => Some(*pc),
            RuntimeVMExitInput::PutStatic { pc, .. } => Some(*pc),
            RuntimeVMExitInput::Throw { pc, .. } => Some(*pc),
            RuntimeVMExitInput::GetStatic { pc, .. } => Some(*pc),
            RuntimeVMExitInput::LogFramePointerOffsetValue { pc, .. } => Some(*pc),
            RuntimeVMExitInput::LogWholeFrame { pc, .. } => Some(*pc),
            RuntimeVMExitInput::TraceInstructionBefore { pc, .. } => Some(*pc),
            RuntimeVMExitInput::TraceInstructionAfter { pc, .. } => Some(*pc),
            RuntimeVMExitInput::NewString { pc, .. } => Some(*pc),
            RuntimeVMExitInput::NewClass { pc, .. } => Some(*pc),
            RuntimeVMExitInput::InvokeVirtualResolve { pc, .. } => Some(*pc),
            RuntimeVMExitInput::InvokeInterfaceResolve { pc, .. } => Some(*pc),
            RuntimeVMExitInput::MonitorEnter { pc, .. } => Some(*pc),
            RuntimeVMExitInput::MonitorExit { pc, .. } => Some(*pc),
            RuntimeVMExitInput::InstanceOf { pc, .. } => Some(*pc),
            RuntimeVMExitInput::CheckCast { pc, .. } => Some(*pc),
            RuntimeVMExitInput::RunNativeVirtual { pc, .. } => Some(*pc),
            RuntimeVMExitInput::RunNativeSpecial { pc, .. } => Some(*pc),
            RuntimeVMExitInput::RunNativeSpecialNew { .. } => None,
            RuntimeVMExitInput::RunNativeStaticNew { .. } => None,
            RuntimeVMExitInput::RunInterpreted { .. } => None,
            RuntimeVMExitInput::AssertInstanceOf { pc, .. } => Some(*pc),
            RuntimeVMExitInput::NewClassRegister { pc, .. } => Some(*pc),
            RuntimeVMExitInput::MonitorEnterRegister { pc, .. } => Some(*pc),
            RuntimeVMExitInput::MonitorExitRegister { pc, .. } => Some(*pc),
            RuntimeVMExitInput::ArrayOutOfBounds { pc, .. } => Some(*pc),
            RuntimeVMExitInput::Todo { pc, .. } => Some(*pc),
            RuntimeVMExitInput::AllocateObjectArrayIntrinsic { .. } => None,
            RuntimeVMExitInput::CheckCastFailure { pc, .. } => Some(*pc)
        }
    }
}

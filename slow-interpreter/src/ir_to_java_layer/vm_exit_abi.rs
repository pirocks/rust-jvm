use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::RwLock;

use bimap::BiHashMap;
use crossbeam::channel::after;
use iced_x86::code_asm::{CodeAssembler, CodeLabel, k0, k1, qword_ptr, rax, rbp, rbx, xmm0};
use nix::convert_ioctl_res;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use another_jit_vm::{Register, SavedRegistersWithIP, SavedRegistersWithoutIP};
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::loading::LoaderName;

use crate::cpdtype_table::CPDTypeID;
use crate::field_table::FieldId;
use crate::gc_memory_layout_common::{AllocatedTypeID, FramePointerOffset};
use crate::ir_to_java_layer::compiler::ByteCodeIndex;
use crate::ir_to_java_layer::vm_exit_abi::IRVMExitType::AllocateObjectArray_;
use crate::java_values::NativeJavaValue;
use crate::jit::MethodResolver;
use crate::JVMState;
use crate::method_table::MethodId;

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

pub struct AllocateVMExit;

impl AllocateVMExit {
    pub const RES: Register = Register(1);
    pub const TYPE: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
}

pub struct AllocateObjectArray;

impl AllocateObjectArray {
    pub const LEN: Register = Register(2);
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
    pub const RESTART_IP: Register = Register(5); //methodid
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

pub struct LoadClassAndRecompile {
    static_arg: LoadClassAndRecompileStaticArgsID,
}

impl LoadClassAndRecompile {
    pub const VM_EXIT_STATIC_ARGS_ID: Register = Register(1);
    pub const LOADER_NUM: Register = Register(2);
}

pub enum IRVMExitType {
    AllocateObjectArray_ {
        array_type: CPDTypeID,
        arr_len: FramePointerOffset,
        arr_res: FramePointerOffset,
    },
    NPE,
    LoadClassAndRecompile {
        class: CPDTypeID,
    },
    InitClassAndRecompile {
        class: CPDTypeID,
        this_method_id: MethodId,
        restart_point_id: RestartPointID,
    },
    RunStaticNative {
        //todo should I actually use these args?
        method_id: MethodId,
        arg_start_frame_offset: FramePointerOffset,
        num_args: u16,
    },
    CompileFunctionAndRecompileCurrent {
        current_method_id: MethodId,
        target_method_id: MethodId,
        restart_point_id: RestartPointID,
    },
    TopLevelReturn,
    PutStatic {
        field_id: FieldId,
        value: FramePointerOffset,
    },
}

impl IRVMExitType {
    pub fn gen_assembly(&self, assembler: &mut CodeAssembler, before_exit_label: &mut CodeLabel, after_exit_label: &mut CodeLabel, registers: Vec<Register>) {
        match self {
            IRVMExitType::AllocateObjectArray_ { array_type, arr_len, arr_res } => {
                // assembler.int3().unwrap();
                assembler.mov(rax, RawVMExitType::AllocateObjectArray as u64).unwrap();
                assembler.mov(AllocateObjectArray::TYPE.to_native_64(), array_type.0 as u64).unwrap();
                assembler.mov(AllocateObjectArray::LEN.to_native_64(), rbp - arr_len.0).unwrap();
                assembler.lea(AllocateObjectArray::RES_PTR.to_native_64(), rbp - arr_res.0).unwrap();
                assembler.lea(AllocateObjectArray::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
            IRVMExitType::LoadClassAndRecompile { .. } => {
                todo!()
            }
            IRVMExitType::RunStaticNative { method_id, arg_start_frame_offset, num_args } => {
                assert!(registers.contains(&RunStaticNative::METHODID));
                assert!(registers.contains(&RunStaticNative::RESTART_IP));
                assert!(registers.contains(&RunStaticNative::NUM_ARGS));
                assert!(registers.contains(&RunStaticNative::RES));
                assert!(registers.contains(&RunStaticNative::ARG_START));
                assembler.mov(rax, RawVMExitType::RunStaticNative as u64).unwrap();
                assembler.mov(RunStaticNative::METHODID.to_native_64(), *method_id as u64).unwrap();
                assembler.lea(RunStaticNative::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
                assembler.lea(RunStaticNative::ARG_START.to_native_64(), rbp - arg_start_frame_offset.0).unwrap();
                assembler.mov(RunStaticNative::NUM_ARGS.to_native_64(), *num_args as u64).unwrap();
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
                assembler.lea(PutStatic::VALUE_PTR.to_native_64(), rbp + value.0).unwrap();
                assembler.mov(PutStatic::FIELD_ID.to_native_64(), *field_id as u64).unwrap();
                assembler.lea(PutStatic::RESTART_IP.to_native_64(), qword_ptr(after_exit_label.clone())).unwrap();
            }
        }
    }
}

pub enum VMExitTypeWithArgs {
    Allocate(AllocateVMExit),
    LoadClassAndRecompile(LoadClassAndRecompile),
    RunStaticNative(RunStaticNative),
    TopLevelReturn,
}


#[derive(FromPrimitive)]
#[repr(u64)]
pub enum RawVMExitType {
    AllocateObjectArray = 1,
    LoadClassAndRecompile,
    InitClassAndRecompile,
    RunStaticNative,
    TopLevelReturn,
    CompileFunctionAndRecompileCurrent,
    NPE,
    PutStatic,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct RestartPointID(pub(crate) u64);

#[derive(Debug)]
pub enum RuntimeVMExitInput {
    AllocateObjectArray {
        type_: CPDTypeID,
        len: i32,
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
        rbp: *const c_void
    },
    RunStaticNative {
        method_id: MethodId,
        arg_start: *mut c_void,
        num_args: u16,
        res_ptr: *mut NativeJavaValue<'static>,
        return_to_ptr: *mut c_void,
    },
    TopLevelReturn {
        return_value: u64
    },
    CompileFunctionAndRecompileCurrent {
        current_method_id: MethodId,
        to_recompile: MethodId,
        restart_point: RestartPointID,
    },
    PutStatic {
        value: *mut NativeJavaValue<'static>,
        field_id: FieldId,
        return_to_ptr: *const c_void,
    },
}

impl RuntimeVMExitInput {
    pub fn from_register_state(register_state: &SavedRegistersWithIP) -> Self {
        let SavedRegistersWithoutIP {
            rcx,
            rdx,
            rsi,
            rdi,
            rsp,
            r8,
            r9,
            r10,
            r11,
            r12,
            r13,
            r14,
            xsave_area,
            ..
        } = register_state.saved_registers_without_ip;
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
                    res_ptr: register_state.saved_registers_without_ip.get_register(RunStaticNative::RES) as *mut NativeJavaValue,
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
                    restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(CompileFunctionAndRecompileCurrent::RESTART_POINT_ID))
                }
            }
            RawVMExitType::NPE => {
                todo!()
            }
            RawVMExitType::PutStatic => {
                RuntimeVMExitInput::PutStatic {
                    value: register_state.saved_registers_without_ip.get_register(PutStatic::VALUE_PTR) as *mut NativeJavaValue<'static>,
                    field_id: register_state.saved_registers_without_ip.get_register(PutStatic::FIELD_ID) as FieldId,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(PutStatic::RESTART_IP) as *const c_void,
                }
            }
            RawVMExitType::InitClassAndRecompile => {
                RuntimeVMExitInput::InitClassAndRecompile {
                    class_type: CPDTypeID(register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::CPDTYPE_ID) as u32),
                    current_method_id: register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::TO_RECOMPILE) as MethodId,
                    restart_point: RestartPointID(register_state.saved_registers_without_ip.get_register(InitClassAndRecompile::RESTART_POINT_ID)),
                    rbp: register_state.saved_registers_without_ip.rbp
                }
            }
        }
    }
}


pub enum RuntimeVMExitOutput {
    Allocate {}
}

impl RuntimeVMExitOutput {}

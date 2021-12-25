use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::RwLock;

use bimap::BiHashMap;
use iced_x86::code_asm::{CodeAssembler, CodeLabel, qword_ptr, rax, rbp, rbx};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use another_jit_vm::{Register, SavedRegistersWithIP, SavedRegistersWithoutIP};
use rust_jvm_common::compressed_classfile::CPDType;

use crate::gc_memory_layout_common::{AllocatedTypeID, FramePointerOffset};
use crate::ir_to_java_layer::compiler::ByteCodeIndex;
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
    Allocate,
    LoadClassAndRecompile {
        class: CPDType,
    },
    RunStaticNative {
        //todo should I actually use these args?
        method_id: MethodId,
        arg_start_frame_offset: FramePointerOffset,
        num_args: u16
    },
    CompileFunctionAndRecompileCurrent {
        method_id: MethodId
    },
    TopLevelReturn,
}

impl IRVMExitType {
    pub fn gen_assembly(&self, assembler: &mut CodeAssembler, before_exit_label: &mut CodeLabel, after_exit_label: &mut CodeLabel, registers: Vec<Register>) {
        match self {
            IRVMExitType::Allocate => {
                todo!()
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
                assembler.lea(RunStaticNative::ARG_START.to_native_64(), rbp + arg_start_frame_offset.0).unwrap();
                assembler.mov(RunStaticNative::NUM_ARGS.to_native_64(),*num_args as u64).unwrap();
                // assembler.mov(RunStaticNative::RES.to_native_64(),).unwrap()

            }
            IRVMExitType::TopLevelReturn => {
                assembler.mov(TopLevelReturn::RES.to_native_64(), rax).unwrap();
                assembler.mov(rax, RawVMExitType::TopLevelReturn as u64).unwrap();
            }
            IRVMExitType::CompileFunctionAndRecompileCurrent  { .. } => {
                //todo does nothing here using non-runtime args only
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
    Allocate = 1,
    LoadClassAndRecompile,
    RunStaticNative,
    TopLevelReturn,
}

pub enum RuntimeVMExitInput {
    Allocate {
        type_: AllocatedTypeID,
        return_to_ptr: *mut c_void,
    },
    LoadClassAndRecompile {
        class_type: CPDType,
        // todo static args?
        restart_bytecode: ByteCodeIndex,
        //if I need to restart within a bytecode have second restart point index, for within that bytecode
    },
    RunStaticNative {
        method_id: MethodId,
        arg_start: *mut c_void,
        num_args: u16,
        res_ptr: *mut NativeJavaValue<'static>,
        return_to_ptr: *mut c_void,

    },
    TopLevelReturn{
        return_value: u64
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
            RawVMExitType::Allocate => {
                let val = register_state.saved_registers_without_ip.get_register(AllocateVMExit::TYPE);
                RuntimeVMExitInput::Allocate { type_: AllocatedTypeID(val), return_to_ptr: todo!() }
            }
            RawVMExitType::LoadClassAndRecompile => todo!(),
            RawVMExitType::RunStaticNative => {
                RuntimeVMExitInput::RunStaticNative {
                    method_id: register_state.saved_registers_without_ip.get_register(RunStaticNative::METHODID) as MethodId,
                    arg_start: register_state.saved_registers_without_ip.get_register(RunStaticNative::ARG_START) as *mut c_void,
                    num_args: register_state.saved_registers_without_ip.get_register(RunStaticNative::NUM_ARGS) as u16,
                    res_ptr: register_state.saved_registers_without_ip.get_register(RunStaticNative::RES) as *mut NativeJavaValue,
                    return_to_ptr: register_state.saved_registers_without_ip.get_register(RunStaticNative::RESTART_IP) as *mut c_void
                }
            },
            RawVMExitType::TopLevelReturn => {
                RuntimeVMExitInput::TopLevelReturn{
                    return_value: register_state.saved_registers_without_ip.get_register(TopLevelReturn::RES)
                }
            },
        }
    }
}


pub enum RuntimeVMExitOutput {
    Allocate {}
}

impl RuntimeVMExitOutput {}

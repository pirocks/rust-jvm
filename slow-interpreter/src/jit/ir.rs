use std::fmt::Debug;

use iced_x86::code_asm::{AsmRegister32, AsmRegister64, CodeAssembler, ebx, ecx, edx, r10, r10d, r11, r11d, r12, r12d, r13, r13d, r14, r14d, r8, r8d, r9, r9d, rbx, rcx, rdx};
use libc::c_void;
use another_jit_vm::Register;

use crate::gc_memory_layout_common::FramePointerOffset;
use crate::ir_to_java_layer::compiler::ByteCodeIndex;
use crate::ir_to_java_layer::vm_exit_abi::{IRVMExitType, RestartPointID, VMExitTypeWithArgs};
use crate::jit::{LabelName, MethodResolver};



// pub struct FramePointerOffset(i16);


use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::{MaybeUninit, size_of, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr::{null_mut, slice_from_raw_parts};
use std::slice::from_raw_parts;
use std::sync::{Arc, RwLock};

use bimap::BiHashMap;
use iced_x86::{BlockEncoder, BlockEncoderOptions, Formatter, InstructionBlock, IntelFormatter};
use iced_x86::CC_b::c;
use iced_x86::CC_g::g;
use iced_x86::CC_np::po;
use iced_x86::code_asm::{byte_ptr, CodeAssembler, CodeLabel, qword_ptr, rax, rbp, rbx, rsp};
use itertools::Itertools;
use libc::{MAP_ANONYMOUS, MAP_GROWSDOWN, MAP_NORESERVE, MAP_PRIVATE, PROT_READ, PROT_WRITE, read, select};
use memoffset::offset_of;

use another_jit_vm::{BaseAddress, Method, MethodImplementationID, Register, SavedRegistersWithoutIP, VMExitAction, VMExitEvent, VMExitLabel, VMState};
use another_jit_vm_ir::{FramePointerOffset, IRInstr};
use rust_jvm_common::compressed_classfile::code::{CompressedInstruction, CompressedInstructionInfo};
use verification::verifier::Frame;

use crate::{InterpreterStateGuard, JavaThread, JVMState};
use crate::gc_memory_layout_common::{FramePointerOffset, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use crate::ir_to_java_layer::compiler::ByteCodeIndex;
use crate::ir_to_java_layer::java_stack::JavaStackPosition;
use crate::ir_to_java_layer::vm_exit_abi::{IRVMExitType, RestartPointID, RuntimeVMExitInput, VMExitTypeWithArgs};
use crate::jit::{ByteCodeOffset, LabelName, NativeInstructionLocation};
use crate::jit::ir::IRInstr;
use crate::method_table::MethodId;

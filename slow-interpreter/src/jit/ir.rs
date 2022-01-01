use std::fmt::Debug;

use iced_x86::code_asm::{AsmRegister32, AsmRegister64, CodeAssembler, ebx, ecx, edx, r10, r10d, r11, r11d, r12, r12d, r13, r13d, r14, r14d, r8, r8d, r9, r9d, rbx, rcx, rdx};
use libc::c_void;
use another_jit_vm::Register;

use crate::gc_memory_layout_common::FramePointerOffset;
use crate::ir_to_java_layer::compiler::ByteCodeIndex;
use crate::ir_to_java_layer::vm_exit_abi::{IRVMExitType, RestartPointID, VMExitTypeWithArgs};
use crate::jit::{LabelName, MethodResolver};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRLabel {
    pub(crate) name: LabelName,
}

// pub struct FramePointerOffset(i16);

pub enum IRInstr {
    LoadFPRelative { from: FramePointerOffset, to: Register },
    StoreFPRelative { from: Register, to: FramePointerOffset },
    Load { to: Register, from_address: Register },
    Store { to_address: Register, from: Register },
    CopyRegister { from: Register, to: Register },
    Add { res: Register, a: Register },
    Sub { res: Register, to_subtract: Register },
    Div { res: Register, divisor: Register },
    Mod { res: Register, divisor: Register },
    Mul { res: Register, a: Register },
    BinaryBitAnd { res: Register, a: Register },
    ForwardBitScan { to_scan: Register, res: Register },
    Const32bit { to: Register, const_: u32 },
    Const64bit { to: Register, const_: u64 },
    BranchToLabel { label: LabelName },
    LoadLabel { label: LabelName, to: Register },
    LoadRBP { to: Register },
    WriteRBP { from: Register },
    BranchEqual { a: Register, b: Register, label: LabelName },
    BranchNotEqual { a: Register, b: Register, label: LabelName },
    Return { return_val: Option<Register>, temp_register_1: Register, temp_register_2: Register, temp_register_3: Register, temp_register_4: Register, frame_size: usize },
    VMExit { before_exit_label: LabelName, after_exit_label: Option<LabelName>, exit_type: VMExitTypeWithArgs },
    RestartPoint(RestartPointID),
    VMExit2 { exit_type: IRVMExitType },
    NPECheck { possibly_null: Register,temp_register: Register, npe_exit_type: IRVMExitType },
    GrowStack { amount: usize },
    LoadSP { to: Register },
    WithAssembler { function: Box<dyn FnOnce(&mut CodeAssembler) -> ()> },
    IRNewFrame {
        current_frame_size: usize,
        temp_register: Register,
        return_to_rip: Register,
    },
    IRCall{
        temp_register_1: Register,
        temp_register_2: Register,
        current_frame_size: usize,
        new_frame_size : usize,
        target_address: *const c_void //todo perhaps this should be an ir_method id
    },
    FNOP,
    Label(IRLabel),
}
use another_jit_vm::Register;
use gc_memory_layout_common::FramePointerOffset;
use iced_x86::code_asm::CodeAssembler;
use std::ffi::c_void;
use crate::IRVMExitType;

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
    // VMExit { before_exit_label: LabelName, after_exit_label: Option<LabelName>, exit_type: VMExitTypeWithArgs },
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct RestartPointID(pub(crate) u64);


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LabelName(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRLabel {
    pub name: LabelName,
}


pub struct RestartPointGenerator {
    current_max_restart_point: RestartPointID,
}

impl RestartPointGenerator {
    pub fn new() -> Self {
        Self {
            current_max_restart_point: RestartPointID(0)
        }
    }

    pub fn new_restart_point(&mut self) -> RestartPointID {
        let res = self.current_max_restart_point;
        self.current_max_restart_point.0 += 1;
        res
    }
}

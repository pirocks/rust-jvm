use iced_x86::code_asm::{AsmRegister32, AsmRegister64, ebx, ecx, edx, r10, r10d, r11, r11d, r12, r12d, r13, r13d, r14, r14d, r8, r8d, r9, r9d, rbx, rcx, rdx};

use gc_memory_layout_common::FramePointerOffset;

use crate::jit2::{LabelName, VMExitType};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRLabel {
    pub(crate) name: LabelName,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Register(pub u8);


impl Register {
    pub fn to_native_64(&self) -> AsmRegister64 {
        match self.0 {
            0 => rbx,
            1 => rcx,
            2 => rdx,
            3 => r8,
            4 => r9,
            5 => r10,
            6 => r11,
            7 => r12,
            8 => r13,
            9 => r14,
            _ => todo!()
        }
    }

    pub fn to_native_32(&self) -> AsmRegister32 {
        match self.0 {
            0 => ebx,
            1 => ecx,
            2 => edx,
            3 => r8d,
            4 => r9d,
            5 => r10d,
            6 => r11d,
            7 => r12d,
            8 => r13d,
            9 => r14d,
            _ => todo!()
        }
    }
}
// pub struct FramePointerOffset(i16);

#[derive(Debug)]
pub enum IRInstr {
    LoadFPRelative {
        from: FramePointerOffset,
        to: Register,
    },
    StoreFPRelative {
        from: Register,
        to: FramePointerOffset,
    },
    Load {
        to: Register,
        from_address: Register,
    },
    Store {
        to_address: Register,
        from: Register,
    },
    Add {
        res: Register,
        a: Register,
    },
    Sub {
        res: Register,
        to_subtract: Register,
    },
    Div {
        res: Register,
        divisor: Register,
    },
    Mod {
        res: Register,
        divisor: Register,
    },
    Mul {
        res: Register,
        a: Register,
    },
    Const32bit {
        to: Register,
        const_: u32,
    },
    Const64bit {
        to: Register,
        const_: u64,
    },
    BranchToLabel {
        label: LabelName
    },
    LoadLabel {
        label: LabelName,
        to: Register,
    },
    LoadRBP {
        to: Register
    },
    WriteRBP {
        from: Register
    },
    BranchEqual {
        a: Register,
        b: Register,
        label: LabelName,
    },
    Return {
        return_val: Option<Register>
    },
    VMExit {
        exit_label: LabelName,
        exit_type: VMExitType,
    },
    GrowStack {
        amount: usize
    },
    LoadSP {
        to: Register
    },
    FNOP,
    Label(IRLabel),
}

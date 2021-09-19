use iced_x86::code_asm::{AsmRegister64, r10, r11, r12, r13, r14, r8, r9, rbx, rcx, rdx};

use gc_memory_layout_common::FramePointerOffset;

use crate::jit2::{LabelName, VMExitType};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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
}
// pub struct FramePointerOffset(i16);

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
        b: Register,
    },
    Sub {
        res: Register,
        a: Register,
        to_subtract: Register,
    },
    Div {
        res: Register,
        to_divide: Register,
        divisor: Register,
    },
    Mod {
        res: Register,
        to_divide: Register,
        divisor: Register,
    },
    Mul {
        res: Register,
        a: Register,
        b: Register,
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
    BranchEqual {
        a: Register,
        b: Register,
        label: LabelName,
    },
    Return {
        return_val: Option<Register>
    },
    VMExit(VMExitType),
    Label(IRLabel),
}

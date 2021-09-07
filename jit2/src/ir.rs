use gc_memory_layout_common::FramePointerOffset;

use crate::{LabelName, VMExitType};

pub struct IRLabel {
    pub(crate) name: LabelName,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Register(u8);

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
    BranchIfZero {
        maybe_zero: Register,
        label: LabelName,
    },
    VMExit(VMExitType),
    Label(IRLabel),
}

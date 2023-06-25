use crate::ir_compiler_common::{DoubleValue, DoubleValueToken, IntegerValue, IntegerValueToken, LongValue, LongValueToken, PointerValue, PointerValueToken};
use crate::native_compiler_common::GeneralRegister;

pub enum GeneralRegisterStatus {
    Empty,
    PointerOccupied(PointerValueToken),
    LongOccupied(LongValueToken),
    DoubleOccupied(DoubleValueToken),
    IntegerOccupied(IntegerValueToken),
}

pub struct GeneralRegisters {
    free_bitfield: u16,
    rax: GeneralRegisterStatus,
    rbx: GeneralRegisterStatus,
}


impl GeneralRegisters{
    pub fn new() -> Self{
        Self{
            free_bitfield: u16::MAX,
            rax: GeneralRegisterStatus::Empty,
            rbx: GeneralRegisterStatus::Empty,
        }
    }
}

pub struct RegistersStatus {
    general_register_status: GeneralRegister,
}

impl RegistersStatus {
    pub fn new() -> Self{
        Self{
            general_register_status: GeneralRegister::new(),
        }
    }
}

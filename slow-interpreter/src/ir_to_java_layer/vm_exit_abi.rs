use crate::jit::ir::Register;

pub struct AllocateVMExit;

impl AllocateVMExit {
    pub const RES: Register = Register(1);
    pub const SIZE: Register = Register(2);
}

pub enum VMExitType {
    Allocate(AllocateVMExit),
    LoadClassAndRecompile,
    RunStaticNative,
    TopLevelReturn {},
}

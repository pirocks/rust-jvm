#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FramePointerOffset(pub usize);

pub const MAGIC_1_EXPECTED: u64 = 0xDEADBEEFDEADBEAF;
pub const MAGIC_2_EXPECTED: u64 = 0xDEADCAFEDEADDEAD;
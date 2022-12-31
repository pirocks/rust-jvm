#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum VectorRegister {
    ZMM0,
    ZMM1,
    ZMM2,
    ZMM3,
    ZMM4,
    ZMM5,
    ZMM6,
    ZMM7,
    ZMM8,
    ZMM9,
    ZMM10,
    ZMM11,
    ZMM12,
    ZMM13,
    ZMM14,
    ZMM15,
    ZMM16,
    ZMM17,
    ZMM18,
    ZMM19,
    ZMM20,
    ZMM21,
    ZMM22,
    ZMM23,
    ZMM24,
    ZMM25,
    ZMM26,
    ZMM27,
    ZMM28,
    ZMM29,
    ZMM30,
    ZMM31,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueVectorPosition32 {
    Pos0,
    Pos1,
    Pos2,
    Pos3,
    Pos4,
    Pos5,
    Pos6,
    Pos7,
    Pos8,
    Pos9,
    Pos10,
    Pos11,
    Pos12,
    Pos13,
    Pos14,
    Pos15,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueVectorPosition64 {
    Pos0,
    Pos1,
    Pos2,
    Pos3,
    Pos4,
    Pos5,
    Pos6,
    Pos7,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GeneralRegister {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GeneralRegisterPart {
    Lower,
    Upper
}

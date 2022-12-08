use iced_x86::code_asm::CodeAssembler;

use another_jit_vm::{DoubleRegister, FloatRegister, Register};

use crate::FloatCompareMode;

pub fn float_compare(assembler: &mut CodeAssembler, value1: FloatRegister, value2: FloatRegister, res: Register, one: Register, zero: Register, m_one: Register, compare_mode: FloatCompareMode) {
    assembler.xor(res.to_native_64(), res.to_native_64()).unwrap();
    assembler.comiss(value1.to_xmm(), value2.to_xmm()).unwrap();
    float_compare_common(assembler, res, one, zero, m_one, compare_mode);
}

pub fn double_compare(assembler: &mut CodeAssembler, value1: DoubleRegister, value2: DoubleRegister, res: Register, one: Register, zero: Register, m_one: Register, compare_mode: FloatCompareMode) {
    assembler.xor(res.to_native_64(), res.to_native_64()).unwrap();
    assembler.comisd(value1.to_xmm(), value2.to_xmm()).unwrap();
    float_compare_common(assembler, res, one, zero, m_one, compare_mode);
}


pub fn float_compare_common(assembler: &mut CodeAssembler, res: Register, one: Register, zero: Register, m_one: Register, compare_mode: FloatCompareMode) {
    assembler.mov(one.to_native_64(), 1u64).unwrap();
    assembler.mov(zero.to_native_64(), 0u64).unwrap();
    assembler.mov(m_one.to_native_64(), -1i64).unwrap();
    assembler.cmovnc(res.to_native_64(), one.to_native_64()).unwrap();
    assembler.cmovc(res.to_native_64(), m_one.to_native_64()).unwrap();
    assembler.cmovz(res.to_native_64(), zero.to_native_64()).unwrap();
    let saved = zero;
    assembler.mov(saved.to_native_64(), res.to_native_64()).unwrap();
    match compare_mode {
        FloatCompareMode::G => {
            assembler.cmovp(res.to_native_64(), one.to_native_64()).unwrap();
        }
        FloatCompareMode::L => {
            assembler.cmovp(res.to_native_64(), m_one.to_native_64()).unwrap();
        }
    }
    assembler.cmovnc(res.to_native_64(), saved.to_native_64()).unwrap();
    assembler.cmovnz(res.to_native_64(), saved.to_native_64()).unwrap();
    assembler.nop().unwrap();
}


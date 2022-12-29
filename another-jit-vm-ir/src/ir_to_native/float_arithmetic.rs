use iced_x86::code_asm::CodeAssembler;
use another_jit_vm::{DoubleRegister, FloatRegister, Register};

pub(crate) fn neg_double(assembler: &mut CodeAssembler, temp_normal: &Register, temp: &DoubleRegister, res: &DoubleRegister) {
    assembler.mov(temp_normal.to_native_64(), 0x8000000000000000u64 as i64).unwrap();
    assembler.vmovq(temp_normal.to_native_64(), temp.to_xmm()).unwrap();
    assembler.vpxor(res.to_xmm(), res.to_xmm(), temp.to_xmm()).unwrap();
}

pub(crate) fn neg_float(assembler: &mut CodeAssembler, temp_normal: &Register, temp: &FloatRegister, res: &FloatRegister) {
    assembler.mov(temp_normal.to_native_32(), 0x80000000u32 as i32).unwrap();
    assembler.vmovd(temp_normal.to_native_32(), temp.to_xmm()).unwrap();
    assembler.vpxor(res.to_xmm(), res.to_xmm(), temp.to_xmm()).unwrap();
}



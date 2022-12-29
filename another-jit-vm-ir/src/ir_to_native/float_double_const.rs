use iced_x86::code_asm::CodeAssembler;
use another_jit_vm::{DoubleRegister, FloatRegister, Register};

pub(crate) fn const_float(assembler: &mut CodeAssembler, to: &FloatRegister, const_: &f32, temp: &Register) {
    assembler.sub(temp.to_native_64(), temp.to_native_64()).unwrap();
    assembler.mov(temp.to_native_32(), const_.to_bits() as i32).unwrap();
    assembler.vmovd(to.to_xmm(), temp.to_native_32()).unwrap();
}

pub(crate) fn const_double(assembler: &mut CodeAssembler, to: &DoubleRegister, temp: &Register, const_: &f64) {
    assembler.sub(temp.to_native_64(), temp.to_native_64()).unwrap();
    assembler.mov(temp.to_native_64(), const_.to_bits() as i64).unwrap();
    assembler.vmovq(to.to_xmm(), temp.to_native_64()).unwrap();
}

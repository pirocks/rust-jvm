use iced_x86::code_asm;
use iced_x86::code_asm::{cl, CodeAssembler};

use another_jit_vm::Register;

use crate::{BitwiseLogicType, Size};

pub fn binary_bit_xor(assembler: &mut CodeAssembler, res: Register, a: Register, size: Size) {
    match size {
        Size::Byte => assembler.xor(res.to_native_8(), a.to_native_8()).unwrap(),
        Size::X86Word => assembler.xor(res.to_native_16(), a.to_native_16()).unwrap(),
        Size::X86DWord => assembler.xor(res.to_native_32(), a.to_native_32()).unwrap(),
        Size::X86QWord => assembler.xor(res.to_native_64(), a.to_native_64()).unwrap(),
    }
}

pub fn binary_bit_and(assembler: &mut CodeAssembler, res: Register, a: Register, size: Size) {
    match size {
        Size::Byte => assembler.and(res.to_native_8(), a.to_native_8()).unwrap(),
        Size::X86Word => assembler.and(res.to_native_16(), a.to_native_16()).unwrap(),
        Size::X86DWord => assembler.and(res.to_native_32(), a.to_native_32()).unwrap(),
        Size::X86QWord => assembler.and(res.to_native_64(), a.to_native_64()).unwrap(),
    }
}


pub fn shift_left(assembler: &mut CodeAssembler, res: Register, a: Register, cl_aka_register_2: Register, size: Size, signed: BitwiseLogicType) {
    assert_eq!(cl_aka_register_2.to_native_8(), cl);
    assembler.mov(cl, a.to_native_8()).unwrap();
    match signed {
        BitwiseLogicType::Arithmetic => match size {
            Size::Byte => assembler.sal(res.to_native_8(), cl).unwrap(),
            Size::X86Word => assembler.sal(res.to_native_16(), cl).unwrap(),
            Size::X86DWord => assembler.sal(res.to_native_32(), cl).unwrap(),
            Size::X86QWord => assembler.sal(res.to_native_64(), cl).unwrap(),
        },
        BitwiseLogicType::Logical => match size {
            Size::Byte => assembler.shl(res.to_native_8(), code_asm::cl).unwrap(),
            Size::X86Word => assembler.shl(res.to_native_16(), code_asm::cl).unwrap(),
            Size::X86DWord => assembler.shl(res.to_native_32(), code_asm::cl).unwrap(),
            Size::X86QWord => assembler.shl(res.to_native_64(), code_asm::cl).unwrap(),
        },
    }
}

pub fn shift_right(assembler: &mut CodeAssembler, res: Register, a: Register, cl_aka_register_2: Register, size: Size, signed: BitwiseLogicType) {
    assert_eq!(cl_aka_register_2.to_native_8(), code_asm::cl);
    assembler.mov(code_asm::cl, a.to_native_8()).unwrap();
    match signed {
        BitwiseLogicType::Arithmetic => match size {
            Size::Byte => assembler.sar(res.to_native_8(), cl).unwrap(),
            Size::X86Word => assembler.sar(res.to_native_16(), cl).unwrap(),
            Size::X86DWord => assembler.sar(res.to_native_32(), cl).unwrap(),
            Size::X86QWord => assembler.sar(res.to_native_64(), cl).unwrap(),
        }
        BitwiseLogicType::Logical => match size {
            Size::Byte => assembler.shr(res.to_native_8(), cl).unwrap(),
            Size::X86Word => assembler.shr(res.to_native_16(), cl).unwrap(),
            Size::X86DWord => assembler.shr(res.to_native_32(), cl).unwrap(),
            Size::X86QWord => assembler.shr(res.to_native_64(), cl).unwrap(),
        }
    }
}

pub fn binary_bit_or(assembler: &mut CodeAssembler, res: Register, a: Register, size: Size) {
    match size {
        Size::Byte => assembler.or(res.to_native_8(), a.to_native_8()).unwrap(),
        Size::X86Word => assembler.or(res.to_native_16(), a.to_native_16()).unwrap(),
        Size::X86DWord => assembler.or(res.to_native_32(), a.to_native_32()).unwrap(),
        Size::X86QWord => assembler.or(res.to_native_64(), a.to_native_64()).unwrap(),
    }
}

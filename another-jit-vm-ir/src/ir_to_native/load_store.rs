use iced_x86::code_asm::{byte_ptr, CodeAssembler, dword_ptr, qword_ptr, rbp, word_ptr};
use another_jit_vm::{FramePointerOffset, Register};
use crate::Size;

pub fn ir_store(assembler: &mut CodeAssembler, from: Register, to_address: Register, size: Size) {
//todo in future will need to make size actually respected here and not zx
    match size {
        Size::Byte => assembler.mov(byte_ptr(to_address.to_native_64()), from.to_native_64()).unwrap(),
        Size::X86Word => assembler.mov(word_ptr(to_address.to_native_64()), from.to_native_64()).unwrap(),
        Size::X86DWord => assembler.mov(dword_ptr(to_address.to_native_64()), from.to_native_64()).unwrap(),
        Size::X86QWord => assembler.mov(qword_ptr(to_address.to_native_64()), from.to_native_64()).unwrap(),
    }
}

pub fn ir_load(assembler: &mut CodeAssembler, to: Register, from_address: Register, size: Size) {
    assembler.sub(to.to_native_64(), to.to_native_64()).unwrap();
    match size {
        Size::Byte => {
            assembler.mov(to.to_native_8(), from_address.to_native_64() + 0i32).unwrap();
        }
        Size::X86Word => assembler.mov(to.to_native_16(), from_address.to_native_64() + 0i32).unwrap(),
        Size::X86DWord => assembler.mov(to.to_native_32(), from_address.to_native_64() + 0i32).unwrap(),
        Size::X86QWord => assembler.mov(to.to_native_64(), from_address.to_native_64() + 0i32).unwrap(),
    }
}

pub fn ir_store_fp_relative(assembler: &mut CodeAssembler, from: Register, to: FramePointerOffset, size: Size) {
    match size {
        Size::Byte => {
            assembler.mov(byte_ptr(rbp - to.0), from.to_native_8()).unwrap()
        }
        Size::X86Word => assembler.mov(rbp - to.0, from.to_native_16()).unwrap(),
        Size::X86DWord => assembler.mov(rbp - to.0, from.to_native_32()).unwrap(),
        Size::X86QWord => assembler.mov(rbp - to.0, from.to_native_64()).unwrap(),
    }
}

pub fn ir_load_fp_relative(assembler: &mut CodeAssembler, from: FramePointerOffset, to: Register, size: Size) {
    assembler.sub(to.to_native_64(), to.to_native_64()).unwrap();
    match size {
        Size::Byte => {
            assembler.mov(to.to_native_8(), rbp - from.0).unwrap();
        }
        Size::X86Word => assembler.mov(to.to_native_16(), rbp - from.0).unwrap(),
        Size::X86DWord => assembler.mov(to.to_native_32(), rbp - from.0).unwrap(),
        Size::X86QWord => assembler.mov(to.to_native_64(), rbp - from.0).unwrap()
    }
}


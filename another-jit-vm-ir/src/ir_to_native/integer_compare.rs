use iced_x86::code_asm::{al, ax, CodeAssembler, eax, rax, rbx, rcx, rdx};
use another_jit_vm::Register;
use crate::{Signed, Size};

pub fn int_compare(assembler: &mut CodeAssembler, res: Register, value1: Register, value2: Register, temp1: Register, temp2: Register, temp3: Register, size: Size) {
    match size {
        Size::Byte => assembler.cmp(value1.to_native_8(), value2.to_native_8()).unwrap(),
        Size::X86Word => assembler.cmp(value1.to_native_16(), value2.to_native_16()).unwrap(),
        Size::X86DWord => assembler.cmp(value1.to_native_32(), value2.to_native_32()).unwrap(),
        Size::X86QWord => assembler.cmp(value1.to_native_64(), value2.to_native_64()).unwrap(),
    }
    assembler.mov(res.to_native_64(), 0u64).unwrap();
    assembler.mov(temp1.to_native_64(), 1u64).unwrap();
    assembler.mov(temp2.to_native_64(), 0u64).unwrap();
    assembler.mov(temp3.to_native_64(), -1i64).unwrap();
    assembler.cmovg(res.to_native_64(), temp1.to_native_64()).unwrap();
    assembler.cmove(res.to_native_64(), temp2.to_native_64()).unwrap();
    assembler.cmovl(res.to_native_64(), temp3.to_native_64()).unwrap();
}


pub fn div_rem_common(assembler: &mut CodeAssembler, res: Register, divisor: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register, size: Size, signed: &Signed) {
    assert_eq!(must_be_rax.0, 0);
    assert_eq!(must_be_rdx.to_native_64(), rdx);
    assert_eq!(must_be_rbx.to_native_64(), rbx);
    assert_eq!(must_be_rcx.to_native_64(), rcx);
    assembler.sub(rax, rax).unwrap();
    match size {
        Size::Byte => assembler.mov(al, res.to_native_8()).unwrap(),
        Size::X86Word => assembler.mov(ax, res.to_native_16()).unwrap(),
        Size::X86DWord => assembler.mov(eax, res.to_native_32()).unwrap(),
        Size::X86QWord => assembler.mov(rax, res.to_native_64()).unwrap(),
    }
    assembler.mov(rbx, 0u64).unwrap();
    assembler.mov(rcx, 0u64).unwrap();
    assembler.mov(rdx, 0u64).unwrap();
    match signed {
        Signed::Signed => {
            match size {
                Size::Byte => {
                    // assembler.idiv(divisor.to_native_8()).unwrap()
                    todo!()
                }
                Size::X86Word => {
                    // assembler.idiv(divisor.to_native_16()).unwrap()
                    todo!()
                }
                Size::X86DWord => {
                    assembler.cdq().unwrap();
                    assembler.idiv(divisor.to_native_32()).unwrap()
                }
                Size::X86QWord => {
                    assembler.cqo().unwrap();
                    assembler.idiv(divisor.to_native_64()).unwrap()
                }
            }
        }
        Signed::Unsigned => {
            match size {
                Size::Byte => assembler.div(divisor.to_native_8()).unwrap(),
                Size::X86Word => assembler.div(divisor.to_native_16()).unwrap(),
                Size::X86DWord => assembler.div(divisor.to_native_32()).unwrap(),
                Size::X86QWord => assembler.div(divisor.to_native_64()).unwrap(),
            }
        }
    }
}

pub fn sized_integer_compare(assembler: &mut CodeAssembler, a: Register, b: Register, size: Size) {
    match size {
        Size::Byte => todo!(),
        Size::X86Word => todo!(),
        Size::X86DWord => assembler.cmp(a.to_native_32(), b.to_native_32()).unwrap(),
        Size::X86QWord => assembler.cmp(a.to_native_64(), b.to_native_64()).unwrap(),
    }
}



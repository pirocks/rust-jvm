use iced_x86::code_asm::{al, ax, CodeAssembler, dl, dx, eax, edx, rax, rbx, rcx, rdx};

use another_jit_vm::Register;

use crate::{Signed, Size};
use crate::ir_to_native::integer_compare::div_rem_common;

pub fn ir_add(assembler: &mut CodeAssembler, res: Register, a: Register, size: Size) {
    match size {
        Size::Byte => assembler.add(res.to_native_8(), a.to_native_8()).unwrap(),
        Size::X86Word => assembler.add(res.to_native_16(), a.to_native_16()).unwrap(),
        Size::X86DWord => assembler.add(res.to_native_32(), a.to_native_32()).unwrap(),
        Size::X86QWord => assembler.add(res.to_native_64(), a.to_native_64()).unwrap(),
    }
}

pub fn ir_sub(assembler: &mut CodeAssembler, res: Register, to_subtract: Register, size: Size) {
    match size {
        Size::Byte => assembler.sub(res.to_native_8(), to_subtract.to_native_8()).unwrap(),
        Size::X86Word => assembler.sub(res.to_native_16(), to_subtract.to_native_16()).unwrap(),
        Size::X86DWord => assembler.sub(res.to_native_32(), to_subtract.to_native_32()).unwrap(),
        Size::X86QWord => assembler.sub(res.to_native_64(), to_subtract.to_native_64()).unwrap(),
    }
}

pub fn ir_div(assembler: &mut CodeAssembler, res: Register, divisor: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register, size: Size, signed: &Signed) {
    div_rem_common(assembler, res, divisor, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed);
    match size {
        Size::Byte => assembler.mov(res.to_native_8(), al).unwrap(),
        Size::X86Word => assembler.mov(res.to_native_16(), ax).unwrap(),
        Size::X86DWord => assembler.mov(res.to_native_32(), eax).unwrap(),
        Size::X86QWord => assembler.mov(res.to_native_64(), rax).unwrap(),
    }
}

pub fn ir_mod(assembler: &mut CodeAssembler, res: Register, divisor: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register, size: Size, signed: &Signed) {
    div_rem_common(assembler, res, divisor, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed);
    match size {
        Size::Byte => assembler.mov(res.to_native_8(), dl).unwrap(),
        Size::X86Word => assembler.mov(res.to_native_16(), dx).unwrap(),
        Size::X86DWord => assembler.mov(res.to_native_32(), edx).unwrap(),
        Size::X86QWord => assembler.mov(res.to_native_64(), rdx).unwrap(),
    }
}


pub fn mul(assembler: &mut CodeAssembler, res: Register, a: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register, size: Size, signed: &Signed) {
    assert_eq!(must_be_rax.0, 0);
    assert_eq!(must_be_rdx.to_native_64(), rdx);
    assert_eq!(must_be_rbx.to_native_64(), rbx);
    assert_eq!(must_be_rcx.to_native_64(), rcx);
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
                Size::Byte => assembler.imul(a.to_native_8()).unwrap(),
                Size::X86Word => assembler.imul(a.to_native_16()).unwrap(),
                Size::X86DWord => assembler.imul(a.to_native_32()).unwrap(),
                Size::X86QWord => assembler.imul(a.to_native_64()).unwrap(),
            }
        }
        Signed::Unsigned => {
            match size {
                Size::Byte => assembler.mul(a.to_native_8()).unwrap(),
                Size::X86Word => assembler.mul(a.to_native_16()).unwrap(),
                Size::X86DWord => assembler.mul(a.to_native_32()).unwrap(),
                Size::X86QWord => assembler.mul(a.to_native_64()).unwrap(),
            }
        }
    }
    match size {
        Size::Byte => assembler.mov(res.to_native_8(), al).unwrap(),
        Size::X86Word => assembler.mov(res.to_native_16(), ax).unwrap(),
        Size::X86DWord => assembler.mov(res.to_native_32(), eax).unwrap(),
        Size::X86QWord => assembler.mov(res.to_native_64(), rax).unwrap(),
    }
}


pub fn mul_const(assembler: &mut CodeAssembler, res: Register, a: &i32, size: Size, signed: &Signed) {
    match signed {
        Signed::Signed => {
            match size {
                Size::Byte => todo!(),
                Size::X86Word => todo!(),
                Size::X86DWord => todo!(),
                Size::X86QWord => assembler.imul_3(res.to_native_64(), res.to_native_64(), *a).unwrap(),
            }
        }
        Signed::Unsigned => {
            match size {
                Size::Byte => todo!(),
                Size::X86Word => todo!(),
                Size::X86DWord => todo!(),
                Size::X86QWord => todo!()/*assembler.imul_3(res.to_native_64(), res.to_native_64(), *a).unwrap()*/,
            }
        }
    }
}

pub fn sign_extend(assembler: &mut CodeAssembler, from: Register, to: Register, from_size: Size, to_size: Size) {
    match from_size {
        Size::Byte => match to_size {
            Size::Byte => {
                todo!()
            }
            Size::X86Word => assembler.movsx(to.to_native_16(), from.to_native_8()).unwrap(),
            Size::X86DWord => assembler.movsx(to.to_native_32(), from.to_native_8()).unwrap(),
            Size::X86QWord => assembler.movsx(to.to_native_64(), from.to_native_8()).unwrap(),
        },
        Size::X86Word => match to_size {
            Size::Byte => {
                todo!()
            }
            Size::X86Word => {
                todo!()
            }
            Size::X86DWord => assembler.movsx(to.to_native_32(), from.to_native_16()).unwrap(),
            Size::X86QWord => assembler.movsx(to.to_native_64(), from.to_native_16()).unwrap()
        },
        Size::X86DWord => match to_size {
            Size::Byte => {
                todo!()
            }
            Size::X86Word => {
                todo!()
            }
            Size::X86DWord => {
                assembler.nop().unwrap();
            }
            Size::X86QWord => assembler.movsxd(to.to_native_64(), from.to_native_32()).unwrap()
        },
        Size::X86QWord => {
            match to_size {
                Size::Byte => {
                    todo!()
                }
                Size::X86Word => {
                    todo!()
                }
                Size::X86DWord => {
                    todo!()
                }
                Size::X86QWord => {
                    assembler.nop().unwrap();
                }
            }
        }
    };
}

pub fn zero_extend(assembler: &mut CodeAssembler, from: Register, to: Register, from_size: Size, to_size: Size) {
    match from_size {
        Size::Byte => match to_size {
            Size::Byte => {
                todo!()
            }
            Size::X86Word => assembler.movzx(to.to_native_16(), from.to_native_8()).unwrap(),
            Size::X86DWord => assembler.movzx(to.to_native_32(), from.to_native_8()).unwrap(),
            Size::X86QWord => assembler.movzx(to.to_native_64(), from.to_native_8()).unwrap(),
        },
        Size::X86Word => match to_size {
            Size::Byte => {
                todo!()
            }
            Size::X86Word => {
                todo!()
            }
            Size::X86DWord => assembler.movzx(to.to_native_32(), from.to_native_16()).unwrap(),
            Size::X86QWord => assembler.movzx(to.to_native_64(), from.to_native_16()).unwrap()
        },
        Size::X86DWord => match to_size {
            Size::Byte => {
                todo!()
            }
            Size::X86Word => {
                todo!()
            }
            Size::X86DWord => {
                todo!()
            }
            Size::X86QWord => assembler.mov(to.to_native_32(), from.to_native_32()).unwrap()//mov zeros the upper in register
        },
        Size::X86QWord => {
            todo!()
        }
    };
}
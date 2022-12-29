use std::ffi::c_void;
use iced_x86::code_asm::{al, ax, CodeAssembler, qword_ptr, r13, r14, r15, r8, r9, rax, rbp, rcx, rdi, rdx, rsi, rsp, xmm0, xmm1, xmm2, xmm3, xmm4, xmm5, xmm6, xmm7};
use memoffset::offset_of;
use nonnull_const::NonNullConst;
use another_jit_vm::{DoubleRegister, FloatRegister, FramePointerOffset, Register};
use another_jit_vm::intrinsic_helpers::IntrinsicHelperType;
use crate::compiler::Size;
use another_jit_vm::JITContext;

pub(crate) fn call_native_helper(assembler: &mut CodeAssembler, to_call: &NonNullConst<c_void>, integer_args: &Vec<FramePointerOffset>, byte_res: &Option<FramePointerOffset>, bool_res: &Option<FramePointerOffset>, char_res: &Option<FramePointerOffset>, short_res: &Option<FramePointerOffset>, integer_res: &Option<FramePointerOffset>, float_double_args: &Vec<(FramePointerOffset, Size)>, float_res: &Option<FramePointerOffset>, double_res: &Option<FramePointerOffset>) {
    let mut integer_args = integer_args.iter();
    if let Some(arg) = integer_args.next() {
        assembler.mov(rdi, rbp - arg.0).unwrap();
    }
    if let Some(arg) = integer_args.next() {
        assembler.mov(rsi, rbp - arg.0).unwrap();
    }
    if let Some(arg) = integer_args.next() {
        assembler.mov(rdx, rbp - arg.0).unwrap();
    }
    if let Some(arg) = integer_args.next() {
        assembler.mov(rcx, rbp - arg.0).unwrap();
    }
    if let Some(arg) = integer_args.next() {
        assembler.mov(r8, rbp - arg.0).unwrap();
    }
    if let Some(arg) = integer_args.next() {
        assembler.mov(r9, rbp - arg.0).unwrap();
    }
    if let Some(_) = integer_args.next() {
        todo!();
    }
    let mut float_double_args = float_double_args.iter();
    let mut sse_registers = vec![xmm0, xmm1, xmm2, xmm3, xmm4, xmm5, xmm6, xmm7].into_iter();
    loop {
        if let Some((arg, size)) = float_double_args.next() {
            let sse_register = sse_registers.next().unwrap();
            match size {
                Size::X86DWord => {
                    assembler.movd(sse_register, rbp - arg.0).unwrap();
                }
                Size::X86QWord => {
                    assembler.movq(sse_register, rbp - arg.0).unwrap();
                }
                _ => {
                    panic!()
                }
            }
        } else {
            break;
        }
    }
    //todo interrupts will need to look at top of this stack for actual rbp and rsp
    assembler.mov(rax, to_call.as_ptr() as i64).unwrap();
    let old_rsp = r14;
    let old_rbp = r13;
    assembler.mov(old_rbp, rbp).unwrap();
    assembler.mov(old_rsp, rsp).unwrap();
    assembler.mov(rbp, r15 + offset_of!(JITContext, alt_native_rbp)).unwrap();
    assembler.mov(rsp, r15 + offset_of!(JITContext, alt_native_rsp)).unwrap();
    assembler.push(old_rsp).unwrap();
    assembler.push(old_rbp).unwrap();
    assembler.push(r15).unwrap();//push twice to maintain stack alignment
    assembler.push(r15).unwrap();
    assembler.call(rax).unwrap();
    assembler.pop(r15).unwrap();
    assembler.pop(r15).unwrap();
    assembler.pop(rbp).unwrap();
    assembler.pop(rsp).unwrap();
    if let Some(byte_res) = byte_res {
        assembler.movsx(rax, al).unwrap();
        assembler.mov(rbp - byte_res.0, rax).unwrap();
    }

    if let Some(bool_res) = bool_res {
        assembler.movzx(rax, al).unwrap();
        assembler.mov(rbp - bool_res.0, rax).unwrap();
    }

    if let Some(short_res) = short_res {
        assembler.movsx(rax, ax).unwrap();
        assembler.mov(rbp - short_res.0, rax).unwrap();
    }

    if let Some(char_res) = char_res {
        assembler.movzx(rax, ax).unwrap();
        assembler.mov(rbp - char_res.0, rax).unwrap();
    }

    if let Some(integer_res) = integer_res {
        assembler.mov(rbp - integer_res.0, rax).unwrap();
    }

    if let Some(float_res) = float_res {
        assembler.movd(rbp - float_res.0, xmm0).unwrap();
    }

    if let Some(double_res) = double_res {
        assembler.movq(rbp - double_res.0, xmm0).unwrap();
    }
}


pub(crate) fn ir_call_intrinsic_helper(assembler: &mut CodeAssembler, intrinsic_helper_type: IntrinsicHelperType, integer_args: &Vec<Register>, integer_res: Option<Register>, float_args: &Vec<FloatRegister>, float_res: &Option<FloatRegister>, double_args: &Vec<DoubleRegister>, double_res: &Option<DoubleRegister>) {
    match intrinsic_helper_type {
        IntrinsicHelperType::Memmove => {
            let first_arg = rdi;
            let second_arg = rsi;
            let third_arg = rdx;
            assert_eq!(integer_args.len(), 3);
            let args = vec![first_arg, second_arg, third_arg];
            assert!(!integer_args.iter().any(|reg| args.contains(&reg.to_native_64())));
            assert!(integer_args.len() <= args.len());
            for (from_arg, to_arg) in integer_args.iter().zip(args.iter()) {
                assembler.mov(*to_arg, from_arg.to_native_64()).unwrap();
            }
            assembler.and(rsp, -32).unwrap();//align stack pointer
            assembler.call(qword_ptr(r15 + intrinsic_helper_type.r15_offset())).unwrap();
        }
        IntrinsicHelperType::Malloc => {
            one_integer_in_and_one_out(assembler, intrinsic_helper_type, integer_args, integer_res);
        }
        IntrinsicHelperType::Free => {
            let first_arg = rdi;
            assert_eq!(integer_args.len(), 1);
            let args = vec![first_arg];
            assert!(!integer_args.iter().any(|reg| args.contains(&reg.to_native_64())));
            assert!(integer_args.len() <= args.len());
            for (from_arg, to_arg) in integer_args.iter().zip(args.iter()) {
                assembler.mov(*to_arg, from_arg.to_native_64()).unwrap();
            }
            assembler.and(rsp, -32).unwrap();//align stack pointer
            assembler.call(qword_ptr(r15 + intrinsic_helper_type.r15_offset())).unwrap();
            assert!(integer_res.is_none());
        }
        IntrinsicHelperType::FRemF => {
            let first_arg = xmm0;
            let second_arg = xmm1;
            assert_eq!(float_args.len(), 2);
            let args = vec![first_arg, second_arg];
            assert!(!float_args.iter().any(|reg| args.contains(&reg.to_xmm())));
            assert!(float_args.len() <= args.len());
            for (from_arg, to_arg) in float_args.iter().zip(args.iter()) {
                assembler.movdqa(*to_arg, from_arg.to_xmm()).unwrap();
            }
            assert!(integer_args.is_empty());
            assert!(double_args.is_empty());
            assembler.and(rsp, -32).unwrap();//align stack pointer
            assembler.call(qword_ptr(r15 + intrinsic_helper_type.r15_offset())).unwrap();
            let float_res = float_res.unwrap();
            assembler.movdqa(float_res.to_xmm(), xmm0).unwrap();
        }
        IntrinsicHelperType::InstanceOf => todo!(),
        IntrinsicHelperType::DRemD => {
            let first_arg = xmm0;
            let second_arg = xmm1;
            assert_eq!(double_args.len(), 2);
            let args = vec![first_arg, second_arg];
            assert!(!double_args.iter().any(|reg| args.contains(&reg.to_xmm())));
            assert!(double_args.len() <= args.len());
            for (from_arg, to_arg) in double_args.iter().zip(args.iter()) {
                assembler.movdqa(*to_arg, from_arg.to_xmm()).unwrap();
            }
            assert!(integer_args.is_empty());
            assert!(float_args.is_empty());
            assembler.and(rsp, -32).unwrap();//align stack pointer
            assembler.call(qword_ptr(r15 + intrinsic_helper_type.r15_offset())).unwrap();
            let double_res = double_res.unwrap();
            assembler.movdqa(double_res.to_xmm(), xmm0).unwrap();
        }
        IntrinsicHelperType::GetConstantAllocation => todo!(),
        IntrinsicHelperType::FindVTablePtr |
        IntrinsicHelperType::FindITablePtr |
        IntrinsicHelperType::FindObjectHeader |
        IntrinsicHelperType::FindClassPtr => {
            one_integer_in_and_one_out(assembler, intrinsic_helper_type, integer_args, integer_res);
        }
    }
}

fn one_integer_in_and_one_out(assembler: &mut CodeAssembler, intrinsic_helper_type: IntrinsicHelperType, integer_args: &Vec<Register>, integer_res: Option<Register>) {
    let first_arg = rdi;
    assert_eq!(integer_args.len(), 1);
    let args = vec![first_arg];
    assert!(!integer_args.iter().any(|reg| args.contains(&reg.to_native_64())));
    assert!(integer_args.len() <= args.len());
    for (from_arg, to_arg) in integer_args.iter().zip(args.iter()) {
        assembler.mov(*to_arg, from_arg.to_native_64()).unwrap();
    }
    assembler.and(rsp, -32).unwrap();//align stack pointer
    assembler.call(qword_ptr(r15 + intrinsic_helper_type.r15_offset())).unwrap();
    let integer_res = integer_res.unwrap();
    assembler.mov(integer_res.to_native_64(), rax).unwrap();
}

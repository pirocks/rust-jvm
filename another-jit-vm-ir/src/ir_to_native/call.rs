use std::mem::size_of;
use iced_x86::code_asm::{CodeAssembler, qword_ptr, rax, rbp, rsp};
use another_jit_vm::{FramePointerOffset, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED, Register};
use gc_memory_layout_common::layout::FrameHeader;
use crate::{FRAME_HEADER_IR_METHOD_ID_OFFSET, FRAME_HEADER_METHOD_ID_OFFSET, FRAME_HEADER_PREV_MAGIC_1_OFFSET, FRAME_HEADER_PREV_MAGIC_2_OFFSET, FRAME_HEADER_PREV_RBP_OFFSET, FRAME_HEADER_PREV_RIP_OFFSET, IRCallTarget};

pub fn ir_return(assembler: &mut CodeAssembler, return_val: Option<Register>, temp_register_1: Register, temp_register_2: Register, temp_register_3: Register, temp_register_4: Register, frame_size: &usize) {
    if let Some(return_register) = return_val {
        assert_ne!(temp_register_1.to_native_64(), rax);
        assert_ne!(temp_register_2.to_native_64(), rax);
        assert_ne!(temp_register_3.to_native_64(), rax);
        assert_ne!(temp_register_4.to_native_64(), rax);
        assembler.mov(rax, return_register.to_native_64()).unwrap();
    }
    //load prev frame pointer
    assembler.mov(temp_register_1.to_native_64(), rbp - FRAME_HEADER_PREV_RIP_OFFSET).unwrap();
    assembler.mov(rbp, rbp - FRAME_HEADER_PREV_RBP_OFFSET).unwrap();
    assembler.add(rsp, *frame_size as i32).unwrap();
    assembler.jmp(temp_register_1.to_native_64()).unwrap();
}

pub fn ir_call(assembler: &mut CodeAssembler, temp_register_1: Register, temp_register_2: Register, arg_from_to_offsets: &Vec<(FramePointerOffset, FramePointerOffset)>, return_value: &Option<FramePointerOffset>, target_address: &IRCallTarget, current_frame_size: &usize) {
    assert!(*current_frame_size >= size_of::<FrameHeader>());
    let temp_register = temp_register_1.to_native_64();
    let return_to_rbp = temp_register_2.to_native_64();
    let mut after_call_label = assembler.create_label();
    assembler.mov(return_to_rbp, rbp).unwrap();
    assembler.sub(rbp, *current_frame_size as i32).unwrap();
    match target_address {
        IRCallTarget::Constant { new_frame_size, .. } => {
            assembler.sub(rsp, *new_frame_size as i32).unwrap();
        }
        IRCallTarget::Variable { new_frame_size, .. } => {
            assembler.sub(rsp, new_frame_size.to_native_64()).unwrap();
        }
    }
    for (from, to) in arg_from_to_offsets {
        assembler.mov(temp_register, return_to_rbp - from.0).unwrap();
        assembler.mov(rbp - to.0, temp_register).unwrap();
    }

    assembler.mov(temp_register, MAGIC_1_EXPECTED).unwrap();
    assembler.mov(rbp - (FRAME_HEADER_PREV_MAGIC_1_OFFSET) as u64, temp_register).unwrap();
    assembler.mov(temp_register, MAGIC_2_EXPECTED).unwrap();
    assembler.mov(rbp - (FRAME_HEADER_PREV_MAGIC_2_OFFSET) as u64, temp_register).unwrap();
    assembler.mov(rbp - (FRAME_HEADER_PREV_RBP_OFFSET) as u64, return_to_rbp).unwrap();
    match target_address {
        IRCallTarget::Constant { method_id, .. } => {
            assembler.mov(temp_register, *method_id as u64).unwrap();
        }
        IRCallTarget::Variable { method_id, .. } => {
            assembler.mov(temp_register, method_id.to_native_64()).unwrap();
        }
    }
    assembler.mov(rbp - (FRAME_HEADER_METHOD_ID_OFFSET) as u64, temp_register).unwrap();
    match target_address {
        IRCallTarget::Constant { ir_method_id, .. } => {
            assembler.mov(temp_register, ir_method_id.0 as u64).unwrap();
        }
        IRCallTarget::Variable { ir_method_id, .. } => {
            assembler.mov(temp_register, ir_method_id.to_native_64()).unwrap();
        }
    }
    assembler.mov(rbp - (FRAME_HEADER_IR_METHOD_ID_OFFSET) as u64, temp_register).unwrap();

    let return_to_rip = temp_register_2.to_native_64();
    assembler.lea(return_to_rip, qword_ptr(after_call_label.clone())).unwrap();
    assembler.mov(rbp - (FRAME_HEADER_PREV_RIP_OFFSET) as u64, return_to_rip).unwrap();
    match target_address {
        IRCallTarget::Constant { address, .. } => {
            assembler.mov(temp_register, *address as u64).unwrap();
        }
        IRCallTarget::Variable { address, .. } => {
            assembler.mov(temp_register, address.to_native_64()).unwrap();
        }
    }
    assembler.jmp(temp_register).unwrap();
    assembler.set_label(&mut after_call_label).unwrap();
    if let Some(return_value) = return_value {
        assembler.mov(rbp - return_value.0, rax).unwrap();
    }
}

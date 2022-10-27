use std::mem::size_of;

use iced_x86::code_asm::{CodeAssembler, qword_ptr, rax, rbp, rsp};

use another_jit_vm::{FramePointerOffset, IRMethodID, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED, Register};
use another_jit_vm::code_modification::{AssemblerFunctionCallTarget, AssemblerRuntimeModificationTarget};
use gc_memory_layout_common::frame_layout::{FRAME_HEADER_IR_METHOD_ID_OFFSET, FRAME_HEADER_METHOD_ID_OFFSET, FRAME_HEADER_PREV_MAGIC_1_OFFSET, FRAME_HEADER_PREV_MAGIC_2_OFFSET, FRAME_HEADER_PREV_RBP_OFFSET, FRAME_HEADER_PREV_RIP_OFFSET, FrameHeader};
use rust_jvm_common::MethodId;

use crate::IRCallTarget;

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

pub fn ir_function_start(assembler: &mut CodeAssembler, temp_register: Register, ir_method_id: IRMethodID, method_id: MethodId, frame_size: usize, num_locals: usize) {
    assembler.mov(temp_register.to_native_64(), 0xeeee_eeee_eeee_eeeeu64).unwrap();
    for i in (size_of::<FrameHeader>() / 8 + num_locals)..(frame_size / 8) {
        assembler.mov(rbp - i * 8, temp_register.to_native_64()).unwrap()
    }
    assembler.mov(temp_register.to_native_64(), method_id as u64).unwrap();
    assembler.mov(rbp - FRAME_HEADER_METHOD_ID_OFFSET as u64, temp_register.to_native_64()).unwrap();
    assembler.mov(temp_register.to_native_64(), ir_method_id.0 as u64).unwrap();
    assembler.mov(rbp - FRAME_HEADER_IR_METHOD_ID_OFFSET as u64, temp_register.to_native_64()).unwrap();
    assembler.lea(rsp, rbp - frame_size).unwrap();
    //these must be last for signal stacktracing mechanism to only see completed frames.
    assembler.mov(temp_register.to_native_64(), MAGIC_1_EXPECTED).unwrap();
    assembler.mov(rbp - FRAME_HEADER_PREV_MAGIC_1_OFFSET as u64, temp_register.to_native_64()).unwrap();
    assembler.mov(temp_register.to_native_64(), MAGIC_2_EXPECTED).unwrap();
    assembler.mov(rbp - FRAME_HEADER_PREV_MAGIC_2_OFFSET as u64, temp_register.to_native_64()).unwrap();
}

pub fn ir_call(assembler: &mut CodeAssembler, temp_register_1: Register, temp_register_2: Register, arg_from_to_offsets: &Vec<(FramePointerOffset, FramePointerOffset)>, return_value: Option<FramePointerOffset>, target_address: IRCallTarget, current_frame_size: usize) -> Option<AssemblerFunctionCallTarget> {
    assert!(current_frame_size >= size_of::<FrameHeader>());
    let temp_register = temp_register_1.to_native_64();
    let return_to_rbp = temp_register_2.to_native_64();
    let mut after_call_label = assembler.create_label();
    assembler.mov(return_to_rbp, rbp).unwrap();
    //todo bug b/c rbp could have valid magic but invalid frame
    assembler.lea(temp_register, rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_1_OFFSET) as i32).unwrap();
    assembler.mov(qword_ptr(temp_register), 0i32).unwrap();
    assembler.lea(temp_register, rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_2_OFFSET) as i32).unwrap();
    assembler.mov(qword_ptr(temp_register), 0i32).unwrap();
    assembler.sub(rbp, current_frame_size as i32).unwrap();
    let max_offset = arg_from_to_offsets.iter().map(|(_, to)| to.0).max().unwrap_or(0);
    assembler.mov(rbp - FRAME_HEADER_PREV_RBP_OFFSET as u64, return_to_rbp).unwrap();
    //so that we don't get red zoned
    assembler.sub(rsp, max_offset as i32).unwrap();
    for (from, to) in arg_from_to_offsets {
        assembler.mov(temp_register, return_to_rbp - from.0).unwrap();
        assembler.mov(rbp - to.0, temp_register).unwrap();
    }

    let return_to_rip = temp_register_2.to_native_64();
    assembler.lea(return_to_rip, qword_ptr(after_call_label.clone())).unwrap();
    assembler.mov(rbp - FRAME_HEADER_PREV_RIP_OFFSET as u64, return_to_rip).unwrap();
    let mov_position_and_method_id = match target_address {
        IRCallTarget::Constant { address, method_id, } => {
            let mov_position = assembler.instructions().len();
            assembler.mov(temp_register, address as u64).unwrap();
            Some((mov_position, method_id))
        }
        IRCallTarget::Variable { address, .. } => {
            assembler.mov(temp_register, address.to_native_64()).unwrap();
            None
        }
    };
    assembler.jmp(temp_register).unwrap();
    assembler.set_label(&mut after_call_label).unwrap();
    if let Some(return_value) = return_value {
        assembler.mov(rbp - return_value.0, rax).unwrap();
    }
    let (mov_position, method_id) = mov_position_and_method_id?;
    let modification_target = AssemblerRuntimeModificationTarget::MovQ { instruction_number: mov_position };
    Some(AssemblerFunctionCallTarget { modification_target, method_id })
}

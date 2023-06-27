use std::mem::size_of;
use std::ptr::NonNull;
use iced_x86::code_asm::{CodeAssembler, dword_ptr, rbp, ymm0, ymm1, ymm2, ymm4};
use memoffset::offset_of;
use another_jit_vm::{FramePointerOffset, Register, VMState};
use another_jit_vm::intrinsic_helpers::IntrinsicHelperType;
use inheritance_tree::ClassID;
use inheritance_tree::paths::BitPath256;
use crate::vm_exit_abi::IRVMExitType;
use gc_memory_layout_common::memory_regions::RegionHeader;
use crate::ir_to_native::native_call::ir_call_intrinsic_helper;

pub(crate) fn instance_of_interface(assembler: &mut CodeAssembler, target_interface_id: &ClassID, object_ref: &FramePointerOffset, return_val: &Register) {
    let obj_ptr = Register(0);
    let mut instance_of_succeed = assembler.create_label();
    let mut instance_of_fail = assembler.create_label();
    assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
    assembler.cmp(obj_ptr.to_native_64(), 0).unwrap();
    assembler.je(instance_of_fail).unwrap();
    let interface_list_base_pointer = Register(3);
    let interface_list_base_pointer_len = Register(5);
    ir_call_intrinsic_helper(assembler, IntrinsicHelperType::FindObjectHeader, &vec![obj_ptr], Some(interface_list_base_pointer), &vec![], &None, &vec![], &None);
    assembler.mov(interface_list_base_pointer_len.to_native_64(), interface_list_base_pointer.to_native_64() + offset_of!(RegionHeader,interface_ids_list_len)).unwrap();
    assembler.mov(interface_list_base_pointer.to_native_64(), interface_list_base_pointer.to_native_64() + offset_of!(RegionHeader,interface_ids_list)).unwrap();
    assembler.lea(interface_list_base_pointer_len.to_native_64(), interface_list_base_pointer.to_native_64() + interface_list_base_pointer_len.to_native_64() * size_of::<ClassID>()).unwrap();
    let mut loop_ = assembler.create_label();
    assembler.set_label(&mut loop_).unwrap();
    assembler.cmp(interface_list_base_pointer.to_native_64(), interface_list_base_pointer_len.to_native_64()).unwrap();
    assembler.je(instance_of_fail).unwrap();
    assembler.cmp(dword_ptr(interface_list_base_pointer.to_native_64()), target_interface_id.0).unwrap();
    assembler.je(instance_of_succeed).unwrap();
    assembler.lea(interface_list_base_pointer.to_native_64(), interface_list_base_pointer.to_native_64() + size_of::<ClassID>() as u64).unwrap();
    assembler.jmp(loop_).unwrap();
    let mut done = assembler.create_label();
    assembler.set_label(&mut instance_of_succeed).unwrap();
    assembler.mov(return_val.to_native_64(), 1u64).unwrap();
    assembler.jmp(done).unwrap();
    assembler.set_label(&mut instance_of_fail).unwrap();
    assembler.mov(return_val.to_native_64(), 0u64).unwrap();
    assembler.jmp(done).unwrap();
    assembler.set_label(&mut done).unwrap();
    assembler.nop().unwrap();
}

pub(crate) fn instance_of_class(assembler: &mut CodeAssembler, inheritance_path: &NonNull<BitPath256>, object_ref: &FramePointerOffset, return_val: &Register, instance_of_exit: &IRVMExitType) {
    let mut instance_of_exit_label = assembler.create_label();
    let mut instance_of_succeed = assembler.create_label();
    let mut instance_of_fail = assembler.create_label();
    let obj_ptr = Register(0);
    assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
    assembler.cmp(obj_ptr.to_native_64(), 0).unwrap();
    assembler.je(instance_of_fail).unwrap();
    let object_inheritance_path_pointer = Register(3);
    ir_call_intrinsic_helper(assembler, IntrinsicHelperType::FindObjectHeader, &vec![obj_ptr], Some(object_inheritance_path_pointer), &vec![], &None, &vec![], &None);
    assembler.mov(object_inheritance_path_pointer.to_native_64(), object_inheritance_path_pointer.to_native_64() + offset_of!(RegionHeader,inheritance_bit_path_ptr)).unwrap();
    assembler.cmp(object_inheritance_path_pointer.to_native_64(), 0).unwrap();
    assembler.je(instance_of_exit_label).unwrap();

    let object_bit_len_register = Register(1);
    assembler.sub(object_bit_len_register.to_native_64(), object_bit_len_register.to_native_64()).unwrap();
    assembler.mov(object_bit_len_register.to_native_8(), object_inheritance_path_pointer.to_native_64() + offset_of!(BitPath256,bit_len)).unwrap();

    let instanceof_path_pointer = Register(2);
    assembler.mov(instanceof_path_pointer.to_native_64(), inheritance_path.as_ptr() as u64).unwrap();

    let instanceof_bit_len_register = Register(4);
    assembler.sub(instanceof_bit_len_register.to_native_64(), instanceof_bit_len_register.to_native_64()).unwrap();
    assembler.mov(instanceof_bit_len_register.to_native_8(), instanceof_path_pointer.to_native_64() + offset_of!(BitPath256,bit_len)).unwrap();

    assembler.cmp(object_bit_len_register.to_native_8(), instanceof_bit_len_register.to_native_8()).unwrap();
    assembler.jl(instance_of_fail).unwrap();


    let mask_register = ymm2;
    let instance_of_bit_path = ymm1;
    let object_inheritance_bit_path = ymm0;
    assembler.vmovdqu(mask_register, instanceof_path_pointer.to_native_64() + offset_of!(BitPath256,valid_mask)).unwrap();
    assembler.vmovdqu(instance_of_bit_path, instanceof_path_pointer.to_native_64() + offset_of!(BitPath256,bit_path)).unwrap();
    assembler.vmovdqu(object_inheritance_bit_path, object_inheritance_path_pointer.to_native_64() + offset_of!(BitPath256,bit_path)).unwrap();
    let xored = ymm4;
    assembler.vpxor(xored, instance_of_bit_path, object_inheritance_bit_path).unwrap();
    assembler.vptest(xored, mask_register).unwrap();// ands xored and mask and cmp whole res with zero
    assembler.je(instance_of_succeed).unwrap();
    assembler.jmp(instance_of_fail).unwrap();
    let mut done = assembler.create_label();
    assembler.set_label(&mut instance_of_succeed).unwrap();
    assembler.mov(return_val.to_native_64(), 1u64).unwrap();
    assembler.jmp(done).unwrap();
    assembler.set_label(&mut instance_of_fail).unwrap();
    assembler.mov(return_val.to_native_64(), 0u64).unwrap();
    assembler.jmp(done).unwrap();
    assembler.set_label(&mut instance_of_exit_label).unwrap();
    match instance_of_exit {
        IRVMExitType::InstanceOf { .. } => {
            let registers = instance_of_exit.registers_to_save();
            instance_of_exit.gen_assembly(assembler, &mut done);
            let mut before_exit_label = assembler.create_label();
            VMState::<u64>::gen_vm_exit(assembler, &mut before_exit_label, &mut done, registers);
        }
        IRVMExitType::CheckCast { .. } => {
            let registers = instance_of_exit.registers_to_save();
            instance_of_exit.gen_assembly(assembler, &mut done);
            let mut before_exit_label = assembler.create_label();
            VMState::<u64>::gen_vm_exit(assembler, &mut before_exit_label, &mut done, registers);
        }
        _ => {
            panic!()
        }
    }
    // assembler.set_label(&mut done).unwrap(); //done in gen_vm_exit
    assembler.nop().unwrap();
}

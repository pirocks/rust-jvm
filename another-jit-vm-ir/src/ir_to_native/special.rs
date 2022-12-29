use std::sync::atomic::AtomicPtr;
use iced_x86::code_asm::{al, ax, CodeAssembler, eax, qword_ptr, r15, rax, rbp, rdi, rsp};

use another_jit_vm::{FramePointerOffset, Register, VMState};
use another_jit_vm::intrinsic_helpers::IntrinsicHelperType;
use gc_memory_layout_common::memory_regions::{MemoryRegions, RegionHeader};
use interface_vtable::generate_itable_access;
use rust_jvm_common::ByteCodeOffset;
use vtable::generate_vtable_access;

use crate::{gen_vm_exit, InvokeVirtualResolve, IRVMExitType};
use crate::compiler::{Size};
use crate::ir_to_native::native_call::ir_call_intrinsic_helper;
use crate::vm_exit_abi::register_structs::InvokeInterfaceResolve;

pub fn npe_check(assembler: &mut CodeAssembler, temp_register: Register, npe_exit_type: &IRVMExitType, possibly_null: Register) {
    let mut after_exit_label = assembler.create_label();
    assembler.xor(temp_register.to_native_64(), temp_register.to_native_64()).unwrap();
    assembler.cmp(temp_register.to_native_64(), possibly_null.to_native_64()).unwrap();
    assembler.jne(after_exit_label).unwrap();
    gen_vm_exit(assembler, npe_exit_type);
    assembler.nop_1(rax).unwrap();
    assembler.set_label(&mut after_exit_label).unwrap();
}

pub fn bounds_check(assembler: &mut CodeAssembler, length: Register, index: Register, size: Size, on_bounds_fail: &IRVMExitType) {
    let mut after_exit_label = assembler.create_label();
    match size {
        Size::Byte => assembler.cmp(index.to_native_8(), length.to_native_8()).unwrap(),
        Size::X86Word => assembler.cmp(index.to_native_16(), length.to_native_16()).unwrap(),
        Size::X86DWord => assembler.cmp(index.to_native_32(), length.to_native_32()).unwrap(),
        Size::X86QWord => assembler.cmp(index.to_native_64(), length.to_native_64()).unwrap(),
    }
    assembler.jl(after_exit_label).unwrap();
    gen_vm_exit(assembler, on_bounds_fail);
    assembler.nop_1(rax).unwrap();
    assembler.set_label(&mut after_exit_label).unwrap();
    assembler.nop().unwrap();
}


pub fn vtable_lookup_or_exit(assembler: &mut CodeAssembler, resolve_exit: &IRVMExitType, java_pc: ByteCodeOffset) {
    match resolve_exit {
        IRVMExitType::InvokeVirtualResolve {
            object_ref,
            method_number,
            ..
        } => {
            let obj_ptr = Register(0);
            assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
            let mut not_null = assembler.create_label();
            assembler.cmp(obj_ptr.to_native_64(), 0).unwrap();
            assembler.jne(not_null).unwrap();
            let registers = resolve_exit.registers_to_save();
            IRVMExitType::NPE { java_pc }.gen_assembly(assembler, &mut not_null);
            let mut before_exit_label = assembler.create_label();
            VMState::<u64>::gen_vm_exit(assembler, &mut before_exit_label, &mut not_null, registers);
            let vtable_ptr_register = Register(3);

            ir_call_intrinsic_helper(assembler, IntrinsicHelperType::FindVTablePtr,&vec![obj_ptr],Some(vtable_ptr_register), &vec![], &None, &vec![], &None );
            let address_register = InvokeVirtualResolve::ADDRESS_RES;// register 4
            assembler.sub(address_register.to_native_64(), address_register.to_native_64()).unwrap();
            generate_vtable_access(assembler, *method_number, vtable_ptr_register, Register(1), address_register);
            assembler.test(address_register.to_native_64(), address_register.to_native_64()).unwrap();
            let mut fast_resolve_worked = assembler.create_label();
            assembler.jnz(fast_resolve_worked).unwrap();
            let registers = resolve_exit.registers_to_save();
            resolve_exit.gen_assembly(assembler, &mut fast_resolve_worked);
            let mut before_exit_label = assembler.create_label();
            VMState::<u64>::gen_vm_exit(assembler, &mut before_exit_label, &mut fast_resolve_worked, registers);
            // assembler.set_label(&mut fast_resolve_worked).unwrap();
            assembler.nop().unwrap();
        }
        _ => panic!(),
    }
}



pub(crate) fn allocate_constant_size(assembler: &mut CodeAssembler, region_header_ptr: &*const AtomicPtr<RegionHeader>, res_offset: &FramePointerOffset, allocate_exit: &IRVMExitType) {
    let mut after_exit_label = assembler.create_label();
    let mut skip_to_exit_label = assembler.create_label();
    let region_header = Register(4);
    let zero = Register(5);
    let res = Register(6);
    // assembler.int3().unwrap();
    // assembler.jmp( skip_to_exit_label.clone()).unwrap();
    assembler.mov(region_header.to_native_64(), *region_header_ptr as u64).unwrap();
    assembler.mov(region_header.to_native_64(), qword_ptr(region_header.to_native_64())).unwrap();
    assembler.mov(rdi, region_header.to_native_64()).unwrap();
    assembler.and(rsp, -32).unwrap();//align stack pointer
    assembler.call(qword_ptr(r15 + IntrinsicHelperType::GetConstantAllocation.r15_offset())).unwrap();
    assembler.mov(res.to_native_64(), rax).unwrap();
    assembler.sub(zero.to_native_64(), zero.to_native_64()).unwrap();
    assembler.cmp(res.to_native_64(), zero.to_native_64()).unwrap();
    assembler.mov(rbp - res_offset.0, res.to_native_64()).unwrap();
    assembler.je(skip_to_exit_label).unwrap();
    assembler.jmp(after_exit_label).unwrap();
    match allocate_exit {
        IRVMExitType::AllocateObject { .. } => {
            assembler.set_label(&mut skip_to_exit_label).unwrap();
            assembler.nop().unwrap();
            let mut before_exit_label = assembler.create_label();
            let registers = allocate_exit.registers_to_save();
            allocate_exit.gen_assembly(assembler, &mut after_exit_label);
            VMState::<u64>::gen_vm_exit(assembler, &mut before_exit_label, &mut after_exit_label, registers);
        }
        _ => {
            panic!()
        }
    }
    assembler.nop().unwrap();
}

pub(crate) fn get_class_or_exit(assembler: &mut CodeAssembler, object_ref: &FramePointerOffset, res: &Register, get_class_exit: &IRVMExitType) {
    let mut after_exit_label = assembler.create_label();
    let obj_ptr = Register(0);
    // assembler.int3().unwrap();
    assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
    let class_ptr_register = Register(3);
    MemoryRegions::generate_find_class_ptr(assembler, obj_ptr, Register(1), Register(2), Register(4), class_ptr_register.clone());
    assembler.mov(res.to_native_64(), class_ptr_register.to_native_64()).unwrap();
    assembler.test(class_ptr_register.to_native_64(), class_ptr_register.to_native_64()).unwrap();
    assembler.jnz(after_exit_label).unwrap();
    match get_class_exit {
        IRVMExitType::RunSpecialNativeNew { .. } => {
            let registers = get_class_exit.registers_to_save();
            get_class_exit.gen_assembly(assembler, &mut after_exit_label);
            let mut before_exit_label = assembler.create_label();
            VMState::<u64>::gen_vm_exit(assembler, &mut before_exit_label, &mut after_exit_label, registers);
            assembler.nop().unwrap();
        }
        _ => {
            panic!()
        }
    }
}

pub(crate) fn itable_lookup_or_exit(assembler: &mut CodeAssembler, resolve_exit: &IRVMExitType) {
    match resolve_exit {
        IRVMExitType::InvokeInterfaceResolve { object_ref, interface_id, method_number, .. } => {
            let mut resolver_exit_label = assembler.create_label();
            let obj_ptr = Register(0);
            assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
            let itable_ptr_register = Register(3);
            ir_call_intrinsic_helper(assembler, IntrinsicHelperType::FindITablePtr, &vec![obj_ptr], Some(itable_ptr_register), &vec![], &None, &vec![], &None);
            let address_register = InvokeInterfaceResolve::ADDRESS_RES;// register 4
            // assembler.int3().unwrap();
            assembler.sub(address_register.to_native_64(), address_register.to_native_64()).unwrap();
            generate_itable_access(assembler, *method_number, *interface_id, itable_ptr_register, Register(5), Register(6), Register(7), address_register);
            assembler.test(address_register.to_native_64(), address_register.to_native_64()).unwrap();
            let mut fast_resolve_worked = assembler.create_label();
            assembler.jnz(fast_resolve_worked).unwrap();
            let registers = resolve_exit.registers_to_save();
            assembler.set_label(&mut resolver_exit_label).unwrap();
            resolve_exit.gen_assembly(assembler, &mut fast_resolve_worked);
            let mut before_exit_label = assembler.create_label();
            VMState::<u64>::gen_vm_exit(assembler, &mut before_exit_label, &mut fast_resolve_worked, registers);
            assembler.nop().unwrap();
        }
        _ => {
            panic!()
        }
    }
}



pub(crate) fn assert_equal(assembler: &mut CodeAssembler, a: &Register, b: &Register, size: &Size) {
    match size {
        Size::Byte => {
            assembler.cmp(a.to_native_8(), b.to_native_8()).unwrap();
        }
        Size::X86Word => {
            assembler.cmp(a.to_native_16(), b.to_native_16()).unwrap();
        }
        Size::X86DWord => {
            assembler.cmp(a.to_native_32(), b.to_native_32()).unwrap();
        }
        Size::X86QWord => {
            assembler.cmp(a.to_native_64(), b.to_native_64()).unwrap();
        }
    }
    let mut after = assembler.create_label();
    assembler.je(after).unwrap();
    assembler.int3().unwrap();
    assembler.set_label(&mut after).unwrap();
    assembler.nop().unwrap();
}

pub(crate) fn compare_and_swap(assembler: &mut CodeAssembler, ptr: &Register, old: &Register, new: &Register, res: &Register, should_be_rax: &Register, size: &Size) {
    assert_eq!(should_be_rax.0, 0);
    match *size {
        Size::Byte => {
            assembler.mov(al, old.to_native_8()).unwrap();
            todo!()
            // assembler.lock().cmpxchg(todo!()/*ptr.to_native_8() + 0*/, new.to_native_8()).unwrap();
        }
        Size::X86Word => {
            assembler.mov(ax, old.to_native_16()).unwrap();
            assembler.lock().cmpxchg(ptr.to_native_64() + 0, new.to_native_16()).unwrap();
        }
        Size::X86DWord => {
            assembler.mov(eax, old.to_native_32()).unwrap();
            assembler.lock().cmpxchg(ptr.to_native_64() + 0, new.to_native_32()).unwrap();
        }
        Size::X86QWord => {
            assembler.mov(rax, old.to_native_64()).unwrap();
            assembler.lock().cmpxchg(ptr.to_native_64() + 0, new.to_native_64()).unwrap();
        }
    }
    assembler.setz(res.to_native_8()).unwrap();
}








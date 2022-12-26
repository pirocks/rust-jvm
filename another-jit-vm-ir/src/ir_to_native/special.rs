use iced_x86::code_asm::{CodeAssembler, rax, rbp};

use another_jit_vm::{Register, VMState};
use gc_memory_layout_common::memory_regions::MemoryRegions;
use rust_jvm_common::ByteCodeOffset;
use vtable::generate_vtable_access;

use crate::{gen_vm_exit, InvokeVirtualResolve, IRVMExitType};
use crate::compiler::Size;

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
            MemoryRegions::generate_find_vtable_ptr(assembler, obj_ptr, Register(1), Register(2), Register(4), vtable_ptr_register);
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


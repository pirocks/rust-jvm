use iced_x86::code_asm::{CodeAssembler, rax, rbp};
use another_jit_vm::{Register, VMState};
use gc_memory_layout_common::memory_regions::MemoryRegions;
use vtable::generate_vtable_access;
use crate::{gen_vm_exit, InvokeVirtualResolve, IRVMExitType, Size};

pub fn npe_check(assembler: &mut CodeAssembler, temp_register: Register, npe_exit_type: &IRVMExitType, possibly_null: Register) {
    let mut after_exit_label = assembler.create_label();
    assembler.xor(temp_register.to_native_64(), temp_register.to_native_64()).unwrap();
    assembler.cmp(temp_register.to_native_64(), possibly_null.to_native_64()).unwrap();
    assembler.jne(after_exit_label).unwrap();
    gen_vm_exit(assembler, npe_exit_type);
    assembler.nop_1(rax).unwrap();
    assembler.set_label(&mut after_exit_label).unwrap();
}

pub fn bounds_check(assembler: &mut CodeAssembler, length: Register, index: Register, size: Size) {
    let mut not_out_of_bounds = assembler.create_label();
    match size {
        Size::Byte => assembler.cmp(index.to_native_8(), length.to_native_8()).unwrap(),
        Size::X86Word => assembler.cmp(index.to_native_16(), length.to_native_16()).unwrap(),
        Size::X86DWord => assembler.cmp(index.to_native_32(), length.to_native_32()).unwrap(),
        Size::X86QWord => assembler.cmp(index.to_native_64(), length.to_native_64()).unwrap(),
    }
    assembler.jl(not_out_of_bounds.clone()).unwrap();
    assembler.int3().unwrap();//todo
    assembler.set_label(&mut not_out_of_bounds).unwrap();
    assembler.nop().unwrap();
}




pub fn vtable_lookup_or_exit(assembler: &mut CodeAssembler, resolve_exit: &IRVMExitType) {
    match resolve_exit {
        IRVMExitType::InvokeVirtualResolve {
            object_ref,
            method_number,
            ..
        } => {
            let obj_ptr = Register(0);
            assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
            let vtable_ptr_register = Register(3);
            MemoryRegions::generate_find_vtable_ptr(assembler, obj_ptr, Register(1), Register(2), Register(4), vtable_ptr_register);
            let address_register = InvokeVirtualResolve::ADDRESS_RES;// register 4
            let ir_method_id_register = InvokeVirtualResolve::IR_METHOD_ID_RES;// register 5
            let method_id_register = InvokeVirtualResolve::METHOD_ID_RES;// register 6
            let frame_size_register = InvokeVirtualResolve::NEW_FRAME_SIZE_RES;// register 7
            generate_vtable_access(assembler, *method_number, vtable_ptr_register, Register(1), address_register, ir_method_id_register, method_id_register, frame_size_register);
            assembler.test(address_register.to_native_64(), address_register.to_native_64()).unwrap();
            let mut fast_resolve_worked = assembler.create_label();
            assembler.jnz(fast_resolve_worked.clone()).unwrap();
            let registers = resolve_exit.registers_to_save();
            resolve_exit.gen_assembly(assembler, &mut fast_resolve_worked, &registers);
            let mut before_exit_label = assembler.create_label();
            VMState::<u64, ()>::gen_vm_exit(assembler, &mut before_exit_label, &mut fast_resolve_worked, registers);
            // assembler.set_label(&mut fast_resolve_worked).unwrap();
            assembler.nop().unwrap();
        }
        _ => panic!(),
    }
}


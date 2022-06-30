use std::collections::HashMap;
use std::mem::size_of;

use iced_x86::code_asm::{byte_ptr, CodeAssembler, CodeLabel, dword_ptr, rbp, rbx, ymm0, ymm1, ymm2, ymm4};
use memoffset::offset_of;

use another_jit_vm::{Register, VMState};
use another_jit_vm::code_modification::AssemblerFunctionCallTarget;
use gc_memory_layout_common::memory_regions::MemoryRegions;
use gc_memory_layout_common::memory_regions::RegionHeader;
use inheritance_tree::ClassID;
use inheritance_tree::paths::BitPath256;
use interface_vtable::generate_itable_access;

use crate::{gen_vm_exit, IRInstr, IRInstructIndex, IRVMExitType, LabelName, RestartPointID, Size};
use crate::ir_to_native::bit_manipulation::{binary_bit_and, binary_bit_or, binary_bit_xor, shift_left, shift_right};
use crate::ir_to_native::call::{ir_call, ir_function_start, ir_return};
use crate::ir_to_native::integer_arithmetic::{ir_add, ir_div, ir_mod, ir_sub, mul, mul_const, sign_extend, zero_extend};
use crate::ir_to_native::integer_compare::{int_compare, sized_integer_compare};
use crate::ir_to_native::load_store::{ir_load, ir_load_fp_relative, ir_store, ir_store_fp_relative};
use crate::ir_to_native::special::{bounds_check, npe_check, vtable_lookup_or_exit};
use crate::vm_exit_abi::register_structs::InvokeInterfaceResolve;

pub mod bit_manipulation;
pub mod integer_arithmetic;
pub mod float_arithmetic;
pub mod integer_compare;
pub mod float_compare;
pub mod call;
pub mod load_store;
pub mod special;


pub fn single_ir_to_native(assembler: &mut CodeAssembler, instruction: &IRInstr, labels: &mut HashMap<LabelName, CodeLabel>,
                           restart_points: &mut HashMap<RestartPointID, IRInstructIndex>, ir_instr_index: IRInstructIndex) -> Option<AssemblerFunctionCallTarget> {
    match instruction {
        IRInstr::LoadFPRelative { from, to, size } => {
            ir_load_fp_relative(assembler, *from, *to, *size)
        }
        IRInstr::StoreFPRelative { from, to, size } => {
            ir_store_fp_relative(assembler, *from, *to, *size)
        }
        IRInstr::Load { to, from_address, size } => {
            ir_load(assembler, *to, *from_address, *size)
        }
        IRInstr::Store { from, to_address, size } => {
            ir_store(assembler, *from, *to_address, *size)
        }
        IRInstr::CopyRegister { .. } => todo!(),
        IRInstr::Add { res, a, size } => {
            ir_add(assembler, *res, *a, *size)
        }
        IRInstr::Sub { res, to_subtract, size } => {
            ir_sub(assembler, *res, *to_subtract, *size)
        }
        IRInstr::Div { res, divisor, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed } => {
            ir_div(assembler, *res, *divisor, *must_be_rax, *must_be_rbx, *must_be_rcx, *must_be_rdx, *size, signed)
        }
        IRInstr::Mod { res, divisor, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed } => {
            ir_mod(assembler, *res, *divisor, *must_be_rax, *must_be_rbx, *must_be_rcx, *must_be_rdx, *size, signed)
        }
        IRInstr::Mul { res, a, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed } => {
            mul(assembler, *res, *a, *must_be_rax, *must_be_rbx, *must_be_rcx, *must_be_rdx, *size, signed)
        }
        IRInstr::BinaryBitAnd { res, a, size } => {
            binary_bit_and(assembler, *res, *a, *size)
        }
        IRInstr::BinaryBitXor { res, a, size } => {
            binary_bit_xor(assembler, *res, *a, *size)
        }
        IRInstr::Const32bit { const_, to } => {
            assembler.mov(to.to_native_32(), *const_).unwrap();
        }
        IRInstr::Const64bit { const_, to } => {
            assembler.mov(to.to_native_64(), *const_).unwrap();
        }
        IRInstr::BranchToLabel { label } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            assembler.jmp(*code_label).unwrap();
        }
        IRInstr::LoadLabel { .. } => todo!(),
        IRInstr::LoadRBP { .. } => todo!(),
        IRInstr::WriteRBP { .. } => todo!(),
        IRInstr::BranchEqual { a, b, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, *a, *b, *size);
            assembler.je(*code_label).unwrap();
        }
        IRInstr::BranchNotEqual { a, b, label, size, } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, *a, *b, *size);
            assembler.jne(*code_label).unwrap();
        }
        IRInstr::BranchAGreaterEqualB { a, b, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, *a, *b, *size);
            assembler.jge(*code_label).unwrap();
        }
        IRInstr::BranchAGreaterB { a, b, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, *a, *b, *size);
            assembler.jg(*code_label).unwrap();
        }
        IRInstr::BranchALessB { a, b, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, *a, *b, *size);
            assembler.jl(*code_label).unwrap();
        }
        IRInstr::Return { return_val, temp_register_1, temp_register_2, temp_register_3, temp_register_4, frame_size } => {
            ir_return(assembler, *return_val, *temp_register_1, *temp_register_2, *temp_register_3, *temp_register_4, frame_size);
        }
        IRInstr::VMExit2 { exit_type } => {
            gen_vm_exit(assembler, exit_type);
        }
        IRInstr::NOP => {
            assembler.nop().unwrap();
        }
        IRInstr::Label(label) => {
            let label_name = label.name;
            let code_label = labels.entry(label_name).or_insert_with(|| assembler.create_label());
            assembler.nop().unwrap();
            assembler.set_label(code_label).unwrap();
        }
        IRInstr::IRCall {
            temp_register_1,
            temp_register_2,
            arg_from_to_offsets,
            return_value,
            target_address,
            current_frame_size
        } => {
            return ir_call(assembler, *temp_register_1, *temp_register_2, arg_from_to_offsets, *return_value, *target_address, *current_frame_size);
        }
        IRInstr::IRStart {
            temp_register, ir_method_id, method_id, frame_size, num_locals
        } => {
            ir_function_start(assembler, *temp_register, *ir_method_id, *method_id, *frame_size, *num_locals)
        }
        IRInstr::NPECheck { temp_register, npe_exit_type, possibly_null } => {
            npe_check(assembler, *temp_register, npe_exit_type, *possibly_null);
        }
        IRInstr::RestartPoint(restart_point_id) => {
            assembler.nop_1(rbx).unwrap();
            restart_points.insert(*restart_point_id, ir_instr_index);
        }
        IRInstr::DebuggerBreakpoint => {
            assembler.int3().unwrap();
        }
        IRInstr::Const16bit { const_, to } => {
            assembler.mov(to.to_native_32(), *const_ as u32).unwrap()
        }
        IRInstr::ShiftLeft { res, a, cl_aka_register_2, size, signed } => {
            shift_left(assembler, *res, *a, *cl_aka_register_2, *size, *signed)
        }
        IRInstr::ShiftRight { res, a, cl_aka_register_2, size, signed } => {
            shift_right(assembler, *res, *a, *cl_aka_register_2, *size, *signed)
        }
        IRInstr::BoundsCheck { length, index, size } => {
            bounds_check(assembler, *length, *index, *size);
        }
        IRInstr::MulConst { res, a, size, signed } => {
            mul_const(assembler, *res, a, *size, signed);
        }
        IRInstr::LoadFPRelativeDouble { from, to } => {
            assembler.vmovsd(to.to_xmm(), rbp - from.0).unwrap();
        }
        IRInstr::StoreFPRelativeDouble { from, to } => {
            assembler.vmovsd(rbp - to.0, from.to_xmm()).unwrap();
        }
        IRInstr::LoadFPRelativeFloat { from, to } => {
            assembler.movss(to.to_xmm(), rbp - from.0).unwrap();
        }
        IRInstr::StoreFPRelativeFloat { from, to } => {
            assembler.movss(rbp - to.0, from.to_xmm()).unwrap();
        }
        IRInstr::DoubleToIntegerConvert { from, temp, to } => {
            assembler.cvtpd2pi(temp.to_mm(), from.to_xmm()).unwrap();
            assembler.movd(to.to_native_32(), temp.to_mm()).unwrap();
        }
        IRInstr::IntegerToDoubleConvert { to, temp, from } => {
            assembler.movd(temp.to_mm(), from.to_native_32()).unwrap();
            assembler.cvtpi2pd(to.to_xmm(), temp.to_mm()).unwrap()
        }
        IRInstr::DoubleToLongConvert { from, to } => {
            assembler.cvttsd2si(to.to_native_64(), from.to_xmm()).unwrap();
            // assembler.movq(to.to_native_64(), temp.to_mm()).unwrap();
        }
        IRInstr::FloatToIntegerConvert { from, temp, to } => {
            assembler.cvtps2pi(temp.to_mm(), from.to_xmm()).unwrap();
            assembler.movd(to.to_native_32(), temp.to_mm()).unwrap();
        }
        IRInstr::IntegerToFloatConvert { to, temp, from } => {
            assembler.movd(temp.to_mm(), from.to_native_32()).unwrap();
            //todo use cvtsi2ss instead avoids the move to mmx
            assembler.cvtpi2ps(to.to_xmm(), temp.to_mm()).unwrap()
        }
        IRInstr::LongToFloatConvert { to, from } => {
            // assembler.movq(temp.to_mm(), from.to_native_64()).unwrap();
            assembler.cvtsi2ss(to.to_xmm(), from.to_native_64()).unwrap()
        }
        IRInstr::LongToDoubleConvert { to, from } => {
            // assembler.movq(temp.to_mm(), from.to_native_64()).unwrap();
            assembler.cvtsi2sd(to.to_xmm(), from.to_native_64()).unwrap()
        }
        IRInstr::FloatCompare { value1, value2, res, temp1: one, temp2: zero, temp3: m_one, compare_mode } => {
            float_compare::float_compare(assembler, *value1, *value2, *res, *one, *zero, *m_one, *compare_mode);
        }
        IRInstr::DoubleCompare { value1, value2, res, temp1: one, temp2: zero, temp3: m_one, compare_mode } => {
            float_compare::double_compare(assembler, *value1, *value2, *res, *one, *zero, *m_one, *compare_mode);
        }
        IRInstr::IntCompare { res, value1, value2, temp1, temp2, temp3, size } => {
            int_compare(assembler, *res, *value1, *value2, *temp1, *temp2, *temp3, *size);
        }
        IRInstr::MulFloat { res, a } => {
            assembler.mulps(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::DivFloat { res, divisor } => {
            assembler.divss(res.to_xmm(), divisor.to_xmm()).unwrap();
        }
        IRInstr::AddFloat { res, a } => {
            assembler.addss(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::SubFloat { res, a } => {
            assembler.subss(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::SubDouble { res, a } => {
            assembler.subsd(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::BinaryBitOr { res, a, size } => {
            binary_bit_or(assembler, *res, *a, *size)
        }
        IRInstr::FloatToDoubleConvert { from, to } => {
            assembler.cvtps2pd(to.to_xmm(), from.to_xmm()).unwrap();
        }
        IRInstr::MulDouble { res, a } => {
            assembler.mulpd(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::AddDouble { res, a } => {
            assembler.addpd(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::SignExtend { from, to, from_size, to_size } => {
            sign_extend(assembler, *from, *to, *from_size, *to_size);
        }
        IRInstr::ZeroExtend { from, to, from_size, to_size } => {
            zero_extend(assembler, *from, *to, *from_size, *to_size);
        }
        IRInstr::VTableLookupOrExit { resolve_exit, java_pc } => {
            vtable_lookup_or_exit(assembler, resolve_exit, *java_pc)
        }
        IRInstr::ITableLookupOrExit { resolve_exit } => {
            match resolve_exit {
                IRVMExitType::InvokeInterfaceResolve { object_ref, interface_id, method_number, .. } => {
                    let obj_ptr = Register(0);
                    assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
                    let vtable_ptr_register = Register(3);
                    MemoryRegions::generate_find_itable_ptr(assembler, obj_ptr, Register(1), Register(2), Register(4), vtable_ptr_register.clone());
                    let address_register = InvokeInterfaceResolve::ADDRESS_RES;// register 4
                    // assembler.int3().unwrap();
                    assembler.sub(address_register.to_native_64(), address_register.to_native_64()).unwrap();
                    generate_itable_access(assembler, *method_number, *interface_id, vtable_ptr_register, Register(5), Register(6), Register(7), address_register);
                    assembler.test(address_register.to_native_64(), address_register.to_native_64()).unwrap();
                    let mut fast_resolve_worked = assembler.create_label();
                    assembler.jnz(fast_resolve_worked.clone()).unwrap();
                    let registers = resolve_exit.registers_to_save();
                    resolve_exit.gen_assembly(assembler, &mut fast_resolve_worked, &registers);
                    let mut before_exit_label = assembler.create_label();
                    VMState::<u64, ()>::gen_vm_exit(assembler, &mut before_exit_label, &mut fast_resolve_worked, registers);
                    assembler.nop().unwrap();
                }
                _ => {
                    panic!()
                }
            }
        }
        IRInstr::InstanceOfClass {
            inheritance_path,
            object_ref,
            return_val,
            instance_of_exit,
        } => {
            let mut instance_of_exit_label = assembler.create_label();
            let mut instance_of_succeed = assembler.create_label();
            let mut instance_of_fail = assembler.create_label();
            let obj_ptr = Register(0);
            assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
            assembler.cmp(obj_ptr.to_native_64(), 0).unwrap();
            assembler.je(instance_of_fail).unwrap();
            let object_inheritance_path_pointer = Register(3);
            MemoryRegions::generate_find_object_region_header(assembler, obj_ptr, Register(1), Register(2), Register(4), object_inheritance_path_pointer.clone());
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
            assembler.je(instance_of_succeed.clone()).unwrap();
            assembler.jmp(instance_of_fail.clone()).unwrap();
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
                    instance_of_exit.gen_assembly(assembler, &mut done, &registers);
                    let mut before_exit_label = assembler.create_label();
                    VMState::<u64, ()>::gen_vm_exit(assembler, &mut before_exit_label, &mut done, registers);
                }
                IRVMExitType::CheckCast { .. } => {
                    let registers = instance_of_exit.registers_to_save();
                    instance_of_exit.gen_assembly(assembler, &mut done, &registers);
                    let mut before_exit_label = assembler.create_label();
                    VMState::<u64, ()>::gen_vm_exit(assembler, &mut before_exit_label, &mut done, registers);
                }
                _ => {
                    panic!()
                }
            }
            // assembler.set_label(&mut done).unwrap(); //done in gen_vm_exit
            assembler.nop().unwrap();
        }
        IRInstr::InstanceOfInterface { target_interface_id, object_ref, return_val } => {
            let obj_ptr = Register(0);
            let mut instance_of_succeed = assembler.create_label();
            let mut instance_of_fail = assembler.create_label();
            assembler.mov(obj_ptr.to_native_64(), rbp - object_ref.0).unwrap();
            assembler.cmp(obj_ptr.to_native_64(), 0).unwrap();
            assembler.je(instance_of_fail).unwrap();
            let interface_list_base_pointer = Register(3);
            let interface_list_base_pointer_len = Register(5);
            MemoryRegions::generate_find_object_region_header(assembler, obj_ptr, Register(1), Register(2), Register(4), interface_list_base_pointer.clone());
            assembler.mov(interface_list_base_pointer_len.to_native_64(), interface_list_base_pointer.to_native_64() + offset_of!(RegionHeader,interface_ids_list_len)).unwrap();
            assembler.mov(interface_list_base_pointer.to_native_64(), interface_list_base_pointer.to_native_64() + offset_of!(RegionHeader,interface_ids_list)).unwrap();
            assembler.lea(interface_list_base_pointer_len.to_native_64(), interface_list_base_pointer.to_native_64() + interface_list_base_pointer_len.to_native_64() * size_of::<ClassID>()).unwrap();
            let mut loop_ = assembler.create_label();
            assembler.set_label(&mut loop_).unwrap();
            assembler.cmp(interface_list_base_pointer.to_native_64(), interface_list_base_pointer_len.to_native_64()).unwrap();
            assembler.je(instance_of_fail).unwrap();
            assembler.cmp(dword_ptr(interface_list_base_pointer.to_native_64()), target_interface_id.0 as u32).unwrap();
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
        IRInstr::BranchEqualVal { a, const_, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            match size {
                Size::Byte => {
                    todo!()
                }
                Size::X86Word => {
                    todo!()
                }
                Size::X86DWord => {
                    assembler.cmp(a.to_native_32(), *const_ as u32).unwrap();
                }
                Size::X86QWord => {
                    panic!()
                }
            }

            assembler.je(*code_label).unwrap();
        }
        IRInstr::Allocate { region_header_ptr, allocate_exit } => {
            let mut before_exit_label = assembler.create_label();
            let mut after_exit_label = assembler.create_label();
            let region_header = Register(4);
            let zero = Register(5);
            assembler.int3().unwrap();
            assembler.sub(zero.to_native_64(), zero.to_native_64()).unwrap();
            assembler.mov(region_header.to_native_64(), *region_header_ptr as u64).unwrap();
            assembler.mov(region_header.to_native_64(), region_header.to_native_64() + 0).unwrap();
            assembler.cmp(region_header.to_native_64(), zero.to_native_64()).unwrap();
            assembler.je(before_exit_label.clone()).unwrap();
            let current_index = Register(6);
            assembler.mov(current_index.to_native_64(), 1 as u64).unwrap();
            assembler.lock().xadd(region_header.to_native_64() + offset_of!(RegionHeader,num_current_elements), current_index.to_native_64()).unwrap();
            let max_elements = Register(7);
            assembler.mov(max_elements.to_native_64(),region_header.to_native_64() + offset_of!(RegionHeader, region_max_elements)).unwrap();
            assembler.cmp(current_index.to_native_64(), max_elements.to_native_64()).unwrap();
            assembler.jge(before_exit_label.clone()).unwrap();
            let region_elem_size = Register(8);
            assembler.mov(region_elem_size.to_native_64(), region_header.to_native_64() + offset_of!(RegionHeader, region_elem_size)).unwrap();
            let res_ptr = Register(0);
            assembler.mov(res_ptr.to_native_64(), current_index.to_native_64()).unwrap();
            assembler.mul(region_elem_size.to_native_64()).unwrap();//implicit mul with res
            assembler.add(res_ptr.to_native_64(), region_header.to_native_64()).unwrap();
            let mut zero_loop_start = assembler.create_label();
            assembler.set_label(&mut zero_loop_start).unwrap();
            assembler.sub(region_elem_size.to_native_64(), 1).unwrap();
            assembler.mov(byte_ptr(res_ptr.to_native_64() + region_elem_size.to_native_64()), zero.to_native_8()).unwrap();
            assembler.cmp(region_elem_size.to_native_64(), 0).unwrap();
            assembler.jne(zero_loop_start).unwrap();

            assembler.jmp( after_exit_label.clone()).unwrap();
            match allocate_exit {
                IRVMExitType::AllocateObject { .. } => {
                    let registers = allocate_exit.registers_to_save();
                    allocate_exit.gen_assembly(assembler, &mut after_exit_label, &registers);
                    VMState::<u64, ()>::gen_vm_exit(assembler, &mut before_exit_label, &mut after_exit_label, registers);
                }
                _ => {
                    panic!()
                }
            }
            assembler.nop().unwrap();
        }
    }
    None
}







use std::collections::HashMap;

use iced_x86::code_asm::{CodeAssembler, CodeLabel, r15, rbp, rbx};
use memoffset::offset_of;

use another_jit_vm::{JITContext};
use another_jit_vm::code_modification::AssemblerFunctionCallTarget;
use another_jit_vm::intrinsic_helpers::{ThreadLocalIntrinsicHelpers};

use crate::{gen_vm_exit, IRInstr, IRInstructIndex, LabelName, RestartPointID};
use crate::ir_to_native::bit_manipulation::{binary_bit_and, binary_bit_or, binary_bit_xor, shift_left, shift_right};
use crate::ir_to_native::branch::{branch_a_greate_equal_b, branch_a_greater_b, branch_a_less_b, branch_equal, branch_equal_val, branch_not_equal};
use crate::ir_to_native::call::{ir_call, ir_function_start, ir_return};
use crate::ir_to_native::float_arithmetic::{neg_double, neg_float};
use crate::ir_to_native::float_double_const::{const_double, const_float};
use crate::ir_to_native::float_double_convert::{double_to_integer, double_to_long, float_to_integer, float_to_long};
use crate::ir_to_native::instance_of::{instance_of_class, instance_of_interface};
use crate::ir_to_native::integer_arithmetic::{ir_add, ir_div, ir_mod, ir_sub, mul, mul_const, sign_extend, zero_extend};
use crate::ir_to_native::integer_compare::{int_compare};
use crate::ir_to_native::load_store::{ir_load, ir_load_fp_relative, ir_store, ir_store_fp_relative};
use crate::ir_to_native::native_call::{call_native_helper, ir_call_intrinsic_helper};
use crate::ir_to_native::special::{allocate_constant_size, assert_equal, bounds_check, compare_and_swap, get_class_or_exit, itable_lookup_or_exit, npe_check, vtable_lookup_or_exit};

pub mod bit_manipulation;
pub mod integer_arithmetic;
pub mod float_arithmetic;
pub mod integer_compare;
pub mod float_compare;
pub mod call;
pub mod load_store;
pub mod special;
pub mod native_call;
pub mod float_double_convert;
pub mod instance_of;
pub mod float_double_const;
pub mod branch;


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
        IRInstr::CopyRegister { from, to } => {
            assembler.mov(to.to_native_64(), from.to_native_64()).unwrap();
        }
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
            assembler.nop().unwrap();
        }
        IRInstr::LoadLabel { .. } => todo!(),
        IRInstr::LoadRBP { .. } => todo!(),
        IRInstr::WriteRBP { .. } => todo!(),
        IRInstr::BranchEqual { a, b, label, size } => {
            branch_equal(assembler, labels, a, b, label, size);
        }
        IRInstr::BranchNotEqual { a, b, label, size, } => {
            branch_not_equal(assembler, labels, a, b, label, size);
        }
        IRInstr::BranchAGreaterEqualB { a, b, label, size } => {
            branch_a_greate_equal_b(assembler, labels, a, b, label, size);
        }
        IRInstr::BranchAGreaterB { a, b, label, size } => {
            branch_a_greater_b(assembler, labels, a, b, label, size);
        }
        IRInstr::BranchALessB { a, b, label, size } => {
            branch_a_less_b(assembler, labels, a, b, label, size);
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
        IRInstr::BoundsCheck { length, index, size, exit } => {
            bounds_check(assembler, *length, *index, *size, exit);
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
        IRInstr::DoubleToIntegerConvert { from, temp_1, temp_2, temp_3, temp_4, to } => {
            double_to_integer(assembler, from, temp_1, temp_2, temp_3, temp_4, to);
        }
        IRInstr::IntegerToDoubleConvert { to, temp, from } => {
            assembler.movd(temp.to_mm(), from.to_native_32()).unwrap();
            assembler.cvtpi2pd(to.to_xmm(), temp.to_mm()).unwrap()
        }
        IRInstr::DoubleToLongConvert { from, temp_1, temp_2: _, temp_3, temp_4: _, to } => {
            double_to_long(assembler, from, temp_1, temp_3, to);
        }
        IRInstr::FloatToLongConvert { from, temp_1, temp_2: _, temp_3, temp_4: _, to } => {
            float_to_long(assembler, from, temp_1, temp_3, to);
        }
        IRInstr::FloatToIntegerConvert { from, temp_1, temp_2: _, temp_3, temp_4: _, to } => {
            float_to_integer(assembler, from, temp_1, temp_3, to);
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
        IRInstr::DivDouble { res, divisor } => {
            assembler.divpd(res.to_xmm(), divisor.to_xmm()).unwrap();
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
        IRInstr::DoubleToFloatConvert { from, to } => {
            assembler.cvtpd2ps(to.to_xmm(), from.to_xmm()).unwrap();
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
        IRInstr::GetClassOrExit { object_ref, res, get_class_exit } => {
            get_class_or_exit(assembler, object_ref, res, get_class_exit)
        }
        IRInstr::ITableLookupOrExit { resolve_exit } => {
            itable_lookup_or_exit(assembler, resolve_exit);
        }
        IRInstr::InstanceOfClass {
            inheritance_path,
            object_ref,
            return_val,
            instance_of_exit,
        } => {
            instance_of_class(assembler, inheritance_path, object_ref, return_val, instance_of_exit);
        }
        IRInstr::InstanceOfInterface { target_interface_id, object_ref, return_val } => {
            instance_of_interface(assembler, target_interface_id, object_ref, return_val);
        }
        IRInstr::BranchEqualVal { a, const_, label, size } => {
            branch_equal_val(assembler, labels, a, const_, label, size);
        }
        IRInstr::AllocateConstantSize { region_header_ptr, res_offset, allocate_exit } => {
            allocate_constant_size(assembler, region_header_ptr, res_offset, allocate_exit);
        }
        IRInstr::ConstFloat { to, const_, temp } => {
            const_float(assembler, to, const_, temp);
        }
        IRInstr::ConstDouble { to, temp, const_ } => {
            const_double(assembler, to, temp, const_);
        }
        IRInstr::AddConst { res, a } => {
            assembler.add(res.to_native_64(), *a).unwrap();
        }
        IRInstr::CompareAndSwapAtomic { ptr, old, new, res, rax: should_be_rax, size } => {
            compare_and_swap(assembler, ptr, old, new, res, should_be_rax, size);
        }
        IRInstr::AssertEqual { a, b, size } => {
            assert_equal(assembler, a, b, size);
        }
        IRInstr::CallIntrinsicHelper { intrinsic_helper_type,
            integer_args,
            integer_res,
            float_args,
            float_res,
            double_args,
            double_res } => {
            ir_call_intrinsic_helper(assembler, *intrinsic_helper_type, integer_args, *integer_res, float_args, float_res, double_args, double_res);
        }
        IRInstr::NegFloat { temp_normal, temp, res } => {
            neg_float(assembler, temp_normal, temp, res);
        }
        IRInstr::NegDouble { temp_normal, temp, res } => {
            neg_double(assembler, temp_normal, temp, res);
        }
        IRInstr::CallNativeHelper { to_call, integer_args, byte_res, bool_res, char_res, short_res, integer_res, float_double_args, float_res, double_res, } => {
            call_native_helper(assembler, to_call, integer_args, byte_res, bool_res, char_res, short_res, integer_res, float_double_args, float_res, double_res)
        }
        IRInstr::ArrayElemSizeLookup { .. } => {
            todo!()
        }
        IRInstr::MaxUnsigned { .. } => {
            todo!()
        }
        IRInstr::GetThread { res_register } => {
            assembler.mov(res_register.to_native_64(), r15 + offset_of!(JITContext,thread_local_intrinsic_data) + offset_of!(ThreadLocalIntrinsicHelpers,current_thread_obj)).unwrap();
        }
        IRInstr::ArrayStoreCheck { array_ref, obj_elem, temp_1, temp_2, temp_3, temp_4 } => {
//             RegionHeader{};
// ir_call_intrinsic_helper()
//             assembler.mov().unwrap();
            todo!()
        }
    }
    None
}


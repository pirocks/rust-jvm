use std::collections::HashMap;
use iced_x86::code_asm::{CodeAssembler, CodeLabel, rbp, rbx};
use another_jit_vm::code_modification::{AssemblerFunctionCallTarget};
use crate::ir_to_native::bit_manipulation::{binary_bit_and, binary_bit_or, binary_bit_xor, shift_left, shift_right};
use crate::ir_to_native::call::{ir_call, ir_function_start, ir_return};
use crate::ir_to_native::integer_arithmetic::{ir_add, ir_div, ir_mod, ir_sub, mul, mul_const, sign_extend, zero_extend};
use crate::ir_to_native::integer_compare::{int_compare, sized_integer_compare};
use crate::ir_to_native::load_store::{ir_load, ir_load_fp_relative, ir_store, ir_store_fp_relative};
use crate::ir_to_native::special::{bounds_check, npe_check, vtable_lookup_or_exit};
use crate::{gen_vm_exit_impl, IRInstr, IRInstructIndex, LabelName, RestartPointID};

pub mod bit_manipulation;
pub mod integer_arithmetic;
pub mod float_arithmetic;
pub mod integer_compare;
pub mod float_compare;
pub mod call;
pub mod load_store;
pub mod special;


pub fn single_ir_to_native(assembler: &mut CodeAssembler, instruction: &IRInstr,
                           labels: &mut HashMap<LabelName, CodeLabel>,
                           restart_points: &mut HashMap<RestartPointID, IRInstructIndex>,
                           ir_instr_index: IRInstructIndex,
                           editable: bool,
) -> Option<AssemblerFunctionCallTarget> {
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
            gen_vm_exit_impl(assembler, exit_type, editable);
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
            npe_check(assembler, *temp_register, npe_exit_type, *possibly_null, editable);
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
            bounds_check(assembler, *length, *index, *size, editable);
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
        IRInstr::VTableLookupOrExit { resolve_exit } => {
            vtable_lookup_or_exit(assembler, resolve_exit)
        }
    }
    None
}







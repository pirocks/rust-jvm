use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::compressed_classfile::code::CompressedInstruction;
use crate::ir_compiler_common::branching::IntegerCompareKind;
use crate::ir_compiler_common::special::IRCompilerState;
use crate::ir_compiler_common::stack_stores::StackPos;

pub(crate) fn emit_integer_if_compare_with_zero(compiler_state: &mut IRCompilerState, instr: &CompressedInstruction, offset: i16, integer_compare_kind: IntegerCompareKind) {
    let (branch_to, label_target) = compiler_state.create_label();
    let target_offset = ByteCodeOffset((instr.offset.0 as i32 + offset as i32) as u16);
    compiler_state.set_label_target_pending(target_offset, label_target);
    let zero = compiler_state.emit_constant_int(0);
    let value = compiler_state.emit_stack_load_int(StackPos::BeforeFromEnd(0));
    compiler_state.emit_branch_compare_int(branch_to, value, zero, integer_compare_kind);
}

pub(crate) fn emit_integer_if_compare_two_values(compiler_state: &mut IRCompilerState, instr: &CompressedInstruction, offset: i16, integer_compare_kind: IntegerCompareKind) {
    let (branch_to, label_target) = compiler_state.create_label();
    let target_offset = ByteCodeOffset((instr.offset.0 as i32 + offset as i32) as u16);
    compiler_state.set_label_target_pending(target_offset, label_target);
    let value1 = compiler_state.emit_stack_load_int(StackPos::BeforeFromEnd(1));
    let value2 = compiler_state.emit_stack_load_int(StackPos::BeforeFromEnd(0));
    compiler_state.emit_branch_compare_int(branch_to, value1, value2, integer_compare_kind);
}

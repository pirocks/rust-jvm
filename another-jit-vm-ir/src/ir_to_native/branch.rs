use std::collections::HashMap;
use iced_x86::code_asm::{CodeAssembler, CodeLabel};
use another_jit_vm::Register;
use crate::compiler::{LabelName, Size};
use crate::ir_to_native::integer_compare::sized_integer_compare;

pub(crate) fn branch_equal_val(assembler: &mut CodeAssembler, labels: &mut HashMap<LabelName, CodeLabel>, a: &Register, const_: &u32, label: &LabelName, size: &Size) {
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

pub(crate) fn branch_a_less_b(assembler: &mut CodeAssembler, labels: &mut HashMap<LabelName, CodeLabel>, a: &Register, b: &Register, label: &LabelName, size: &Size) {
    let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
    sized_integer_compare(assembler, *a, *b, *size);
    assembler.jl(*code_label).unwrap();
}

pub(crate) fn branch_a_greater_b(assembler: &mut CodeAssembler, labels: &mut HashMap<LabelName, CodeLabel>, a: &Register, b: &Register, label: &LabelName, size: &Size) {
    let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
    sized_integer_compare(assembler, *a, *b, *size);
    assembler.jg(*code_label).unwrap();
}

pub(crate) fn branch_a_greate_equal_b(assembler: &mut CodeAssembler, labels: &mut HashMap<LabelName, CodeLabel>, a: &Register, b: &Register, label: &LabelName, size: &Size) {
    let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
    sized_integer_compare(assembler, *a, *b, *size);
    assembler.jge(*code_label).unwrap();
}

pub(crate) fn branch_not_equal(assembler: &mut CodeAssembler, labels: &mut HashMap<LabelName, CodeLabel>, a: &Register, b: &Register, label: &LabelName, size: &Size) {
    let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
    sized_integer_compare(assembler, *a, *b, *size);
    assembler.jne(*code_label).unwrap();
}

pub(crate) fn branch_equal(assembler: &mut CodeAssembler, labels: &mut HashMap<LabelName, CodeLabel>, a: &Register, b: &Register, label: &LabelName, size: &Size) {
    let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
    sized_integer_compare(assembler, *a, *b, *size);
    assembler.je(*code_label).unwrap();
}

use another_jit_vm_ir::compiler::Size;
use array_memory_layout::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use crate::ir_compiler_common::ONE;
use crate::ir_compiler_common::special::IRCompilerState;
use crate::ir_compiler_common::stack_stores::StackPos;

pub(crate) fn array_load_impl(compiler: &mut IRCompilerState, arr_sub_type: CPDType) {
    let array_layout = ArrayMemoryLayout::from_cpdtype(arr_sub_type);
    let elem_0_offset = array_layout.elem_0_entry_offset();
    let len_offset = array_layout.len_entry_offset();
    let array_elem_size = array_layout.elem_size();
    let index = compiler.emit_stack_load_int(StackPos::BeforeFromEnd(0));
    let array_ref = compiler.emit_stack_load_pointer(StackPos::BeforeFromEnd(1));
    compiler.emit_npe_check(array_ref);
    let len_pointer = compiler.emit_address_calculate_int(array_ref, index, len_offset, ONE);
    let len = compiler.emit_load_int_sign_extend(len_pointer, Size::int());
    compiler.emit_array_bounds_check(len, index);
    let elem_pointer = compiler.emit_address_calculate_int(array_ref, index, elem_0_offset, array_elem_size);
    match arr_sub_type {
        CPDType::BooleanType => {
            let res = compiler.emit_load_int_zero_extend(elem_pointer, Size::boolean());
            compiler.emit_stack_store_int(StackPos::AfterFromEnd(0), res);
        }
        CPDType::ByteType => {
            let res = compiler.emit_load_int_sign_extend(elem_pointer, Size::byte());
            compiler.emit_stack_store_int(StackPos::AfterFromEnd(0), res);
        }
        CPDType::ShortType => {
            let res = compiler.emit_load_int_sign_extend(elem_pointer, Size::short());
            compiler.emit_stack_store_int(StackPos::AfterFromEnd(0), res);
        }
        CPDType::CharType => {
            let res = compiler.emit_load_int_zero_extend(elem_pointer, Size::short());
            compiler.emit_stack_store_int(StackPos::AfterFromEnd(0), res);
        }
        CPDType::IntType => {
            let res = compiler.emit_load_int(elem_pointer);
            compiler.emit_stack_store_int(StackPos::AfterFromEnd(0), res);
        }
        CPDType::LongType => {
            let res = compiler.emit_load_long(elem_pointer);
            compiler.emit_stack_store_long(StackPos::AfterFromEnd(0), res);
        }
        CPDType::FloatType => {
            let res = compiler.emit_load_float(elem_pointer);
            compiler.emit_stack_store_float(StackPos::AfterFromEnd(0), res);
        }
        CPDType::DoubleType => {
            let res = compiler.emit_load_double(elem_pointer);
            compiler.emit_stack_store_double(StackPos::AfterFromEnd(0), res);
        }
        CPDType::Class(_) |
        CPDType::Array { .. } => {
            let res = compiler.emit_load_pointer(elem_pointer);
            compiler.emit_stack_store_pointer(StackPos::AfterFromEnd(0), res);
        }
        CPDType::VoidType => {
            panic!()
        }
    }
}


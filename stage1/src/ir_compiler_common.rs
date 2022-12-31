use std::num::NonZeroUsize;
use another_jit_vm::{FramePointerOffset, IRMethodID};
use another_jit_vm_ir::compiler::Size;
use array_memory_layout::layout::ArrayMemoryLayout;
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use crate::ir_compiler_common::special::IRCompilerState;
use crate::native_compiler_common::{GeneralRegister, GeneralRegisterPart, ValueVectorPosition32, ValueVectorPosition64, VectorRegister};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum IntegerValue {
    MemoryFramePointerOffset {
        offset: FramePointerOffset
    },
    MemoryRIPRelative {
        offset_from_function_base: usize
    },
    VectorRegister {
        vector_register: VectorRegister,
        position: ValueVectorPosition32,
    },
    GeneralRegister {
        general_register: GeneralRegister,
        part: GeneralRegisterPart,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FloatValue {
    MemoryFramePointerOffset {
        offset: FramePointerOffset
    },
    MemoryRIPRelative {
        offset_from_function_base: usize
    },
    VectorRegister {
        vector_register: VectorRegister,
        position: ValueVectorPosition32,
    },
    GeneralRegister {
        general_register: GeneralRegister,
        part: GeneralRegisterPart,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DoubleValue {
    MemoryFramePointerOffset {
        offset: FramePointerOffset
    },
    MemoryRIPRelative {
        offset_from_function_base: usize
    },
    VectorRegister {
        vector_register: VectorRegister,
        position: ValueVectorPosition64,
    },
    GeneralRegister {
        general_register: GeneralRegister,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum LongValue {
    MemoryFramePointerOffset {
        offset: FramePointerOffset
    },
    MemoryRIPRelative {
        offset_from_function_base: usize
    },
    VectorRegister {
        vector_register: VectorRegister,
        position: ValueVectorPosition64,
    },
    GeneralRegister {
        general_register: GeneralRegister,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PointerValue {
    MemoryFramePointerOffset {
        offset: FramePointerOffset
    },
    MemoryRIPRelative {
        offset_from_function_base: usize
    },
    VectorRegister {
        vector_register: VectorRegister,
        position: ValueVectorPosition64,
    },
    GeneralRegister {
        general_register: GeneralRegister,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueStatusChange {
    StackValueMapPointer {
        depth: u16,
        //0 is element closest to local vars
        pointer_value: PointerValue,
    },
    StackValueMapLong {
        depth: u16,
        pointer_value: LongValue,
    },
    StackValueMapDouble {
        depth: u16,
        pointer_value: DoubleValue,
    },
    StackValueMapFloat {
        depth: u16,
        pointer_value: FloatValue,
    },
    StackValueMapInteger {
        depth: u16,
        pointer_value: IntegerValue,
    },
    StackValueUnMapPointer {
        depth: u16
    },
    StackValueUnMapLong {
        depth: u16
    },
    StackValueUnMapDouble {
        depth: u16
    },
    StackValueUnMapFloat {
        depth: u16
    },
    StackValueUnMapInteger {
        depth: u16
    },
    LocalVariableMapPointer {
        var_idx: u16,
        pointer_value: PointerValue,
    },
    LocalVariableMapLong {
        var_idx: u16,
        long_value: LongValue,
    },
    LocalVariableMapDouble {
        var_idx: u16,
        long_value: DoubleValue,
    },
    LocalVariableMapFloat {
        var_idx: u16,
        long_value: FloatValue,
    },
    LocalVariableMapInteger {
        var_idx: u16,
        long_value: IntegerValue,
    },
}


//point of tokens is so that when registers are saved to stack or etc. we can use these to refer to
// wherever the value in question is.

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PointerValueToken(u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct LongValueToken(u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct DoubleValueToken(u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FloatValueToken(u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IntegerValueToken(u32);

//creating a label id creates a branch to id used for
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BranchToLabelID(u32);

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct TargetLabelID(u32);

pub enum Stage1IRInstr {
    IRStart {
        ir_method_id: IRMethodID,
        method_id: MethodId,
        frame_size: usize,
    },
    MonitorEnter {
        java_pc: ByteCodeOffset,
        obj: PointerValueToken,
    },
}

pub mod special;
pub mod loads;
pub mod stack_loads;
pub mod stack_stores;
pub mod exit_checks;
pub mod branching;
pub mod addressing;
pub mod arithmetic;
pub mod constant;
pub mod array_loads;

const ONE: NonZeroUsize = NonZeroUsize::new(1).unwrap();

pub fn array_load_impl(compiler: &mut IRCompilerState, arr_sub_type: CPDType) {
    let array_layout = ArrayMemoryLayout::from_cpdtype(arr_sub_type);
    let elem_0_offset = array_layout.elem_0_entry_offset();
    let len_offset = array_layout.len_entry_offset();
    let array_elem_size = array_layout.elem_size();
    let index = compiler.emit_stack_load_int(0);
    let array_ref = compiler.emit_stack_load_pointer(1);
    compiler.emit_npe_check(array_ref);
    let len_pointer = compiler.emit_address_calculate_int(array_ref, index, len_offset, ONE);
    let len = compiler.emit_load_int_sign_extend(len_pointer, Size::int());
    compiler.emit_array_bounds_check(len, index);
    let elem_pointer = compiler.emit_address_calculate_int(array_ref, index, elem_0_offset, array_elem_size);
    match arr_sub_type {
        CPDType::BooleanType => {
            let res = compiler.emit_load_int_zero_extend(elem_pointer, Size::boolean());
            compiler.emit_stack_store_int(0, res);
        }
        CPDType::ByteType => {
            let res = compiler.emit_load_int_sign_extend(elem_pointer, Size::byte());
            compiler.emit_stack_store_int(0, res);
        }
        CPDType::ShortType => {
            let res = compiler.emit_load_int_sign_extend(elem_pointer, Size::short());
            compiler.emit_stack_store_int(0, res);
        }
        CPDType::CharType => {
            let res = compiler.emit_load_int_zero_extend(elem_pointer, Size::short());
            compiler.emit_stack_store_int(0, res);
        }
        CPDType::IntType => {
            let res = compiler.emit_load_int(elem_pointer);
            compiler.emit_stack_store_int(0, res);
        }
        CPDType::LongType => {
            let res = compiler.emit_load_long(elem_pointer);
            compiler.emit_stack_store_long(0, res);
        }
        CPDType::FloatType => {
            let res = compiler.emit_load_float(elem_pointer);
            compiler.emit_stack_store_float(0, res);
        }
        CPDType::DoubleType => {
            let res = compiler.emit_load_double(elem_pointer);
            compiler.emit_stack_store_double(0, res);
        }
        CPDType::Class(_) |
        CPDType::Array { .. } => {
            let res = compiler.emit_load_pointer(elem_pointer);
            compiler.emit_stack_store_pointer(0, res);
        }
        CPDType::VoidType => {
            panic!()
        }
    }
}
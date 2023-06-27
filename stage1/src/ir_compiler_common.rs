use std::num::NonZeroUsize;

use another_jit_vm::{FramePointerOffset, IRMethodID};


use rust_jvm_common::{ByteCodeOffset, MethodId};


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
pub enum ValueToken {
    Integer(IntegerValueToken),
    Float(FloatValueToken),
    Pointer(PointerValueToken),
    Long(LongValueToken),
    Double(DoubleValueToken),
    Top,
}

impl ValueToken {
    pub fn unwrap_pointer(&self) -> PointerValueToken {
        if let ValueToken::Pointer(pointer) = self{
            return *pointer
        }
        panic!()
    }

    pub fn unwrap_integer(&self) -> IntegerValueToken {
        todo!()
    }

    pub fn unwrap_long(&self) -> LongValueToken {
        todo!()
    }

    pub fn unwrap_double(&self) -> DoubleValueToken {
        todo!()
    }

    pub fn unwrap_float(&self) -> FloatValueToken {
        todo!()
    }
}

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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct TargetLabelIDInternal(u32);


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
    LoadPointer {
        from: PointerValueToken,
        to: PointerValueToken,
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

pub const ONE: NonZeroUsize = NonZeroUsize::new(1).unwrap();


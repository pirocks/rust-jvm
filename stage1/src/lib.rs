use another_jit_vm::{FramePointerOffset, IRMethodID};
use compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};
use rust_jvm_common::{ByteCodeOffset, MethodId};

//todo fix instanceof/checkcast
//todo fix class loaders
//todo make a get object class fast path

//todo maybe an r15 offset consts makes sense here as well

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum VectorRegister {
    ZMM0,
    ZMM1,
    ZMM2,
    ZMM3,
    ZMM4,
    ZMM5,
    ZMM6,
    ZMM7,
    ZMM8,
    ZMM9,
    ZMM10,
    ZMM11,
    ZMM12,
    ZMM13,
    ZMM14,
    ZMM15,
    ZMM16,
    ZMM17,
    ZMM18,
    ZMM19,
    ZMM20,
    ZMM21,
    ZMM22,
    ZMM23,
    ZMM24,
    ZMM25,
    ZMM26,
    ZMM27,
    ZMM28,
    ZMM29,
    ZMM30,
    ZMM31,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueVectorPosition32 {
    Pos0,
    Pos1,
    Pos2,
    Pos3,
    Pos4,
    Pos5,
    Pos6,
    Pos7,
    Pos8,
    Pos9,
    Pos10,
    Pos11,
    Pos12,
    Pos13,
    Pos14,
    Pos15,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueVectorPosition64 {
    Pos0,
    Pos1,
    Pos2,
    Pos3,
    Pos4,
    Pos5,
    Pos6,
    Pos7,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GeneralRegister{
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15

}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GeneralRegisterPart{
    Lower,
    Upper
}

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
    GeneralRegister{
        general_register:GeneralRegister,
        part: GeneralRegisterPart
    }
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
    GeneralRegister{
        general_register:GeneralRegister,
        part: GeneralRegisterPart
    }
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
    GeneralRegister{
        general_register:GeneralRegister,
    }
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
    GeneralRegister{
        general_register:GeneralRegister,
    }
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
    GeneralRegister{
        general_register:GeneralRegister,
    }
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

pub mod frame_layout;

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

pub struct IRCompilerState {}

impl IRCompilerState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn emit_ir_start(&self) {
        todo!()
    }

    pub fn emit_monitor_enter(&self, obj: PointerValueToken) {
        todo!()
    }

    //todo have emits that return something return those values. 
}


pub fn compile_to_ir<'vm>(resolver: &impl MethodResolver<'vm>, method_frame_data: &JavaCompilerMethodAndFrameData, method_id: MethodId, ir_method_id: IRMethodID) -> Vec<Stage1IRInstr> {
    //todo use ir emit functions
    // let mut res = vec![];
    // res.push(Stage1IRInstr::IRStart {
    //     ir_method_id,
    //     method_id,
    //     frame_size: method_frame_data.full_frame_size(),
    // });
    // if method_frame_data.should_synchronize {
    //     if method_frame_data.is_static {
    //         res.push();
    //     } else {
    //         res.push(Stage1IRInstr::MonitorEnter {})
    //     }
    // }
    // res
    todo!()
}


pub struct CompilerState {}

impl CompilerState {
    pub fn new() -> Self {
        Self {}
    }
}
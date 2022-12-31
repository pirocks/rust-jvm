use another_jit_vm::{FramePointerOffset, IRMethodID};
use compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::code::CompressedInstructionInfo;

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
    pub fn new(
        method_id: MethodId,
        ir_method_id: IRMethodID,
        method_frame_data: &JavaCompilerMethodAndFrameData
    ) -> Self {
        Self {}
    }

    pub fn emit_ir_start(&mut self) {
        todo!()
    }

    pub fn emit_monitor_enter(&mut self, obj: PointerValueToken) {
        todo!()
    }

    pub fn emit_get_class_object(&mut self) -> PointerValueToken{
        todo!()
    }

    pub fn emit_load_arg_pointer(&mut self, arg_num: u16) -> PointerValueToken{
        todo!()
    }

    //todo have emits that return something return those values. 
}


pub fn compile_to_ir<'vm>(resolver: &impl MethodResolver<'vm>, method_frame_data: &JavaCompilerMethodAndFrameData, method_id: MethodId, ir_method_id: IRMethodID) -> Vec<Stage1IRInstr> {
    //todo use ir emit functions
    let mut compiler_state = IRCompilerState::new(method_id, ir_method_id, method_frame_data);
    compiler_state.emit_ir_start();
    if method_frame_data.should_synchronize {
        if method_frame_data.is_static {
            let class_object = compiler_state.emit_get_class_object();
            compiler_state.emit_monitor_enter(class_object);
        } else {
            let this_object = compiler_state.emit_load_arg_pointer(0);
            compiler_state.emit_monitor_enter(this_object);
        }
    }
    let code = resolver.get_compressed_code(method_id);
    for (java_pc, instr) in code.instructions.iter() {
        match instr.info {
            CompressedInstructionInfo::aaload => {
                todo!()
            }
            CompressedInstructionInfo::aastore => {
                todo!()
            }
            CompressedInstructionInfo::aconst_null => {
                todo!()
            }
            CompressedInstructionInfo::aload(_) => {
                todo!()
            }
            CompressedInstructionInfo::aload_0 => {
                todo!()
            }
            CompressedInstructionInfo::aload_1 => {
                todo!()
            }
            CompressedInstructionInfo::aload_2 => {
                todo!()
            }
            CompressedInstructionInfo::aload_3 => {
                todo!()
            }
            CompressedInstructionInfo::anewarray(_) => {
                todo!()
            }
            CompressedInstructionInfo::areturn => {
                todo!()
            }
            CompressedInstructionInfo::arraylength => {
                todo!()
            }
            CompressedInstructionInfo::astore(_) => {
                todo!()
            }
            CompressedInstructionInfo::astore_0 => {
                todo!()
            }
            CompressedInstructionInfo::astore_1 => {
                todo!()
            }
            CompressedInstructionInfo::astore_2 => {
                todo!()
            }
            CompressedInstructionInfo::astore_3 => {
                todo!()
            }
            CompressedInstructionInfo::athrow => {
                todo!()
            }
            CompressedInstructionInfo::baload => {
                todo!()
            }
            CompressedInstructionInfo::bastore => {
                todo!()
            }
            CompressedInstructionInfo::bipush(_) => {
                todo!()
            }
            CompressedInstructionInfo::caload => {
                todo!()
            }
            CompressedInstructionInfo::castore => {
                todo!()
            }
            CompressedInstructionInfo::checkcast(_) => {
                todo!()
            }
            CompressedInstructionInfo::d2f => {
                todo!()
            }
            CompressedInstructionInfo::d2i => {
                todo!()
            }
            CompressedInstructionInfo::d2l => {
                todo!()
            }
            CompressedInstructionInfo::dadd => {
                todo!()
            }
            CompressedInstructionInfo::daload => {
                todo!()
            }
            CompressedInstructionInfo::dastore => {
                todo!()
            }
            CompressedInstructionInfo::dcmpg => {
                todo!()
            }
            CompressedInstructionInfo::dcmpl => {
                todo!()
            }
            CompressedInstructionInfo::dconst_0 => {
                todo!()
            }
            CompressedInstructionInfo::dconst_1 => {
                todo!()
            }
            CompressedInstructionInfo::ddiv => {
                todo!()
            }
            CompressedInstructionInfo::dload(_) => {
                todo!()
            }
            CompressedInstructionInfo::dload_0 => {
                todo!()
            }
            CompressedInstructionInfo::dload_1 => {
                todo!()
            }
            CompressedInstructionInfo::dload_2 => {
                todo!()
            }
            CompressedInstructionInfo::dload_3 => {
                todo!()
            }
            CompressedInstructionInfo::dmul => {
                todo!()
            }
            CompressedInstructionInfo::dneg => {
                todo!()
            }
            CompressedInstructionInfo::drem => {
                todo!()
            }
            CompressedInstructionInfo::dreturn => {
                todo!()
            }
            CompressedInstructionInfo::dstore(_) => {
                todo!()
            }
            CompressedInstructionInfo::dstore_0 => {
                todo!()
            }
            CompressedInstructionInfo::dstore_1 => {
                todo!()
            }
            CompressedInstructionInfo::dstore_2 => {
                todo!()
            }
            CompressedInstructionInfo::dstore_3 => {
                todo!()
            }
            CompressedInstructionInfo::dsub => {
                todo!()
            }
            CompressedInstructionInfo::dup => {
                todo!()
            }
            CompressedInstructionInfo::dup_x1 => {
                todo!()
            }
            CompressedInstructionInfo::dup_x2 => {
                todo!()
            }
            CompressedInstructionInfo::dup2 => {
                todo!()
            }
            CompressedInstructionInfo::dup2_x1 => {
                todo!()
            }
            CompressedInstructionInfo::dup2_x2 => {
                todo!()
            }
            CompressedInstructionInfo::f2d => {
                todo!()
            }
            CompressedInstructionInfo::f2i => {
                todo!()
            }
            CompressedInstructionInfo::f2l => {
                todo!()
            }
            CompressedInstructionInfo::fadd => {
                todo!()
            }
            CompressedInstructionInfo::faload => {
                todo!()
            }
            CompressedInstructionInfo::fastore => {
                todo!()
            }
            CompressedInstructionInfo::fcmpg => {
                todo!()
            }
            CompressedInstructionInfo::fcmpl => {
                todo!()
            }
            CompressedInstructionInfo::fconst_0 => {
                todo!()
            }
            CompressedInstructionInfo::fconst_1 => {
                todo!()
            }
            CompressedInstructionInfo::fconst_2 => {
                todo!()
            }
            CompressedInstructionInfo::fdiv => {
                todo!()
            }
            CompressedInstructionInfo::fload(_) => {
                todo!()
            }
            CompressedInstructionInfo::fload_0 => {
                todo!()
            }
            CompressedInstructionInfo::fload_1 => {
                todo!()
            }
            CompressedInstructionInfo::fload_2 => {
                todo!()
            }
            CompressedInstructionInfo::fload_3 => {
                todo!()
            }
            CompressedInstructionInfo::fmul => {
                todo!()
            }
            CompressedInstructionInfo::fneg => {
                todo!()
            }
            CompressedInstructionInfo::frem => {
                todo!()
            }
            CompressedInstructionInfo::freturn => {
                todo!()
            }
            CompressedInstructionInfo::fstore(_) => {
                todo!()
            }
            CompressedInstructionInfo::fstore_0 => {
                todo!()
            }
            CompressedInstructionInfo::fstore_1 => {
                todo!()
            }
            CompressedInstructionInfo::fstore_2 => {
                todo!()
            }
            CompressedInstructionInfo::fstore_3 => {
                todo!()
            }
            CompressedInstructionInfo::fsub => {
                todo!()
            }
            CompressedInstructionInfo::getfield { .. } => {
                todo!()
            }
            CompressedInstructionInfo::getstatic { .. } => {
                todo!()
            }
            CompressedInstructionInfo::goto_(_) => {
                todo!()
            }
            CompressedInstructionInfo::goto_w(_) => {
                todo!()
            }
            CompressedInstructionInfo::i2b => {
                todo!()
            }
            CompressedInstructionInfo::i2c => {
                todo!()
            }
            CompressedInstructionInfo::i2d => {
                todo!()
            }
            CompressedInstructionInfo::i2f => {
                todo!()
            }
            CompressedInstructionInfo::i2l => {
                todo!()
            }
            CompressedInstructionInfo::i2s => {
                todo!()
            }
            CompressedInstructionInfo::iadd => {
                todo!()
            }
            CompressedInstructionInfo::iaload => {
                todo!()
            }
            CompressedInstructionInfo::iand => {
                todo!()
            }
            CompressedInstructionInfo::iastore => {
                todo!()
            }
            CompressedInstructionInfo::iconst_m1 => {
                todo!()
            }
            CompressedInstructionInfo::iconst_0 => {
                todo!()
            }
            CompressedInstructionInfo::iconst_1 => {
                todo!()
            }
            CompressedInstructionInfo::iconst_2 => {
                todo!()
            }
            CompressedInstructionInfo::iconst_3 => {
                todo!()
            }
            CompressedInstructionInfo::iconst_4 => {
                todo!()
            }
            CompressedInstructionInfo::iconst_5 => {
                todo!()
            }
            CompressedInstructionInfo::idiv => {
                todo!()
            }
            CompressedInstructionInfo::if_acmpeq(_) => {
                todo!()
            }
            CompressedInstructionInfo::if_acmpne(_) => {
                todo!()
            }
            CompressedInstructionInfo::if_icmpeq(_) => {
                todo!()
            }
            CompressedInstructionInfo::if_icmpne(_) => {
                todo!()
            }
            CompressedInstructionInfo::if_icmplt(_) => {
                todo!()
            }
            CompressedInstructionInfo::if_icmpge(_) => {
                todo!()
            }
            CompressedInstructionInfo::if_icmpgt(_) => {
                todo!()
            }
            CompressedInstructionInfo::if_icmple(_) => {
                todo!()
            }
            CompressedInstructionInfo::ifeq(_) => {
                todo!()
            }
            CompressedInstructionInfo::ifne(_) => {
                todo!()
            }
            CompressedInstructionInfo::iflt(_) => {
                todo!()
            }
            CompressedInstructionInfo::ifge(_) => {
                todo!()
            }
            CompressedInstructionInfo::ifgt(_) => {
                todo!()
            }
            CompressedInstructionInfo::ifle(_) => {
                todo!()
            }
            CompressedInstructionInfo::ifnonnull(_) => {
                todo!()
            }
            CompressedInstructionInfo::ifnull(_) => {
                todo!()
            }
            CompressedInstructionInfo::iinc(_) => {
                todo!()
            }
            CompressedInstructionInfo::iload(_) => {
                todo!()
            }
            CompressedInstructionInfo::iload_0 => {
                todo!()
            }
            CompressedInstructionInfo::iload_1 => {
                todo!()
            }
            CompressedInstructionInfo::iload_2 => {
                todo!()
            }
            CompressedInstructionInfo::iload_3 => {
                todo!()
            }
            CompressedInstructionInfo::imul => {
                todo!()
            }
            CompressedInstructionInfo::ineg => {
                todo!()
            }
            CompressedInstructionInfo::instanceof(_) => {
                todo!()
            }
            CompressedInstructionInfo::invokedynamic(_) => {
                todo!()
            }
            CompressedInstructionInfo::invokeinterface { .. } => {
                todo!()
            }
            CompressedInstructionInfo::invokespecial { .. } => {
                todo!()
            }
            CompressedInstructionInfo::invokestatic { .. } => {
                todo!()
            }
            CompressedInstructionInfo::invokevirtual { .. } => {
                todo!()
            }
            CompressedInstructionInfo::ior => {
                todo!()
            }
            CompressedInstructionInfo::irem => {
                todo!()
            }
            CompressedInstructionInfo::ireturn => {
                todo!()
            }
            CompressedInstructionInfo::ishl => {
                todo!()
            }
            CompressedInstructionInfo::ishr => {
                todo!()
            }
            CompressedInstructionInfo::istore(_) => {
                todo!()
            }
            CompressedInstructionInfo::istore_0 => {
                todo!()
            }
            CompressedInstructionInfo::istore_1 => {
                todo!()
            }
            CompressedInstructionInfo::istore_2 => {
                todo!()
            }
            CompressedInstructionInfo::istore_3 => {
                todo!()
            }
            CompressedInstructionInfo::isub => {
                todo!()
            }
            CompressedInstructionInfo::iushr => {
                todo!()
            }
            CompressedInstructionInfo::ixor => {
                todo!()
            }
            CompressedInstructionInfo::jsr(_) => {
                todo!()
            }
            CompressedInstructionInfo::jsr_w(_) => {
                todo!()
            }
            CompressedInstructionInfo::l2d => {
                todo!()
            }
            CompressedInstructionInfo::l2f => {
                todo!()
            }
            CompressedInstructionInfo::l2i => {
                todo!()
            }
            CompressedInstructionInfo::ladd => {
                todo!()
            }
            CompressedInstructionInfo::laload => {
                todo!()
            }
            CompressedInstructionInfo::land => {
                todo!()
            }
            CompressedInstructionInfo::lastore => {
                todo!()
            }
            CompressedInstructionInfo::lcmp => {
                todo!()
            }
            CompressedInstructionInfo::lconst_0 => {
                todo!()
            }
            CompressedInstructionInfo::lconst_1 => {
                todo!()
            }
            CompressedInstructionInfo::ldc(_) => {
                todo!()
            }
            CompressedInstructionInfo::ldc_w(_) => {
                todo!()
            }
            CompressedInstructionInfo::ldc2_w(_) => {
                todo!()
            }
            CompressedInstructionInfo::ldiv => {
                todo!()
            }
            CompressedInstructionInfo::lload(_) => {
                todo!()
            }
            CompressedInstructionInfo::lload_0 => {
                todo!()
            }
            CompressedInstructionInfo::lload_1 => {
                todo!()
            }
            CompressedInstructionInfo::lload_2 => {
                todo!()
            }
            CompressedInstructionInfo::lload_3 => {
                todo!()
            }
            CompressedInstructionInfo::lmul => {
                todo!()
            }
            CompressedInstructionInfo::lneg => {
                todo!()
            }
            CompressedInstructionInfo::lookupswitch(_) => {
                todo!()
            }
            CompressedInstructionInfo::lor => {
                todo!()
            }
            CompressedInstructionInfo::lrem => {
                todo!()
            }
            CompressedInstructionInfo::lreturn => {
                todo!()
            }
            CompressedInstructionInfo::lshl => {
                todo!()
            }
            CompressedInstructionInfo::lshr => {
                todo!()
            }
            CompressedInstructionInfo::lstore(_) => {
                todo!()
            }
            CompressedInstructionInfo::lstore_0 => {
                todo!()
            }
            CompressedInstructionInfo::lstore_1 => {
                todo!()
            }
            CompressedInstructionInfo::lstore_2 => {
                todo!()
            }
            CompressedInstructionInfo::lstore_3 => {
                todo!()
            }
            CompressedInstructionInfo::lsub => {
                todo!()
            }
            CompressedInstructionInfo::lushr => {
                todo!()
            }
            CompressedInstructionInfo::lxor => {
                todo!()
            }
            CompressedInstructionInfo::monitorenter => {
                todo!()
            }
            CompressedInstructionInfo::monitorexit => {
                todo!()
            }
            CompressedInstructionInfo::multianewarray { .. } => {
                todo!()
            }
            CompressedInstructionInfo::new(_) => {
                todo!()
            }
            CompressedInstructionInfo::newarray(_) => {
                todo!()
            }
            CompressedInstructionInfo::nop => {
                todo!()
            }
            CompressedInstructionInfo::pop => {
                todo!()
            }
            CompressedInstructionInfo::pop2 => {
                todo!()
            }
            CompressedInstructionInfo::putfield { .. } => {
                todo!()
            }
            CompressedInstructionInfo::putstatic { .. } => {
                todo!()
            }
            CompressedInstructionInfo::ret(_) => {
                todo!()
            }
            CompressedInstructionInfo::return_ => {
                todo!()
            }
            CompressedInstructionInfo::saload => {
                todo!()
            }
            CompressedInstructionInfo::sastore => {
                todo!()
            }
            CompressedInstructionInfo::sipush(_) => {
                todo!()
            }
            CompressedInstructionInfo::swap => {
                todo!()
            }
            CompressedInstructionInfo::tableswitch(_) => {
                todo!()
            }
            CompressedInstructionInfo::wide(_) => {
                todo!()
            }
            CompressedInstructionInfo::EndOfCode => {
                todo!()
            }
        }
    }
    res
}


pub struct CompilerState {}

impl CompilerState {
    pub fn new() -> Self {
        Self {}
    }
}
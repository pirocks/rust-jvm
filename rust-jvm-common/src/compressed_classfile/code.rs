use std::collections::HashMap;
use std::num::NonZeroU8;

use itertools::Either;
use wtf8::Wtf8Buf;

use crate::classfile::{Atype, CPIndex, IInc, LookupSwitch, SameFrame, TableSwitch, Wide};
use crate::compressed_classfile::{CFieldDescriptor, CMethodDescriptor, CPDType, CPRefType};
use crate::compressed_classfile::names::{CClassName, FieldName, MethodName};
use crate::vtype::VType;

pub type CInstruction = CompressedInstruction;

#[derive(Debug, Clone)]
pub struct CompressedInstruction {
    pub offset: u16,
    pub instruction_size: u16,
    pub info: CompressedInstructionInfo,
}

pub type CInstructionInfo = CompressedInstructionInfo;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum CompressedInstructionInfo {
    aaload,
    aastore,
    aconst_null,
    aload(u8),
    aload_0,
    aload_1,
    aload_2,
    aload_3,
    anewarray(CPDType),
    areturn,
    arraylength,
    astore(u8),
    astore_0,
    astore_1,
    astore_2,
    astore_3,
    athrow,
    baload,
    bastore,
    bipush(u8),
    caload,
    castore,
    checkcast(CPDType),
    d2f,
    d2i,
    d2l,
    dadd,
    daload,
    dastore,
    dcmpg,
    dcmpl,
    dconst_0,
    dconst_1,
    ddiv,
    dload(u8),
    dload_0,
    dload_1,
    dload_2,
    dload_3,
    dmul,
    dneg,
    drem,
    dreturn,
    dstore(u8),
    dstore_0,
    dstore_1,
    dstore_2,
    dstore_3,
    dsub,
    dup,
    dup_x1,
    dup_x2,
    dup2,
    dup2_x1,
    dup2_x2,
    f2d,
    f2i,
    f2l,
    fadd,
    faload,
    fastore,
    fcmpg,
    fcmpl,
    fconst_0,
    fconst_1,
    fconst_2,
    fdiv,
    fload(u8),
    fload_0,
    fload_1,
    fload_2,
    fload_3,
    fmul,
    fneg,
    frem,
    freturn,
    fstore(u8),
    fstore_0,
    fstore_1,
    fstore_2,
    fstore_3,
    fsub,
    getfield {
        name: FieldName,
        desc: CFieldDescriptor,
        target_class: CClassName,
    },
    getstatic {
        name: FieldName,
        desc: CFieldDescriptor,
        target_class: CClassName,
    },
    goto_(i16),
    goto_w(i32),
    i2b,
    i2c,
    i2d,
    i2f,
    i2l,
    i2s,
    iadd,
    iaload,
    iand,
    iastore,
    iconst_m1,
    iconst_0,
    iconst_1,
    iconst_2,
    iconst_3,
    iconst_4,
    iconst_5,
    idiv,
    if_acmpeq(i16),
    if_acmpne(i16),
    if_icmpeq(i16),
    if_icmpne(i16),
    if_icmplt(i16),
    if_icmpge(i16),
    if_icmpgt(i16),
    if_icmple(i16),
    ifeq(i16),
    ifne(i16),
    iflt(i16),
    ifge(i16),
    ifgt(i16),
    ifle(i16),
    ifnonnull(i16),
    ifnull(i16),
    iinc(IInc),
    iload(u8),
    iload_0,
    iload_1,
    iload_2,
    iload_3,
    imul,
    ineg,
    instanceof(CPDType),
    invokedynamic(CPIndex),
    invokeinterface {
        method_name: MethodName,
        descriptor: CMethodDescriptor,
        classname_ref_type: CPRefType,
        count: NonZeroU8,
    },
    invokespecial {
        method_name: MethodName,
        descriptor: CMethodDescriptor,
        classname_ref_type: CPRefType,
    },
    invokestatic {
        method_name: MethodName,
        descriptor: CMethodDescriptor,
        classname_ref_type: CPRefType,
    },
    invokevirtual {
        method_name: MethodName,
        descriptor: CMethodDescriptor,
        classname_ref_type: CPRefType,
    },
    ior,
    irem,
    ireturn,
    ishl,
    ishr,
    istore(u8),
    istore_0,
    istore_1,
    istore_2,
    istore_3,
    isub,
    iushr,
    ixor,
    jsr(i16),
    jsr_w(i32),
    l2d,
    l2f,
    l2i,
    ladd,
    laload,
    land,
    lastore,
    lcmp,
    lconst_0,
    lconst_1,
    ldc(Either<CompressedLdcW, CompressedLdc2W>),
    ldc_w(CompressedLdcW),
    ldc2_w(CompressedLdc2W),
    ldiv,
    lload(u8),
    lload_0,
    lload_1,
    lload_2,
    lload_3,
    lmul,
    lneg,
    lookupswitch(LookupSwitch),
    lor,
    lrem,
    lreturn,
    lshl,
    lshr,
    lstore(u8),
    lstore_0,
    lstore_1,
    lstore_2,
    lstore_3,
    lsub,
    lushr,
    lxor,
    monitorenter,
    monitorexit,
    multianewarray {
        type_: CPDType,
        dimensions: NonZeroU8,
    },
    new(CClassName),
    newarray(Atype),
    nop,
    pop,
    pop2,
    putfield {
        name: FieldName,
        desc: CFieldDescriptor,
        target_class: CClassName,
    },
    putstatic {
        name: FieldName,
        desc: CFieldDescriptor,
        target_class: CClassName,
    },
    ret(u8),
    return_,
    saload,
    sastore,
    sipush(u16),
    swap,
    tableswitch(Box<TableSwitch>),
    wide(Wide),
    EndOfCode,
}


impl CInstructionInfo {
    pub fn size(&self, starting_offset: u16) -> u16 {
        match self {
            CompressedInstructionInfo::aaload => 1,
            CompressedInstructionInfo::aastore => 1,
            CompressedInstructionInfo::aconst_null => 1,
            CompressedInstructionInfo::aload(_) => 2,
            CompressedInstructionInfo::aload_0 => 1,
            CompressedInstructionInfo::aload_1 => 1,
            CompressedInstructionInfo::aload_2 => 1,
            CompressedInstructionInfo::aload_3 => 1,
            CompressedInstructionInfo::anewarray(_) => 3,
            CompressedInstructionInfo::areturn => 1,
            CompressedInstructionInfo::arraylength => 1,
            CompressedInstructionInfo::astore(_) => 2,
            CompressedInstructionInfo::astore_0 => 1,
            CompressedInstructionInfo::astore_1 => 1,
            CompressedInstructionInfo::astore_2 => 1,
            CompressedInstructionInfo::astore_3 => 1,
            CompressedInstructionInfo::athrow => 1,
            CompressedInstructionInfo::baload => 1,
            CompressedInstructionInfo::bastore => 1,
            CompressedInstructionInfo::bipush(_) => 2,
            CompressedInstructionInfo::caload => 1,
            CompressedInstructionInfo::castore => 1,
            CompressedInstructionInfo::checkcast(_) => 3,
            CompressedInstructionInfo::d2f => 1,
            CompressedInstructionInfo::d2i => 1,
            CompressedInstructionInfo::d2l => 1,
            CompressedInstructionInfo::dadd => 1,
            CompressedInstructionInfo::daload => 1,
            CompressedInstructionInfo::dastore => 1,
            CompressedInstructionInfo::dcmpg => 1,
            CompressedInstructionInfo::dcmpl => 1,
            CompressedInstructionInfo::dconst_0 => 1,
            CompressedInstructionInfo::dconst_1 => 1,
            CompressedInstructionInfo::ddiv => 1,
            CompressedInstructionInfo::dload(_) => 2,
            CompressedInstructionInfo::dload_0 => 1,
            CompressedInstructionInfo::dload_1 => 1,
            CompressedInstructionInfo::dload_2 => 1,
            CompressedInstructionInfo::dload_3 => 1,
            CompressedInstructionInfo::dmul => 1,
            CompressedInstructionInfo::dneg => 1,
            CompressedInstructionInfo::drem => 1,
            CompressedInstructionInfo::dreturn => 1,
            CompressedInstructionInfo::dstore(_) => 2,
            CompressedInstructionInfo::dstore_0 => 1,
            CompressedInstructionInfo::dstore_1 => 1,
            CompressedInstructionInfo::dstore_2 => 1,
            CompressedInstructionInfo::dstore_3 => 1,
            CompressedInstructionInfo::dsub => 1,
            CompressedInstructionInfo::dup => 1,
            CompressedInstructionInfo::dup_x1 => 1,
            CompressedInstructionInfo::dup_x2 => 1,
            CompressedInstructionInfo::dup2 => 1,
            CompressedInstructionInfo::dup2_x1 => 1,
            CompressedInstructionInfo::dup2_x2 => 1,
            CompressedInstructionInfo::f2d => 1,
            CompressedInstructionInfo::f2i => 1,
            CompressedInstructionInfo::f2l => 1,
            CompressedInstructionInfo::fadd => 1,
            CompressedInstructionInfo::faload => 1,
            CompressedInstructionInfo::fastore => 1,
            CompressedInstructionInfo::fcmpg => 1,
            CompressedInstructionInfo::fcmpl => 1,
            CompressedInstructionInfo::fconst_0 => 1,
            CompressedInstructionInfo::fconst_1 => 1,
            CompressedInstructionInfo::fconst_2 => 1,
            CompressedInstructionInfo::fdiv => 1,
            CompressedInstructionInfo::fload(_) => 2,
            CompressedInstructionInfo::fload_0 => 1,
            CompressedInstructionInfo::fload_1 => 1,
            CompressedInstructionInfo::fload_2 => 1,
            CompressedInstructionInfo::fload_3 => 1,
            CompressedInstructionInfo::fmul => 1,
            CompressedInstructionInfo::fneg => 1,
            CompressedInstructionInfo::frem => 1,
            CompressedInstructionInfo::freturn => 1,
            CompressedInstructionInfo::fstore(_) => 2,
            CompressedInstructionInfo::fstore_0 => 1,
            CompressedInstructionInfo::fstore_1 => 1,
            CompressedInstructionInfo::fstore_2 => 1,
            CompressedInstructionInfo::fstore_3 => 1,
            CompressedInstructionInfo::fsub => 1,
            CompressedInstructionInfo::getfield { .. } => 3,
            CompressedInstructionInfo::getstatic { .. } => 3,
            CompressedInstructionInfo::goto_(_) => 3,
            CompressedInstructionInfo::goto_w(_) => 5,
            CompressedInstructionInfo::i2b => 1,
            CompressedInstructionInfo::i2c => 1,
            CompressedInstructionInfo::i2d => 1,
            CompressedInstructionInfo::i2f => 1,
            CompressedInstructionInfo::i2l => 1,
            CompressedInstructionInfo::i2s => 1,
            CompressedInstructionInfo::iadd => 1,
            CompressedInstructionInfo::iaload => 1,
            CompressedInstructionInfo::iand => 1,
            CompressedInstructionInfo::iastore => 1,
            CompressedInstructionInfo::iconst_m1 => 1,
            CompressedInstructionInfo::iconst_0 => 1,
            CompressedInstructionInfo::iconst_1 => 1,
            CompressedInstructionInfo::iconst_2 => 1,
            CompressedInstructionInfo::iconst_3 => 1,
            CompressedInstructionInfo::iconst_4 => 1,
            CompressedInstructionInfo::iconst_5 => 1,
            CompressedInstructionInfo::idiv => 1,
            CompressedInstructionInfo::if_acmpeq(_) => 3,
            CompressedInstructionInfo::if_acmpne(_) => 3,
            CompressedInstructionInfo::if_icmpeq(_) => 3,
            CompressedInstructionInfo::if_icmpne(_) => 3,
            CompressedInstructionInfo::if_icmplt(_) => 3,
            CompressedInstructionInfo::if_icmpge(_) => 3,
            CompressedInstructionInfo::if_icmpgt(_) => 3,
            CompressedInstructionInfo::if_icmple(_) => 3,
            CompressedInstructionInfo::ifeq(_) => 3,
            CompressedInstructionInfo::ifne(_) => 3,
            CompressedInstructionInfo::iflt(_) => 3,
            CompressedInstructionInfo::ifge(_) => 3,
            CompressedInstructionInfo::ifgt(_) => 3,
            CompressedInstructionInfo::ifle(_) => 3,
            CompressedInstructionInfo::ifnonnull(_) => 3,
            CompressedInstructionInfo::ifnull(_) => 3,
            CompressedInstructionInfo::iinc(_) => 3,
            CompressedInstructionInfo::iload(_) => 2,
            CompressedInstructionInfo::iload_0 => 1,
            CompressedInstructionInfo::iload_1 => 1,
            CompressedInstructionInfo::iload_2 => 1,
            CompressedInstructionInfo::iload_3 => 1,
            CompressedInstructionInfo::imul => 1,
            CompressedInstructionInfo::ineg => 1,
            CompressedInstructionInfo::instanceof(_) => 3,
            CompressedInstructionInfo::invokedynamic(_) => 5,
            CompressedInstructionInfo::invokeinterface { .. } => 5,
            CompressedInstructionInfo::invokespecial { .. } => 3,
            CompressedInstructionInfo::invokestatic { .. } => 3,
            CompressedInstructionInfo::invokevirtual { .. } => 3,
            CompressedInstructionInfo::ior => 1,
            CompressedInstructionInfo::irem => 1,
            CompressedInstructionInfo::ireturn => 1,
            CompressedInstructionInfo::ishl => 1,
            CompressedInstructionInfo::ishr => 1,
            CompressedInstructionInfo::istore(_) => 2,
            CompressedInstructionInfo::istore_0 => 1,
            CompressedInstructionInfo::istore_1 => 1,
            CompressedInstructionInfo::istore_2 => 1,
            CompressedInstructionInfo::istore_3 => 1,
            CompressedInstructionInfo::isub => 1,
            CompressedInstructionInfo::iushr => 1,
            CompressedInstructionInfo::ixor => 1,
            CompressedInstructionInfo::jsr(_) => 3,
            CompressedInstructionInfo::jsr_w(_) => 5,
            CompressedInstructionInfo::l2d => 1,
            CompressedInstructionInfo::l2f => 1,
            CompressedInstructionInfo::l2i => 1,
            CompressedInstructionInfo::ladd => 1,
            CompressedInstructionInfo::laload => 1,
            CompressedInstructionInfo::land => 1,
            CompressedInstructionInfo::lastore => 1,
            CompressedInstructionInfo::lcmp => 1,
            CompressedInstructionInfo::lconst_0 => 1,
            CompressedInstructionInfo::lconst_1 => 1,
            CompressedInstructionInfo::ldc(_) => 2,
            CompressedInstructionInfo::ldc_w(_) => 3,
            CompressedInstructionInfo::ldc2_w(_) => 3,
            CompressedInstructionInfo::ldiv => 1,
            CompressedInstructionInfo::lload(_) => 2,
            CompressedInstructionInfo::lload_0 => 1,
            CompressedInstructionInfo::lload_1 => 1,
            CompressedInstructionInfo::lload_2 => 1,
            CompressedInstructionInfo::lload_3 => 1,
            CompressedInstructionInfo::lmul => 1,
            CompressedInstructionInfo::lneg => 1,
            CompressedInstructionInfo::lookupswitch(LookupSwitch { pairs, default: _ }) => {
                let pad_and_bytecode = 4 - (starting_offset % 4);
                pad_and_bytecode + 4 + 4 + pairs.len() as u16 * 8
            }
            CompressedInstructionInfo::lor => 1,
            CompressedInstructionInfo::lrem => 1,
            CompressedInstructionInfo::lreturn => 1,
            CompressedInstructionInfo::lshl => 1,
            CompressedInstructionInfo::lshr => 1,
            CompressedInstructionInfo::lstore(_) => 2,
            CompressedInstructionInfo::lstore_0 => 1,
            CompressedInstructionInfo::lstore_1 => 1,
            CompressedInstructionInfo::lstore_2 => 1,
            CompressedInstructionInfo::lstore_3 => 1,
            CompressedInstructionInfo::lsub => 1,
            CompressedInstructionInfo::lushr => 1,
            CompressedInstructionInfo::lxor => 1,
            CompressedInstructionInfo::monitorenter => 1,
            CompressedInstructionInfo::monitorexit => 1,
            CompressedInstructionInfo::multianewarray { .. } => 4,
            CompressedInstructionInfo::new(_) => 3,
            CompressedInstructionInfo::newarray(_) => 2,
            CompressedInstructionInfo::nop => 1,
            CompressedInstructionInfo::pop => 1,
            CompressedInstructionInfo::pop2 => 1,
            CompressedInstructionInfo::putfield { .. } => 3,
            CompressedInstructionInfo::putstatic { .. } => 3,
            CompressedInstructionInfo::ret(_) => 2,
            CompressedInstructionInfo::return_ => 1,
            CompressedInstructionInfo::saload => 1,
            CompressedInstructionInfo::sastore => 1,
            CompressedInstructionInfo::sipush(_) => 3,
            CompressedInstructionInfo::swap => 1,
            CompressedInstructionInfo::tableswitch(box TableSwitch { default: _, low: _, high: _, offsets }) => {
                let pad_and_bytecode = 4 - (starting_offset % 4);
                pad_and_bytecode + 4 + 4 + 4 + offsets.len() as u16 * 4
            }
            CompressedInstructionInfo::wide(wide) => {
                match wide {
                    Wide::Iload(_) => 4,
                    Wide::Fload(_) => 4,
                    Wide::Aload(_) => 4,
                    Wide::Lload(_) => 4,
                    Wide::Dload(_) => 4,
                    Wide::Istore(_) => 4,
                    Wide::Fstore(_) => 4,
                    Wide::Astore(_) => 4,
                    Wide::Lstore(_) => 4,
                    Wide::Dstore(_) => 4,
                    Wide::Ret(_) => 4,
                    Wide::IInc(_) => 6
                }
            }
            CompressedInstructionInfo::EndOfCode => 0,
        }
    }
}


#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LiveObjectIndex(pub usize);

#[derive(Debug, Clone)]
pub enum CompressedLdcW {
    String {
        str: Wtf8Buf
    },
    Class {
        type_: CPDType
    },
    Float {
        float: f32
    },
    Integer {
        integer: i32
    },
    MethodType {},
    MethodHandle {},
    LiveObject(LiveObjectIndex),
}

#[derive(Debug, Clone)]
pub enum CompressedLdc2W {
    Long(i64),
    Double(f64),
}

pub struct CompressedInvokeInterface {}

#[derive(Clone)]
pub struct CompressedCode {
    pub instructions: HashMap<u16, CompressedInstruction>,
    pub max_locals: u16,
    pub max_stack: u16,
    pub exception_table: Vec<CompressedExceptionTableElem>,
    pub stack_map_table: Vec<CompressedStackMapFrame>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub enum CompressedStackMapFrame {
    SameFrame(SameFrame),
    SameLocals1StackItemFrame(CompressedSameLocals1StackItemFrame),
    SameLocals1StackItemFrameExtended(CompressedSameLocals1StackItemFrameExtended),
    ChopFrame(CompressedChopFrame),
    SameFrameExtended(CompressedSameFrameExtended),
    AppendFrame(CompressedAppendFrame),
    FullFrame(CompressedFullFrame),
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct CompressedSameLocals1StackItemFrame {
    pub offset_delta: u16,
    pub stack: VType,
}


#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct CompressedSameLocals1StackItemFrameExtended {
    pub offset_delta: u16,
    pub stack: VType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct CompressedChopFrame {
    //todo why is this a thing
    pub offset_delta: u16,
    pub k_frames_to_chop: u8,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct CompressedSameFrameExtended {
    //todo why is this is a thing as well, since its same as non-compressed
    pub offset_delta: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct CompressedAppendFrame {
    pub offset_delta: u16,
    pub locals: Vec<VType>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct CompressedFullFrame {
    pub offset_delta: u16,
    pub number_of_locals: u16,
    pub locals: Vec<VType>,
    pub number_of_stack_items: u16,
    pub stack: Vec<VType>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Clone)]
pub struct CompressedExceptionTableElem {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: Option<CClassName>,
}

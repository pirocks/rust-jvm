use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU8;

use itertools::Either;
use wtf8::Wtf8Buf;

use crate::ByteCodeOffset;
use crate::classfile::{Atype, ChopFrame, CPIndex, IInc, LookupSwitch, SameFrame, SameFrameExtended, TableSwitch, Wide};
use crate::compressed_classfile::{CMethodDescriptor, CompressedClassfileStringPool, CPDType, CPRefType};
use crate::compressed_classfile::class_names::CClassName;
use crate::compressed_classfile::compressed_descriptors::CFieldDescriptor;
use crate::compressed_classfile::field_names::FieldName;
use crate::compressed_classfile::method_names::MethodName;
use crate::vtype::VType;

pub type CInstruction = CompressedInstruction;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CompressedInstruction {
    pub offset: ByteCodeOffset,
    pub instruction_size: u16,
    pub info: CompressedInstructionInfo,
}

impl CompressedInstructionInfo {
    pub fn better_debug_string(&self, string_pool: &CompressedClassfileStringPool) -> String {
        match self {
            CompressedInstructionInfo::aaload => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::aastore => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::aconst_null => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::aload(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::aload_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::aload_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::aload_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::aload_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::anewarray(type_) => {
                format!("anewarray:{}", type_.jvm_representation(string_pool))
            }
            CompressedInstructionInfo::areturn => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::arraylength => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::astore(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::astore_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::astore_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::astore_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::astore_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::athrow => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::baload => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::bastore => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::bipush(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::caload => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::castore => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::checkcast(type_) => {
                format!("checkcast:{}", type_.jvm_representation(string_pool))
            }
            CompressedInstructionInfo::d2f => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::d2i => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::d2l => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dadd => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::daload => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dastore => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dcmpg => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dcmpl => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dconst_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dconst_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ddiv => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dload(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dload_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dload_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dload_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dload_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dmul => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dneg => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::drem => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dreturn => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dstore(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dstore_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dstore_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dstore_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dstore_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dsub => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dup => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dup_x1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dup_x2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dup2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dup2_x1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::dup2_x2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::f2d => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::f2i => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::f2l => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fadd => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::faload => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fastore => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fcmpg => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fcmpl => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fconst_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fconst_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fconst_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fdiv => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fload(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fload_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fload_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fload_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fload_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fmul => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fneg => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::frem => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::freturn => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fstore(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fstore_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fstore_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fstore_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fstore_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::fsub => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::getfield { name, desc, target_class } => {
                format!("getfield:{}/{}/{}", name.0.to_str(string_pool), desc.0.jvm_representation(string_pool), target_class.0.to_str(string_pool))
            }
            CompressedInstructionInfo::getstatic { name, desc, target_class } => {
                format!("getstatic:{}/{}/{}", name.0.to_str(string_pool), desc.0.jvm_representation(string_pool), target_class.0.to_str(string_pool))
            }
            CompressedInstructionInfo::putfield { name, desc, target_class } => {
                format!("putfield:{}/{}/{}", name.0.to_str(string_pool), desc.0.jvm_representation(string_pool), target_class.0.to_str(string_pool))
            }
            CompressedInstructionInfo::putstatic { name, desc, target_class } => {
                format!("putstatic:{}/{}/{}", name.0.to_str(string_pool), desc.0.jvm_representation(string_pool), target_class.0.to_str(string_pool))
            }
            CompressedInstructionInfo::goto_(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::goto_w(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::i2b => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::i2c => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::i2d => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::i2f => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::i2l => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::i2s => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iadd => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iaload => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iand => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iastore => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iconst_m1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iconst_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iconst_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iconst_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iconst_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iconst_4 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iconst_5 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::idiv => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::if_acmpeq(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::if_acmpne(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::if_icmpeq(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::if_icmpne(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::if_icmplt(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::if_icmpge(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::if_icmpgt(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::if_icmple(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ifeq(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ifne(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iflt(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ifge(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ifgt(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ifle(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ifnonnull(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ifnull(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iinc(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iload(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iload_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iload_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iload_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iload_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::imul => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ineg => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::instanceof(type_) => {
                format!("instanceof:{}", type_.jvm_representation(string_pool))
            }
            CompressedInstructionInfo::invokedynamic(_) => {
                "invokedynamic".to_string()
            }
            CompressedInstructionInfo::invokeinterface { method_name, descriptor, classname_ref_type, count } => {
                format!("invokeinterface:{}/{}/{}/{}", classname_ref_type.unwrap_name().0.to_str(string_pool), descriptor.jvm_representation(string_pool), method_name.0.to_str(string_pool), count)
            }
            CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                format!("invokespecial:{}/{}/{}", classname_ref_type.unwrap_name().0.to_str(string_pool), descriptor.jvm_representation(string_pool), method_name.0.to_str(string_pool))
            }
            CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                format!("invokestatic:{}/{}/{}", classname_ref_type.unwrap_name().0.to_str(string_pool), descriptor.jvm_representation(string_pool), method_name.0.to_str(string_pool))
            }
            CompressedInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
                format!("invokevirtual:{}/{}/{}", classname_ref_type.try_unwrap_name().map(|name| name.0.to_str(string_pool)).unwrap_or("array".to_string()), descriptor.jvm_representation(string_pool), method_name.0.to_str(string_pool))
            }
            CompressedInstructionInfo::ior => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::irem => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ireturn => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ishl => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ishr => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::istore(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::istore_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::istore_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::istore_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::istore_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::isub => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::iushr => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ixor => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::jsr(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::jsr_w(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::l2d => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::l2f => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::l2i => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ladd => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::laload => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::land => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lastore => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lcmp => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lconst_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lconst_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ldc(ldc_type) => {
                match ldc_type {
                    Either::Left(ldc_type) => {
                        match ldc_type {
                            CompressedLdcW::String { str } => {
                                format!("ldc:string:\"{}\"", str.to_string_lossy())
                            }
                            CompressedLdcW::Class { type_ } => {
                                format!("ldc:class:{}", type_.jvm_representation(string_pool))
                            }
                            CompressedLdcW::Float { float } => {
                                format!("ldc:float:{}", float)
                            }
                            CompressedLdcW::Integer { integer } => {
                                format!("ldc:integer:{}", integer)
                            }
                            CompressedLdcW::MethodType { .. } => {
                                todo!()
                            }
                            CompressedLdcW::MethodHandle { .. } => {
                                todo!()
                            }
                            CompressedLdcW::LiveObject(_) => {
                                todo!()
                            }
                        }
                    }
                    Either::Right(ldc_type) => {
                        match ldc_type {
                            CompressedLdc2W::Long(_) => {
                                todo!()
                            }
                            CompressedLdc2W::Double(double) => {
                                format!("ldc:double:{}", double)
                            }
                        }
                    }
                }
            }
            CompressedInstructionInfo::ldc_w(ldc_type) => {
                match ldc_type {
                    CompressedLdcW::String { str } => {
                        format!("ldc:string:\"{}\"", str.to_string_lossy())
                    }
                    CompressedLdcW::Class { type_ } => {
                        format!("ldc:class:\"{}\"", type_.jvm_representation(string_pool))
                    }
                    CompressedLdcW::Float { float } => {
                        format!("ldc:class:\"{}\"", float)
                    }
                    CompressedLdcW::Integer { integer } => {
                        format!("ldc_w:integer:\"{}\"", *integer)
                    }
                    CompressedLdcW::MethodType { .. } => {
                        todo!()
                    }
                    CompressedLdcW::MethodHandle { .. } => {
                        todo!()
                    }
                    CompressedLdcW::LiveObject(_) => {
                        todo!()
                    }
                }
            }
            CompressedInstructionInfo::ldc2_w(ldc_type) => {
                match ldc_type {
                    CompressedLdc2W::Long(long) => {
                        format!("ldc2_w:long:{}", long)
                    }
                    CompressedLdc2W::Double(double) => {
                        format!("ldc2_w:double:{}", double)
                    }
                }
            }
            CompressedInstructionInfo::ldiv => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lload(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lload_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lload_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lload_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lload_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lmul => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lneg => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lookupswitch(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lor => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lrem => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lreturn => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lshl => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lshr => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lstore(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lstore_0 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lstore_1 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lstore_2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lstore_3 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lsub => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lushr => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::lxor => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::monitorenter => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::monitorexit => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::multianewarray { .. } => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::new(a_type) => {
                format!("new/{}", a_type.0.to_str(string_pool))
            }
            CompressedInstructionInfo::newarray(a_type) => {
                format!("newarray/{:?}", a_type)
            }
            CompressedInstructionInfo::nop => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::pop => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::pop2 => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::ret(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::return_ => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::saload => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::sastore => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::sipush(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::swap => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::tableswitch(_) => {
                format!("{:?}", self)
            }
            CompressedInstructionInfo::wide(wide) => {
                match wide {
                    Wide::Iload(_) => {
                        todo!()
                    }
                    Wide::Fload(_) => {
                        todo!()
                    }
                    Wide::Aload(_) => {
                        todo!()
                    }
                    Wide::Lload(_) => {
                        todo!()
                    }
                    Wide::Dload(_) => {
                        todo!()
                    }
                    Wide::Istore(_) => {
                        todo!()
                    }
                    Wide::Fstore(_) => {
                        todo!()
                    }
                    Wide::Astore(_) => {
                        todo!()
                    }
                    Wide::Lstore(_) => {
                        todo!()
                    }
                    Wide::Dstore(_) => {
                        todo!()
                    }
                    Wide::Ret(_) => {
                        todo!()
                    }
                    Wide::IInc(_) => {
                        format!("{:?}", self)
                    }
                }
            }
            CompressedInstructionInfo::EndOfCode => {
                todo!()
            }
        }
    }
}

pub type CInstructionInfo = CompressedInstructionInfo;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    bipush(i8),
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
    getfield { name: FieldName, desc: CFieldDescriptor, target_class: CClassName },
    getstatic { name: FieldName, desc: CFieldDescriptor, target_class: CClassName },
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
    invokeinterface { method_name: MethodName, descriptor: CMethodDescriptor, classname_ref_type: CPRefType, count: NonZeroU8 },
    invokespecial { method_name: MethodName, descriptor: CMethodDescriptor, classname_ref_type: CPRefType },
    invokestatic { method_name: MethodName, descriptor: CMethodDescriptor, classname_ref_type: CPRefType },
    invokevirtual { method_name: MethodName, descriptor: CMethodDescriptor, classname_ref_type: CPRefType },
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
    multianewarray { type_: CPDType, dimensions: NonZeroU8 },
    new(CClassName),
    newarray(Atype),
    nop,
    pop,
    pop2,
    putfield { name: FieldName, desc: CFieldDescriptor, target_class: CClassName },
    putstatic { name: FieldName, desc: CFieldDescriptor, target_class: CClassName },
    ret(u8),
    return_,
    saload,
    sastore,
    sipush(i16),
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
            CompressedInstructionInfo::wide(wide) => match wide {
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
                Wide::IInc(_) => 6,
            },
            CompressedInstructionInfo::EndOfCode => 0,
        }
    }

    pub fn instruction_to_string_without_meta(&self) -> String {
        match self {
            CInstructionInfo::aaload => "aaload".to_string(),
            CInstructionInfo::aastore => "aastore".to_string(),
            CInstructionInfo::aconst_null => "aconst_null".to_string(),
            CInstructionInfo::aload(_) => "aload".to_string(),
            CInstructionInfo::aload_0 => "aload_0".to_string(),
            CInstructionInfo::aload_1 => "aload_1".to_string(),
            CInstructionInfo::aload_2 => "aload_2".to_string(),
            CInstructionInfo::aload_3 => "aload_3".to_string(),
            CInstructionInfo::anewarray(_) => "anewarray".to_string(),
            CInstructionInfo::areturn => "areturn".to_string(),
            CInstructionInfo::arraylength => "arraylength".to_string(),
            CInstructionInfo::astore(_) => "astore".to_string(),
            CInstructionInfo::astore_0 => "astore_0".to_string(),
            CInstructionInfo::astore_1 => "astore_1".to_string(),
            CInstructionInfo::astore_2 => "astore_2".to_string(),
            CInstructionInfo::astore_3 => "astore_3".to_string(),
            CInstructionInfo::athrow => "athrow".to_string(),
            CInstructionInfo::baload => "baload".to_string(),
            CInstructionInfo::bastore => "bastore".to_string(),
            CInstructionInfo::bipush(_) => "bipush".to_string(),
            CInstructionInfo::caload => "caload".to_string(),
            CInstructionInfo::castore => "castore".to_string(),
            CInstructionInfo::checkcast(_) => "checkcast".to_string(),
            CInstructionInfo::d2f => "d2f".to_string(),
            CInstructionInfo::d2i => "d2i".to_string(),
            CInstructionInfo::d2l => "d2l".to_string(),
            CInstructionInfo::dadd => "dadd".to_string(),
            CInstructionInfo::daload => "daload".to_string(),
            CInstructionInfo::dastore => "dastore".to_string(),
            CInstructionInfo::dcmpg => "dcmpg".to_string(),
            CInstructionInfo::dcmpl => "dcmpl".to_string(),
            CInstructionInfo::dconst_0 => "dconst_0".to_string(),
            CInstructionInfo::dconst_1 => "dconst_1".to_string(),
            CInstructionInfo::ddiv => "ddiv".to_string(),
            CInstructionInfo::dload(_) => "dload".to_string(),
            CInstructionInfo::dload_0 => "dload_0".to_string(),
            CInstructionInfo::dload_1 => "dload_1".to_string(),
            CInstructionInfo::dload_2 => "dload_2".to_string(),
            CInstructionInfo::dload_3 => "dload_3".to_string(),
            CInstructionInfo::dmul => "dmul".to_string(),
            CInstructionInfo::dneg => "dneg".to_string(),
            CInstructionInfo::drem => "drem".to_string(),
            CInstructionInfo::dreturn => "dreturn".to_string(),
            CInstructionInfo::dstore(_) => "dstore".to_string(),
            CInstructionInfo::dstore_0 => "dstore_0".to_string(),
            CInstructionInfo::dstore_1 => "dstore_1".to_string(),
            CInstructionInfo::dstore_2 => "dstore_2".to_string(),
            CInstructionInfo::dstore_3 => "dstore_3".to_string(),
            CInstructionInfo::dsub => "dsub".to_string(),
            CInstructionInfo::dup => "dup".to_string(),
            CInstructionInfo::dup_x1 => "dup_x1".to_string(),
            CInstructionInfo::dup_x2 => "dup_x2".to_string(),
            CInstructionInfo::dup2 => "dup2".to_string(),
            CInstructionInfo::dup2_x1 => "dup2_x1".to_string(),
            CInstructionInfo::dup2_x2 => "dup2_x2".to_string(),
            CInstructionInfo::f2d => "f2d".to_string(),
            CInstructionInfo::f2i => "f2i".to_string(),
            CInstructionInfo::f2l => "f2l".to_string(),
            CInstructionInfo::fadd => "fadd".to_string(),
            CInstructionInfo::faload => "faload".to_string(),
            CInstructionInfo::fastore => "fastore".to_string(),
            CInstructionInfo::fcmpg => "fcmpg".to_string(),
            CInstructionInfo::fcmpl => "fcmpl".to_string(),
            CInstructionInfo::fconst_0 => "fconst_0".to_string(),
            CInstructionInfo::fconst_1 => "fconst_1".to_string(),
            CInstructionInfo::fconst_2 => "fconst_2".to_string(),
            CInstructionInfo::fdiv => "fdiv".to_string(),
            CInstructionInfo::fload(_) => "fload".to_string(),
            CInstructionInfo::fload_0 => "fload_0".to_string(),
            CInstructionInfo::fload_1 => "fload_1".to_string(),
            CInstructionInfo::fload_2 => "fload_2".to_string(),
            CInstructionInfo::fload_3 => "fload_3".to_string(),
            CInstructionInfo::fmul => "fmul".to_string(),
            CInstructionInfo::fneg => "fneg".to_string(),
            CInstructionInfo::frem => "frem".to_string(),
            CInstructionInfo::freturn => "freturn".to_string(),
            CInstructionInfo::fstore(_) => "fstore".to_string(),
            CInstructionInfo::fstore_0 => "fstore_0".to_string(),
            CInstructionInfo::fstore_1 => "fstore_1".to_string(),
            CInstructionInfo::fstore_2 => "fstore_2".to_string(),
            CInstructionInfo::fstore_3 => "fstore_3".to_string(),
            CInstructionInfo::fsub => "fsub".to_string(),
            CInstructionInfo::getfield { .. } => "getfield".to_string(),
            CInstructionInfo::getstatic { .. } => "getstatic".to_string(),
            CInstructionInfo::goto_(_) => "goto_".to_string(),
            CInstructionInfo::goto_w(_) => "goto_w".to_string(),
            CInstructionInfo::i2b => "i2b".to_string(),
            CInstructionInfo::i2c => "i2c".to_string(),
            CInstructionInfo::i2d => "i2d".to_string(),
            CInstructionInfo::i2f => "i2f".to_string(),
            CInstructionInfo::i2l => "i2l".to_string(),
            CInstructionInfo::i2s => "i2s".to_string(),
            CInstructionInfo::iadd => "iadd".to_string(),
            CInstructionInfo::iaload => "iaload".to_string(),
            CInstructionInfo::iand => "iand".to_string(),
            CInstructionInfo::iastore => "iastore".to_string(),
            CInstructionInfo::iconst_m1 => "iconst_m1".to_string(),
            CInstructionInfo::iconst_0 => "iconst_0".to_string(),
            CInstructionInfo::iconst_1 => "iconst_1".to_string(),
            CInstructionInfo::iconst_2 => "iconst_2".to_string(),
            CInstructionInfo::iconst_3 => "iconst_3".to_string(),
            CInstructionInfo::iconst_4 => "iconst_4".to_string(),
            CInstructionInfo::iconst_5 => "iconst_5".to_string(),
            CInstructionInfo::idiv => "idiv".to_string(),
            CInstructionInfo::if_acmpeq(_) => "if_acmpeq".to_string(),
            CInstructionInfo::if_acmpne(_) => "if_acmpne".to_string(),
            CInstructionInfo::if_icmpeq(_) => "if_icmpeq".to_string(),
            CInstructionInfo::if_icmpne(_) => "if_icmpne".to_string(),
            CInstructionInfo::if_icmplt(_) => "if_icmplt".to_string(),
            CInstructionInfo::if_icmpge(_) => "if_icmpge".to_string(),
            CInstructionInfo::if_icmpgt(_) => "if_icmpgt".to_string(),
            CInstructionInfo::if_icmple(_) => "if_icmple".to_string(),
            CInstructionInfo::ifeq(_) => "ifeq".to_string(),
            CInstructionInfo::ifne(_) => "ifne".to_string(),
            CInstructionInfo::iflt(_) => "iflt".to_string(),
            CInstructionInfo::ifge(_) => "ifge".to_string(),
            CInstructionInfo::ifgt(_) => "ifgt".to_string(),
            CInstructionInfo::ifle(_) => "ifle".to_string(),
            CInstructionInfo::ifnonnull(_) => "ifnonnull".to_string(),
            CInstructionInfo::ifnull(_) => "ifnull".to_string(),
            CInstructionInfo::iinc(_) => "iinc".to_string(),
            CInstructionInfo::iload(_) => "iload".to_string(),
            CInstructionInfo::iload_0 => "iload_0".to_string(),
            CInstructionInfo::iload_1 => "iload_1".to_string(),
            CInstructionInfo::iload_2 => "iload_2".to_string(),
            CInstructionInfo::iload_3 => "iload_3".to_string(),
            CInstructionInfo::imul => "imul".to_string(),
            CInstructionInfo::ineg => "ineg".to_string(),
            CInstructionInfo::instanceof(_) => "instanceof".to_string(),
            CInstructionInfo::invokedynamic(_) => "invokedynamic".to_string(),
            CInstructionInfo::invokeinterface { .. } => "invokeinterface".to_string(),
            CInstructionInfo::invokespecial { .. } => "invokespecial".to_string(),
            CInstructionInfo::invokestatic { .. } => "invokestatic".to_string(),
            CInstructionInfo::invokevirtual { .. } => "invokevirtual".to_string(),
            CInstructionInfo::ior => "ior".to_string(),
            CInstructionInfo::irem => "irem".to_string(),
            CInstructionInfo::ireturn => "ireturn".to_string(),
            CInstructionInfo::ishl => "ishl".to_string(),
            CInstructionInfo::ishr => "ishr".to_string(),
            CInstructionInfo::istore(_) => "istore".to_string(),
            CInstructionInfo::istore_0 => "istore_0".to_string(),
            CInstructionInfo::istore_1 => "istore_1".to_string(),
            CInstructionInfo::istore_2 => "istore_2".to_string(),
            CInstructionInfo::istore_3 => "istore_3".to_string(),
            CInstructionInfo::isub => "isub".to_string(),
            CInstructionInfo::iushr => "iushr".to_string(),
            CInstructionInfo::ixor => "ixor".to_string(),
            CInstructionInfo::jsr(_) => "jsr".to_string(),
            CInstructionInfo::jsr_w(_) => "jsr_w".to_string(),
            CInstructionInfo::l2d => "l2d".to_string(),
            CInstructionInfo::l2f => "l2f".to_string(),
            CInstructionInfo::l2i => "l2i".to_string(),
            CInstructionInfo::ladd => "ladd".to_string(),
            CInstructionInfo::laload => "laload".to_string(),
            CInstructionInfo::land => "land".to_string(),
            CInstructionInfo::lastore => "lastore".to_string(),
            CInstructionInfo::lcmp => "lcmp".to_string(),
            CInstructionInfo::lconst_0 => "lconst_0".to_string(),
            CInstructionInfo::lconst_1 => "lconst_1".to_string(),
            CInstructionInfo::ldc(_) => "ldc".to_string(),
            CInstructionInfo::ldc_w(_) => "ldc_w".to_string(),
            CInstructionInfo::ldc2_w(_) => "ldc2_w".to_string(),
            CInstructionInfo::ldiv => "ldiv".to_string(),
            CInstructionInfo::lload(_) => "lload".to_string(),
            CInstructionInfo::lload_0 => "lload_0".to_string(),
            CInstructionInfo::lload_1 => "lload_1".to_string(),
            CInstructionInfo::lload_2 => "lload_2".to_string(),
            CInstructionInfo::lload_3 => "lload_3".to_string(),
            CInstructionInfo::lmul => "lmul".to_string(),
            CInstructionInfo::lneg => "lneg".to_string(),
            CInstructionInfo::lookupswitch(_) => "lookupswitch".to_string(),
            CInstructionInfo::lor => "lor".to_string(),
            CInstructionInfo::lrem => "lrem".to_string(),
            CInstructionInfo::lreturn => "lreturn".to_string(),
            CInstructionInfo::lshl => "lshl".to_string(),
            CInstructionInfo::lshr => "lshr".to_string(),
            CInstructionInfo::lstore(_) => "lstore".to_string(),
            CInstructionInfo::lstore_0 => "lstore_0".to_string(),
            CInstructionInfo::lstore_1 => "lstore_1".to_string(),
            CInstructionInfo::lstore_2 => "lstore_2".to_string(),
            CInstructionInfo::lstore_3 => "lstore_3".to_string(),
            CInstructionInfo::lsub => "lsub".to_string(),
            CInstructionInfo::lushr => "lushr".to_string(),
            CInstructionInfo::lxor => "lxor".to_string(),
            CInstructionInfo::monitorenter => "monitorenter".to_string(),
            CInstructionInfo::monitorexit => "monitorexit".to_string(),
            CInstructionInfo::multianewarray { .. } => "multianewarray".to_string(),
            CInstructionInfo::new(_) => "new".to_string(),
            CInstructionInfo::newarray(_) => "newarray".to_string(),
            CInstructionInfo::nop => "nop".to_string(),
            CInstructionInfo::pop => "pop".to_string(),
            CInstructionInfo::pop2 => "pop2".to_string(),
            CInstructionInfo::putfield { .. } => "putfield".to_string(),
            CInstructionInfo::putstatic { .. } => "putstatic".to_string(),
            CInstructionInfo::ret(_) => "ret".to_string(),
            CInstructionInfo::return_ => "return_".to_string(),
            CInstructionInfo::saload => "saload".to_string(),
            CInstructionInfo::sastore => "sastore".to_string(),
            CInstructionInfo::sipush(_) => "sipush".to_string(),
            CInstructionInfo::swap => "swap".to_string(),
            CInstructionInfo::tableswitch(_) => "tableswitch".to_string(),
            CInstructionInfo::wide(_) => "wide".to_string(),
            CInstructionInfo::EndOfCode => "EndOfCode".to_string(),
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LiveObjectIndex(pub usize);

#[derive(Debug, Clone)]
pub enum CompressedLdcW {
    String { str: Wtf8Buf },
    Class { type_: CPDType },
    Float { float: f32 },
    Integer { integer: i32 },
    MethodType {},
    MethodHandle {},
    LiveObject(LiveObjectIndex),
}

impl PartialEq for CompressedLdcW {
    fn eq(&self, other: &Self) -> bool {
        match self {
            CompressedLdcW::String { str } => {
                if let CompressedLdcW::String { str: other_str } = other {
                    return str == other_str;
                }
                false
            }
            CompressedLdcW::Class { type_ } => {
                if let CompressedLdcW::Class { type_: other_type } = other {
                    return type_ == other_type;
                }
                false
            }
            CompressedLdcW::Float { float } => {
                if let CompressedLdcW::Float { float: other_float } = other {
                    return float == other_float;
                }
                false
            }
            CompressedLdcW::Integer { integer } => {
                if let CompressedLdcW::Integer { integer: other_integer } = other {
                    return integer == other_integer;
                }
                false
            }
            CompressedLdcW::MethodType {} => {
                if let CompressedLdcW::MethodType {} = other {
                    return true;
                }
                false
            }
            CompressedLdcW::MethodHandle {} => {
                if let CompressedLdcW::MethodHandle {} = other {
                    return true;
                }
                false
            }
            CompressedLdcW::LiveObject(LiveObjectIndex(index)) => {
                if let CompressedLdcW::LiveObject(LiveObjectIndex(other_index)) = other {
                    return index == other_index;
                }
                false
            }
        }
    }
}

impl Hash for CompressedLdcW {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CompressedLdcW::String { str } => str.hash(state),
            CompressedLdcW::Class { type_ } => type_.hash(state),
            CompressedLdcW::Float { float } => state.write_u32(float.to_bits()),
            CompressedLdcW::Integer { integer } => state.write_i32(*integer),
            CompressedLdcW::MethodType {} => {
                state.write_usize(1);
            }
            CompressedLdcW::MethodHandle {} => {
                state.write_usize(0);
            }
            CompressedLdcW::LiveObject(LiveObjectIndex(index)) => state.write_usize(*index),
        }
    }
}

impl Eq for CompressedLdcW {}

#[derive(Debug, Clone)]
pub enum CompressedLdc2W {
    Long(i64),
    Double(f64),
}

impl PartialEq for CompressedLdc2W {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl Hash for CompressedLdc2W {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        todo!()
    }
}

impl Eq for CompressedLdc2W {}

pub struct CompressedInvokeInterface {}

#[derive(Clone)]
pub struct CompressedCode {
    pub instructions: HashMap<ByteCodeOffset, CompressedInstruction>,
    pub max_locals: u16,
    pub max_stack: u16,
    pub exception_table: Vec<CompressedExceptionTableElem>,
    pub stack_map_table: Vec<CompressedStackMapFrame>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CompressedStackMapFrame {
    SameFrame(SameFrame),
    SameLocals1StackItemFrame(CompressedSameLocals1StackItemFrame),
    SameLocals1StackItemFrameExtended(CompressedSameLocals1StackItemFrameExtended),
    ChopFrame(CompressedChopFrame),
    SameFrameExtended(CompressedSameFrameExtended),
    AppendFrame(CompressedAppendFrame),
    FullFrame(CompressedFullFrame),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompressedSameLocals1StackItemFrame {
    pub offset_delta: u16,
    pub stack: VType,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompressedSameLocals1StackItemFrameExtended {
    pub offset_delta: u16,
    pub stack: VType,
}

pub type CompressedChopFrame = ChopFrame;
pub type CompressedSameFrameExtended = SameFrameExtended;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompressedAppendFrame {
    pub offset_delta: u16,
    pub locals: Vec<VType>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompressedFullFrame {
    pub offset_delta: u16,
    pub number_of_locals: u16,
    pub locals: Vec<VType>,
    pub number_of_stack_items: u16,
    pub stack: Vec<VType>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompressedExceptionTableElem {
    pub start_pc: ByteCodeOffset,
    pub end_pc: ByteCodeOffset,
    pub handler_pc: ByteCodeOffset,
    pub catch_type: Option<CClassName>,
}
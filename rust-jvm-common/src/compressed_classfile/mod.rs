use std::num::NonZeroU8;

use itertools::{Either, Itertools};

use crate::classfile::{AppendFrame, AttributeType, BootstrapMethods, ChopFrame, Class, Classfile, Code, ConstantInfo, ConstantKind, Double, ExceptionTableElem, FieldInfo, Fieldref, Float, FullFrame, Instruction, InstructionInfo, Integer, InterfaceMethodref, InvokeInterface, Long, MethodInfo, Methodref, MultiNewArray, SameFrameExtended, SameLocals1StackItemFrame, SameLocals1StackItemFrameExtended, StackMapFrame, StackMapTable, String_};
use crate::classnames::{class_name, ClassName};
use crate::compressed_classfile::class_names::CompressedClassName;
use crate::compressed_classfile::code::{CInstructionInfo, CompressedAppendFrame, CompressedChopFrame, CompressedCode, CompressedExceptionTableElem, CompressedFullFrame, CompressedInstruction, CompressedInstructionInfo, CompressedLdc2W, CompressedLdcW, CompressedSameFrameExtended, CompressedSameLocals1StackItemFrame, CompressedSameLocals1StackItemFrameExtended, CompressedStackMapFrame};
use crate::compressed_classfile::compressed_descriptors::{CompressedFieldDescriptor, CompressedMethodDescriptor};
use crate::compressed_classfile::compressed_types::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType, CPRefType};
use crate::compressed_classfile::field_names::FieldName;
use crate::compressed_classfile::method_names::MethodName;
use crate::compressed_classfile::string_pool::{CCString, CompressedClassfileString, CompressedClassfileStringPool};
use crate::descriptor_parser::{MethodDescriptor, parse_field_descriptor, parse_method_descriptor};
use crate::EXPECTED_CLASSFILE_MAGIC;
use crate::loading::LoaderName;
use crate::ptype::{PType, ReferenceType};
use crate::vtype::VType;

pub mod descriptors;
pub mod names;
pub mod native_cpdtype;
pub mod method_names;
pub mod field_names;
pub mod class_names;
pub mod string_pool;
pub mod compressed_types;
pub mod compressed_descriptors;
pub mod code;


#[derive(Clone)]
pub struct CompressedFieldInfo {
    pub access_flags: u16,
    pub name: CCString,
    pub descriptor_type: CompressedParsedDescriptorType,
    // attributes: Vec<AttributeInfo>,
}

#[derive(Clone)]
pub struct CompressedMethodInfo {
    pub access_flags: u16,
    pub name: CompressedClassfileString,
    pub descriptor: CompressedMethodDescriptor,
    pub descriptor_str: CCString,
    pub code: Option<CompressedCode>,
}

#[derive(Clone)]
pub struct CompressedClassfile {
    pub minor_version: u16,
    pub major_version: u16,
    // constant_pool: Vec<ConstantInfo>,
    pub access_flags: u16,
    pub this_class: CompressedClassName,
    pub super_class: Option<CompressedClassName>,
    pub interfaces: Vec<CompressedClassName>,
    pub fields: Vec<CompressedFieldInfo>,
    pub methods: Vec<CompressedMethodInfo>,
    // attributes: Vec<AttributeInfo>,
    pub bootstrap_methods: Option<CompressedBootstrapMethods>,
}

#[derive(Clone)]
pub struct CompressedBootstrapMethods {
    inner: BootstrapMethods,
}

impl CompressedClassfile {
    pub fn new(pool: &CompressedClassfileStringPool, classfile: &Classfile) -> Self {
        let Classfile {
            magic,
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class: _,
            super_class: _,
            interfaces,
            fields,
            methods,
            attributes: _,
        } = classfile;
        assert_eq!(*magic, EXPECTED_CLASSFILE_MAGIC);
        let super_class = classfile.super_class_name().map(|name| CompressedClassName(pool.add_name(name.get_referred_name().to_string(), true)));
        let this = class_name(classfile).get_referred_name().to_string();
        let this_class = CompressedClassName(pool.add_name(this, true));

        let interfaces = interfaces
            .iter()
            .map(|interface| {
                let interface = *interface as usize;
                match &constant_pool[interface].kind {
                    ConstantKind::Class(c) => CompressedClassName(pool.add_name(constant_pool[c.name_index as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo"), true)),
                    _ => panic!(),
                }
            })
            .collect_vec();

        let fields = fields
            .iter()
            .map(|field_info| {
                let FieldInfo { access_flags, name_index, descriptor_index, attributes: _ } = field_info;
                let desc_str = classfile.constant_pool[*descriptor_index as usize].extract_string_from_utf8();
                let parsed = parse_field_descriptor(desc_str.into_string().expect("should have validated this earlier maybe todo").as_str()).unwrap();
                CompressedFieldInfo {
                    access_flags: *access_flags,
                    name: pool.add_name(constant_pool[*name_index as usize].extract_string_from_utf8().as_str().expect("should have validated this earlier maybe todo"), false),
                    descriptor_type: CompressedParsedDescriptorType::from_ptype(&parsed.field_type, pool),
                }
            })
            .collect_vec();
        let methods = methods
            .iter()
            .map(|method_info| {
                let MethodInfo { access_flags, name_index, descriptor_index, attributes } = method_info;
                let descriptor_str = constant_pool[*descriptor_index as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo");
                let MethodDescriptor { parameter_types, return_type } = parse_method_descriptor(descriptor_str.as_str()).unwrap();
                let descriptor_str = pool.add_name(descriptor_str, false);
                let return_type = CompressedParsedDescriptorType::from_ptype(&return_type, pool);
                let arg_types = parameter_types.iter().map(|ptype| CompressedParsedDescriptorType::from_ptype(ptype, pool)).collect_vec();
                let mut code_attr = None;
                for attribute in attributes.iter() {
                    if let AttributeType::Code(Code { attributes, max_stack, max_locals, code_raw: _, code, exception_table }) = &attribute.attribute_type {
                        let instructions = code
                            .iter()
                            .map(|instruction| {
                                let info = CompressedClassfile::compressed_instruction_from_instruction(pool, classfile, constant_pool, instruction);
                                (instruction.offset, CompressedInstruction { offset: instruction.offset, instruction_size: instruction.size, info })
                            })
                            .collect();
                        let exception_table = exception_table
                            .iter()
                            .map(|ExceptionTableElem { start_pc, end_pc, handler_pc, catch_type }| {
                                let catch_type = if *catch_type == 0 { None } else { Some(CPDType::from_ptype(&PType::Ref(classfile.extract_class_from_constant_pool_name(*catch_type)), pool).unwrap_class_type()) };
                                CompressedExceptionTableElem { start_pc: *start_pc, end_pc: *end_pc, handler_pc: *handler_pc, catch_type }
                            })
                            .collect_vec();
                        let stack_map_table = attributes
                            .iter()
                            .find_map(|attr| match &attr.attribute_type {
                                AttributeType::StackMapTable(StackMapTable { entries }) => CompressedClassfile::convert_stack_map_table_entries(pool, entries),
                                _ => None,
                            })
                            .unwrap_or_default();
                        code_attr = Some(CompressedCode {
                            instructions,
                            max_locals: *max_locals,
                            max_stack: *max_stack,
                            exception_table,
                            stack_map_table,
                        });
                    }
                }
                CompressedMethodInfo {
                    access_flags: *access_flags,
                    name: pool.add_name(constant_pool[*name_index as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo"), false),
                    descriptor: CompressedMethodDescriptor { arg_types, return_type },
                    descriptor_str,
                    code: code_attr,
                }
            })
            .collect_vec();
        let bootstrap_methods = classfile.attributes.iter().find_map(|x| match &x.attribute_type {
            AttributeType::BootstrapMethods(bm) => Some(bm.clone()),
            _ => None,
        });
        Self {
            minor_version: *minor_version,
            major_version: *major_version,
            access_flags: *access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            bootstrap_methods: (|| Some(CompressedBootstrapMethods { inner: bootstrap_methods? }))(),
        }
    }

    fn convert_stack_map_table_entries(pool: &CompressedClassfileStringPool, entries: &[StackMapFrame]) -> Option<Vec<CompressedStackMapFrame>> {
        Some(
            entries
                .iter()
                .map(|stackmapframe| {
                    match stackmapframe {
                        StackMapFrame::SameFrame(sf) => CompressedStackMapFrame::SameFrame(sf.clone()),
                        StackMapFrame::SameLocals1StackItemFrame(SameLocals1StackItemFrame { offset_delta, stack }) => {
                            let stack = VType::from_ptype(stack, LoaderName::BootstrapLoader, pool);
                            //todo deal with this usage of bootstrap loader
                            CompressedStackMapFrame::SameLocals1StackItemFrame(CompressedSameLocals1StackItemFrame { offset_delta: *offset_delta, stack })
                        }
                        StackMapFrame::SameLocals1StackItemFrameExtended(SameLocals1StackItemFrameExtended { offset_delta, stack }) => {
                            let stack = CPDType::from_ptype(stack, pool).to_verification_type(LoaderName::BootstrapLoader);
                            CompressedStackMapFrame::SameLocals1StackItemFrameExtended(CompressedSameLocals1StackItemFrameExtended { offset_delta: *offset_delta, stack })
                        }
                        StackMapFrame::ChopFrame(ChopFrame { offset_delta, k_frames_to_chop }) => CompressedStackMapFrame::ChopFrame(CompressedChopFrame { offset_delta: *offset_delta, k_frames_to_chop: *k_frames_to_chop }),
                        StackMapFrame::SameFrameExtended(SameFrameExtended { offset_delta }) => CompressedStackMapFrame::SameFrameExtended(CompressedSameFrameExtended { offset_delta: *offset_delta }),
                        StackMapFrame::AppendFrame(AppendFrame { offset_delta, locals }) => {
                            let locals = locals.iter().map(|local| VType::from_ptype(local, LoaderName::BootstrapLoader, pool)).collect_vec(); //todo deal with this usage of bootstrap loader
                            CompressedStackMapFrame::AppendFrame(CompressedAppendFrame { offset_delta: *offset_delta, locals })
                        }
                        StackMapFrame::FullFrame(FullFrame { offset_delta, number_of_locals, locals, number_of_stack_items, stack }) => {
                            let locals = locals.iter().map(|local| VType::from_ptype(local, LoaderName::BootstrapLoader, pool)).collect_vec(); //todo deal with this usage of bootstrap loader
                            let stack = stack.iter().map(|local| VType::from_ptype(local, LoaderName::BootstrapLoader, pool)).collect_vec(); //todo deal with this usage of bootstrap loader
                            CompressedStackMapFrame::FullFrame(CompressedFullFrame {
                                offset_delta: *offset_delta,
                                number_of_locals: *number_of_locals,
                                locals,
                                number_of_stack_items: *number_of_stack_items,
                                stack,
                            })
                        }
                    }
                })
                .collect_vec(),
        )
    }

    fn compressed_instruction_from_instruction(pool: &CompressedClassfileStringPool, classfile: &Classfile, constant_pool: &[ConstantInfo], instruction: &Instruction) -> CompressedInstructionInfo {
        match &instruction.instruction {
            InstructionInfo::aaload => CInstructionInfo::aaload,
            InstructionInfo::aastore => CInstructionInfo::aastore,
            InstructionInfo::aconst_null => CInstructionInfo::aconst_null,
            InstructionInfo::aload(idx) => CInstructionInfo::aload(*idx),
            InstructionInfo::aload_0 => CInstructionInfo::aload_0,
            InstructionInfo::aload_1 => CInstructionInfo::aload_1,
            InstructionInfo::aload_2 => CInstructionInfo::aload_2,
            InstructionInfo::aload_3 => CInstructionInfo::aload_3,
            InstructionInfo::anewarray(cp) => {
                let type_ = CPDType::from_ptype(&PType::Ref(classfile.extract_class_from_constant_pool_name(*cp)), pool);
                CInstructionInfo::anewarray(type_)
            }
            InstructionInfo::areturn => CInstructionInfo::areturn,
            InstructionInfo::arraylength => CInstructionInfo::arraylength,
            InstructionInfo::astore(idx) => CInstructionInfo::astore(*idx),
            InstructionInfo::astore_0 => CInstructionInfo::astore_0,
            InstructionInfo::astore_1 => CInstructionInfo::astore_1,
            InstructionInfo::astore_2 => CInstructionInfo::astore_2,
            InstructionInfo::astore_3 => CInstructionInfo::astore_3,
            InstructionInfo::athrow => CInstructionInfo::athrow,
            InstructionInfo::baload => CInstructionInfo::baload,
            InstructionInfo::bastore => CInstructionInfo::bastore,
            InstructionInfo::bipush(idx) => CInstructionInfo::bipush(*idx),
            InstructionInfo::caload => CInstructionInfo::caload,
            InstructionInfo::castore => CInstructionInfo::castore,
            InstructionInfo::checkcast(cp) => {
                let type_ = CPDType::from_ptype(&PType::Ref(classfile.extract_class_from_constant_pool_name(*cp)), pool);
                CInstructionInfo::checkcast(type_)
            }
            InstructionInfo::d2f => CInstructionInfo::d2f,
            InstructionInfo::d2i => CInstructionInfo::d2i,
            InstructionInfo::d2l => CInstructionInfo::d2l,
            InstructionInfo::dadd => CInstructionInfo::dadd,
            InstructionInfo::daload => CInstructionInfo::daload,
            InstructionInfo::dastore => CInstructionInfo::dastore,
            InstructionInfo::dcmpg => CInstructionInfo::dcmpg,
            InstructionInfo::dcmpl => CInstructionInfo::dcmpl,
            InstructionInfo::dconst_0 => CInstructionInfo::dconst_0,
            InstructionInfo::dconst_1 => CInstructionInfo::dconst_1,
            InstructionInfo::ddiv => CInstructionInfo::ddiv,
            InstructionInfo::dload(idx) => CInstructionInfo::dload(*idx),
            InstructionInfo::dload_0 => CInstructionInfo::dload_0,
            InstructionInfo::dload_1 => CInstructionInfo::dload_1,
            InstructionInfo::dload_2 => CInstructionInfo::dload_2,
            InstructionInfo::dload_3 => CInstructionInfo::dload_3,
            InstructionInfo::dmul => CInstructionInfo::dmul,
            InstructionInfo::dneg => CInstructionInfo::dneg,
            InstructionInfo::drem => CInstructionInfo::drem,
            InstructionInfo::dreturn => CInstructionInfo::dreturn,
            InstructionInfo::dstore(idx) => CInstructionInfo::dstore(*idx),
            InstructionInfo::dstore_0 => CInstructionInfo::dstore_0,
            InstructionInfo::dstore_1 => CInstructionInfo::dstore_1,
            InstructionInfo::dstore_2 => CInstructionInfo::dstore_2,
            InstructionInfo::dstore_3 => CInstructionInfo::dstore_3,
            InstructionInfo::dsub => CInstructionInfo::dsub,
            InstructionInfo::dup => CInstructionInfo::dup,
            InstructionInfo::dup_x1 => CInstructionInfo::dup_x1,
            InstructionInfo::dup_x2 => CInstructionInfo::dup_x2,
            InstructionInfo::dup2 => CInstructionInfo::dup2,
            InstructionInfo::dup2_x1 => CInstructionInfo::dup2_x1,
            InstructionInfo::dup2_x2 => CInstructionInfo::dup2_x2,
            InstructionInfo::f2d => CInstructionInfo::f2d,
            InstructionInfo::f2i => CInstructionInfo::f2i,
            InstructionInfo::f2l => CInstructionInfo::f2l,
            InstructionInfo::fadd => CInstructionInfo::fadd,
            InstructionInfo::faload => CInstructionInfo::faload,
            InstructionInfo::fastore => CInstructionInfo::fastore,
            InstructionInfo::fcmpg => CInstructionInfo::fcmpg,
            InstructionInfo::fcmpl => CInstructionInfo::fcmpl,
            InstructionInfo::fconst_0 => CInstructionInfo::fconst_0,
            InstructionInfo::fconst_1 => CInstructionInfo::fconst_1,
            InstructionInfo::fconst_2 => CInstructionInfo::fconst_2,
            InstructionInfo::fdiv => CInstructionInfo::fdiv,
            InstructionInfo::fload(idx) => CInstructionInfo::fload(*idx),
            InstructionInfo::fload_0 => CInstructionInfo::fload_0,
            InstructionInfo::fload_1 => CInstructionInfo::fload_1,
            InstructionInfo::fload_2 => CInstructionInfo::fload_2,
            InstructionInfo::fload_3 => CInstructionInfo::fload_3,
            InstructionInfo::fmul => CInstructionInfo::fmul,
            InstructionInfo::fneg => CInstructionInfo::fneg,
            InstructionInfo::frem => CInstructionInfo::frem,
            InstructionInfo::freturn => CInstructionInfo::freturn,
            InstructionInfo::fstore(idx) => CInstructionInfo::fstore(*idx),
            InstructionInfo::fstore_0 => CInstructionInfo::fstore_0,
            InstructionInfo::fstore_1 => CInstructionInfo::fstore_1,
            InstructionInfo::fstore_2 => CInstructionInfo::fstore_2,
            InstructionInfo::fstore_3 => CInstructionInfo::fstore_3,
            InstructionInfo::fsub => CInstructionInfo::fsub,
            InstructionInfo::getfield(cp) => {
                let (target_class, field_name, desc) = CompressedClassfile::field_descriptor_extraction(pool, classfile, constant_pool, *cp);
                CInstructionInfo::getfield { name: FieldName(pool.add_name(field_name, false)), desc, target_class }
            }
            InstructionInfo::getstatic(cp) => {
                let (target_class, field_name, desc) = CompressedClassfile::field_descriptor_extraction(pool, classfile, constant_pool, *cp);
                CInstructionInfo::getstatic { name: FieldName(pool.add_name(field_name, false)), desc, target_class }
            }
            InstructionInfo::goto_(offset) => CInstructionInfo::goto_(*offset),
            InstructionInfo::goto_w(offset) => CInstructionInfo::goto_w(*offset),
            InstructionInfo::i2b => CInstructionInfo::i2b,
            InstructionInfo::i2c => CInstructionInfo::i2c,
            InstructionInfo::i2d => CInstructionInfo::i2d,
            InstructionInfo::i2f => CInstructionInfo::i2f,
            InstructionInfo::i2l => CInstructionInfo::i2l,
            InstructionInfo::i2s => CInstructionInfo::i2s,
            InstructionInfo::iadd => CInstructionInfo::iadd,
            InstructionInfo::iaload => CInstructionInfo::iaload,
            InstructionInfo::iand => CInstructionInfo::iand,
            InstructionInfo::iastore => CInstructionInfo::iastore,
            InstructionInfo::iconst_m1 => CInstructionInfo::iconst_m1,
            InstructionInfo::iconst_0 => CInstructionInfo::iconst_0,
            InstructionInfo::iconst_1 => CInstructionInfo::iconst_1,
            InstructionInfo::iconst_2 => CInstructionInfo::iconst_2,
            InstructionInfo::iconst_3 => CInstructionInfo::iconst_3,
            InstructionInfo::iconst_4 => CInstructionInfo::iconst_4,
            InstructionInfo::iconst_5 => CInstructionInfo::iconst_5,
            InstructionInfo::idiv => CInstructionInfo::idiv,
            InstructionInfo::if_acmpeq(idx) => CInstructionInfo::if_acmpeq(*idx),
            InstructionInfo::if_acmpne(idx) => CInstructionInfo::if_acmpne(*idx),
            InstructionInfo::if_icmpeq(idx) => CInstructionInfo::if_icmpeq(*idx),
            InstructionInfo::if_icmpne(idx) => CInstructionInfo::if_icmpne(*idx),
            InstructionInfo::if_icmplt(idx) => CInstructionInfo::if_icmplt(*idx),
            InstructionInfo::if_icmpge(idx) => CInstructionInfo::if_icmpge(*idx),
            InstructionInfo::if_icmpgt(idx) => CInstructionInfo::if_icmpgt(*idx),
            InstructionInfo::if_icmple(idx) => CInstructionInfo::if_icmple(*idx),
            InstructionInfo::ifeq(idx) => CInstructionInfo::ifeq(*idx),
            InstructionInfo::ifne(idx) => CInstructionInfo::ifne(*idx),
            InstructionInfo::iflt(idx) => CInstructionInfo::iflt(*idx),
            InstructionInfo::ifge(idx) => CInstructionInfo::ifge(*idx),
            InstructionInfo::ifgt(idx) => CInstructionInfo::ifgt(*idx),
            InstructionInfo::ifle(idx) => CInstructionInfo::ifle(*idx),
            InstructionInfo::ifnonnull(idx) => CInstructionInfo::ifnonnull(*idx),
            InstructionInfo::ifnull(idx) => CInstructionInfo::ifnull(*idx),
            InstructionInfo::iinc(iinc) => CInstructionInfo::iinc(*iinc),
            InstructionInfo::iload(idx) => CInstructionInfo::iload(*idx),
            InstructionInfo::iload_0 => CInstructionInfo::iload_0,
            InstructionInfo::iload_1 => CInstructionInfo::iload_1,
            InstructionInfo::iload_2 => CInstructionInfo::iload_2,
            InstructionInfo::iload_3 => CInstructionInfo::iload_3,
            InstructionInfo::imul => CInstructionInfo::imul,
            InstructionInfo::ineg => CInstructionInfo::ineg,
            InstructionInfo::instanceof(cp) => {
                let type_ = CPDType::from_ptype(&PType::Ref(classfile.extract_class_from_constant_pool_name(*cp)), pool);
                CInstructionInfo::instanceof(type_)
            }
            InstructionInfo::invokedynamic(cp) => CInstructionInfo::invokedynamic(*cp),
            InstructionInfo::invokeinterface(InvokeInterface { index, count }) => {
                let (classname_ref_type, descriptor, method_name) = CompressedClassfile::method_descriptor_extraction(pool, classfile, constant_pool, index);
                CInstructionInfo::invokeinterface { method_name, descriptor, classname_ref_type, count: NonZeroU8::new(*count).expect("") }
            }
            InstructionInfo::invokespecial(cp) => {
                let (classname_ref_type, descriptor, method_name) = CompressedClassfile::method_descriptor_extraction(pool, classfile, constant_pool, cp);
                CInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type }
            }
            InstructionInfo::invokestatic(cp) => {
                let (classname_ref_type, descriptor, method_name) = CompressedClassfile::method_descriptor_extraction(pool, classfile, constant_pool, cp);
                CInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type }
            }
            InstructionInfo::invokevirtual(cp) => {
                let (classname_ref_type, descriptor, method_name) = CompressedClassfile::method_descriptor_extraction(pool, classfile, constant_pool, cp);
                CInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type }
            }
            InstructionInfo::ior => CInstructionInfo::ior,
            InstructionInfo::irem => CInstructionInfo::irem,
            InstructionInfo::ireturn => CInstructionInfo::ireturn,
            InstructionInfo::ishl => CInstructionInfo::ishl,
            InstructionInfo::ishr => CInstructionInfo::ishr,
            InstructionInfo::istore(idx) => CInstructionInfo::istore(*idx),
            InstructionInfo::istore_0 => CInstructionInfo::istore_0,
            InstructionInfo::istore_1 => CInstructionInfo::istore_1,
            InstructionInfo::istore_2 => CInstructionInfo::istore_2,
            InstructionInfo::istore_3 => CInstructionInfo::istore_3,
            InstructionInfo::isub => CInstructionInfo::isub,
            InstructionInfo::iushr => CInstructionInfo::iushr,
            InstructionInfo::ixor => CInstructionInfo::ixor,
            InstructionInfo::jsr(offset) => CInstructionInfo::jsr(*offset),
            InstructionInfo::jsr_w(offset) => CInstructionInfo::jsr_w(*offset),
            InstructionInfo::l2d => CInstructionInfo::l2d,
            InstructionInfo::l2f => CInstructionInfo::l2f,
            InstructionInfo::l2i => CInstructionInfo::l2i,
            InstructionInfo::ladd => CInstructionInfo::ladd,
            InstructionInfo::laload => CInstructionInfo::laload,
            InstructionInfo::land => CInstructionInfo::land,
            InstructionInfo::lastore => CInstructionInfo::lastore,
            InstructionInfo::lcmp => CInstructionInfo::lcmp,
            InstructionInfo::lconst_0 => CInstructionInfo::lconst_0,
            InstructionInfo::lconst_1 => CInstructionInfo::lconst_1,
            InstructionInfo::ldc(cp) => CInstructionInfo::ldc(CompressedClassfile::constant_value(pool, constant_pool, *cp as u16)),
            InstructionInfo::ldc_w(cp) => CInstructionInfo::ldc_w(CompressedClassfile::constant_value(pool, constant_pool, *cp).unwrap_left()),
            InstructionInfo::ldc2_w(cp) => CInstructionInfo::ldc2_w(CompressedClassfile::constant_value(pool, constant_pool, *cp).unwrap_right()),
            InstructionInfo::ldiv => CInstructionInfo::ldiv,
            InstructionInfo::lload(idx) => CInstructionInfo::lload(*idx),
            InstructionInfo::lload_0 => CInstructionInfo::lload_0,
            InstructionInfo::lload_1 => CInstructionInfo::lload_1,
            InstructionInfo::lload_2 => CInstructionInfo::lload_2,
            InstructionInfo::lload_3 => CInstructionInfo::lload_3,
            InstructionInfo::lmul => CInstructionInfo::lmul,
            InstructionInfo::lneg => CInstructionInfo::lneg,
            InstructionInfo::lookupswitch(ls) => CInstructionInfo::lookupswitch(ls.clone()),
            InstructionInfo::lor => CInstructionInfo::lor,
            InstructionInfo::lrem => CInstructionInfo::lrem,
            InstructionInfo::lreturn => CInstructionInfo::lreturn,
            InstructionInfo::lshl => CInstructionInfo::lshl,
            InstructionInfo::lshr => CInstructionInfo::lshr,
            InstructionInfo::lstore(idx) => CInstructionInfo::lstore(*idx),
            InstructionInfo::lstore_0 => CInstructionInfo::lstore_0,
            InstructionInfo::lstore_1 => CInstructionInfo::lstore_1,
            InstructionInfo::lstore_2 => CInstructionInfo::lstore_2,
            InstructionInfo::lstore_3 => CInstructionInfo::lstore_3,
            InstructionInfo::lsub => CInstructionInfo::lsub,
            InstructionInfo::lushr => CInstructionInfo::lushr,
            InstructionInfo::lxor => CInstructionInfo::lxor,
            InstructionInfo::monitorenter => CInstructionInfo::monitorenter,
            InstructionInfo::monitorexit => CInstructionInfo::monitorexit,
            InstructionInfo::multianewarray(MultiNewArray { index, dims }) => {
                let type_ = CPDType::from_ptype(&PType::Ref(classfile.extract_class_from_constant_pool_name(*index)), pool);
                CInstructionInfo::multianewarray { type_, dimensions: NonZeroU8::new(*dims).unwrap() }
            }
            InstructionInfo::new(cp) => {
                let classname = CPDType::from_ptype(&PType::Ref(classfile.extract_class_from_constant_pool_name(*cp)), pool).unwrap_class_type();
                CInstructionInfo::new(classname)
            }
            InstructionInfo::newarray(cpdtype) => CInstructionInfo::newarray(*cpdtype),
            InstructionInfo::nop => CInstructionInfo::nop,
            InstructionInfo::pop => CInstructionInfo::pop,
            InstructionInfo::pop2 => CInstructionInfo::pop2,
            InstructionInfo::putfield(cp) => {
                let (target_class, field_name, desc) = CompressedClassfile::field_descriptor_extraction(pool, classfile, constant_pool, *cp);
                CInstructionInfo::putfield { name: FieldName(pool.add_name(field_name, false)), desc, target_class }
            }
            InstructionInfo::putstatic(cp) => {
                let (target_class, field_name, desc) = CompressedClassfile::field_descriptor_extraction(pool, classfile, constant_pool, *cp);
                CInstructionInfo::putstatic { name: FieldName(pool.add_name(field_name, false)), desc, target_class }
            }
            InstructionInfo::ret(idx) => CInstructionInfo::ret(*idx),
            InstructionInfo::return_ => CInstructionInfo::return_,
            InstructionInfo::saload => CInstructionInfo::saload,
            InstructionInfo::sastore => CInstructionInfo::sastore,
            InstructionInfo::sipush(idx) => CInstructionInfo::sipush(*idx),
            InstructionInfo::swap => CInstructionInfo::swap,
            InstructionInfo::tableswitch(ts) => CInstructionInfo::tableswitch(box ts.clone()),
            InstructionInfo::wide(wide) => CInstructionInfo::wide(*wide),
            InstructionInfo::EndOfCode => CInstructionInfo::EndOfCode,
        }
    }

    fn constant_value(pool: &CompressedClassfileStringPool, constant_pool: &[ConstantInfo], cp: u16) -> Either<CompressedLdcW, CompressedLdc2W> {
        match constant_pool[cp as usize].kind {
            ConstantKind::Utf8(_) => todo!(),
            ConstantKind::Integer(Integer { bytes }) => Either::Left(CompressedLdcW::Integer { integer: bytes as i32 }),
            ConstantKind::Float(Float { bytes }) => Either::Left(CompressedLdcW::Float { float: f32::from_ne_bytes(bytes.to_ne_bytes()) }),
            ConstantKind::Long(Long { low_bytes, high_bytes }) => Either::Right(CompressedLdc2W::Long(((high_bytes as u64) << 32 | low_bytes as u64) as i64)),
            ConstantKind::Double(Double { low_bytes, high_bytes }) => Either::Right(CompressedLdc2W::Double(f64::from_ne_bytes((((high_bytes as u64) << 32) | low_bytes as u64).to_ne_bytes()))),
            ConstantKind::Class(Class { name_index }) => {
                let name = constant_pool[name_index as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo");
                let type_ = if name.starts_with('[') {
                    CPDType::from_ptype(&parse_field_descriptor(name.as_str()).unwrap().field_type, pool)
                } else {
                    CPDType::from_ptype(&PType::Ref(ReferenceType::Class(ClassName::Str(name))), pool)
                };

                Either::Left(CompressedLdcW::Class { type_ })
            }
            ConstantKind::String(String_ { string_index }) => {
                let string = constant_pool[string_index as usize].extract_string_from_utf8();
                Either::Left(CompressedLdcW::String { str: string })
            }
            ConstantKind::MethodHandle(_) => todo!(),
            ConstantKind::MethodType(_) => todo!(),
            ConstantKind::LiveObject(index) => Either::Left(CompressedLdcW::LiveObject(index)),
            _ => {
                dbg!(&constant_pool[cp as usize].kind);
                panic!()
            }
        }
    }

    fn field_descriptor_extraction(pool: &CompressedClassfileStringPool, classfile: &Classfile, constant_pool: &[ConstantInfo], cp: u16) -> (CompressedClassName, String, CompressedFieldDescriptor) {
        match &constant_pool[cp as usize].kind {
            ConstantKind::Fieldref(Fieldref { class_index, name_and_type_index }) => {
                let target_class = CPDType::from_ptype(&PType::Ref(classfile.extract_class_from_constant_pool_name(*class_index)), pool).unwrap_class_type();
                let (field_name, desc) = classfile.name_and_type_extractor(*name_and_type_index);
                let desc = CompressedFieldDescriptor(CPDType::from_ptype(&parse_field_descriptor(desc.as_str()).unwrap().field_type, pool));
                (target_class, field_name, desc)
            }
            _ => panic!(),
        }
    }

    fn method_descriptor_extraction(pool: &CompressedClassfileStringPool, classfile: &Classfile, constant_pool: &[ConstantInfo], index: &u16) -> (CPRefType, CompressedMethodDescriptor, MethodName) {
        let (class_index, nt_index) = match constant_pool[*index as usize].kind {
            ConstantKind::Methodref(Methodref { class_index, name_and_type_index }) => (class_index, name_and_type_index),
            ConstantKind::InterfaceMethodref(InterfaceMethodref { class_index, nt_index }) => (class_index, nt_index),
            _ => panic!(),
        };
        let p_type = PType::Ref(classfile.extract_class_from_constant_pool_name(class_index));
        let (method_name, desc) = classfile.name_and_type_extractor(nt_index);
        let ref_type = CPDType::from_ptype(&p_type, pool).unwrap_ref_type();
        let descriptor = CMethodDescriptor::from_legacy(parse_method_descriptor(desc.as_str()).unwrap(), pool);
        let method_name = MethodName(pool.add_name(method_name, true));
        (ref_type, descriptor, method_name)
    }
}
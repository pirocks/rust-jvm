use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::classfile::{Annotation, AnnotationDefault, AnnotationValue, AppendFrame, ArrayValue, AttributeInfo, AttributeType, BootstrapMethod, BootstrapMethods, ChopFrame, ClassInfoIndex, Code, ConstantKind, ConstantValue, Deprecated, ElementValue, ElementValuePair, EnumConstValue, Exceptions, ExceptionTableElem, FullFrame, InnerClass, InnerClasses, LineNumber, LineNumberTable, LineNumberTableEntry, LocalVariableTable, LocalVariableTableEntry, LocalVariableTypeTable, LocalVariableTypeTableEntry, LocalVarTargetTableEntry, MethodParameter, MethodParameters, NestHost, NestMembers, RuntimeInvisibleAnnotations, RuntimeVisibleAnnotations, RuntimeVisibleParameterAnnotations, RuntimeVisibleTypeAnnotations, SameFrame, SameFrameExtended, SameLocals1StackItemFrame, SameLocals1StackItemFrameExtended, Signature, SourceDebugExtension, SourceFile, StackMapFrame, StackMapTable, Synthetic, TargetInfo, TypeAnnotation, TypePath, TypePathEntry, UninitializedVariableInfo};
use rust_jvm_common::classfile::AttributeType::Unknown;
use rust_jvm_common::classfile::EnclosingMethod;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::descriptor_parser::parse_field_descriptor;
use rust_jvm_common::ptype::{PType, ReferenceType};

use crate::ClassfileParsingError;
use crate::code::parse_code_raw;
use crate::constant_infos::is_utf8;
use crate::parsing_util::ParsingContext;

pub fn parse_attribute(p: &mut dyn ParsingContext) -> Result<AttributeInfo, ClassfileParsingError> {
    let attribute_name_index = p.read16()?;
    let attribute_length = p.read32()?;
    let name_pool = &p.constant_pool_borrow()[attribute_name_index as usize];
    assert!(is_utf8(&name_pool.kind).is_some());
    let name_struct = is_utf8(&name_pool.kind).ok_or(ClassfileParsingError::NoAttributeName)?;
    let name = &name_struct.string.clone().into_string()?;
    let attribute_type = match name.as_str() {
        "ConstantValue" => parse_constant_value_index(p),
        "Code" => parse_code(p),
        "StackMapTable" => parse_stack_map_table(p),
        "Exceptions" => parse_exceptions(p),
        "InnerClasses" => parse_inner_classes(p),
        "EnclosingMethod" => parse_enclosing_method(p),
        "Synthetic" => parse_synthetic(p),
        "Signature" => parse_signature(p),
        "SourceFile" => parse_sourcefile(p),
        "SourceDebugExtension" => parse_source_debug_extension(p, attribute_length),
        "LineNumberTable" => parse_line_number_table(p),
        "LocalVariableTable" => parse_local_variable_table(p),
        "LocalVariableTypeTable" => parse_local_variable_type_table(p),
        "Deprecated" => parse_deprecated(p),
        "RuntimeVisibleAnnotations" => parse_runtime_visible_annotations(p),
        "RuntimeInVisibleAnnotations" => parse_runtime_invisible_annotations(p),
        "RuntimeVisibleParameterAnnotations" => parse_runtime_visible_parameter_annotations(p),
        "RuntimeInVisibleParameterAnnotations" => parse_runtime_invisible_parameter_annotations(p),
        "RuntimeVisibleTypeAnnotations" => parse_runtime_visible_type_annotations(p),
        "RuntimeInVisibleTypeAnnotations" => parse_runtime_invisible_type_annotations(p),
        "AnnotationDefault" => parse_annotation_default(p),
        "BootstrapMethods" => parse_bootstrap_methods(p),
        "MethodParameters" => parse_method_parameters(p),
        //java 9+ but gets parsed anyway:
        "NestMembers" => parse_nest_members(p),
        "NestHost" => parse_nest_host(p),
        _ => {
            for _ in 0..attribute_length {
                p.read8()?;
            }
            Ok(Unknown)
        }
    }?;
    Ok(AttributeInfo { attribute_name_index, attribute_length, attribute_type })
}

fn parse_local_variable_type_table_entry(p: &mut dyn ParsingContext) -> Result<LocalVariableTypeTableEntry, ClassfileParsingError> {
    let start_pc = p.read16()?;
    let length = p.read16()?;
    let name_index = p.read16()?;
    let descriptor_index = p.read16()?;
    let index = p.read16()?;
    Ok(LocalVariableTypeTableEntry { start_pc, length, name_index, descriptor_index, index })
}

fn parse_local_variable_type_table(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let local_variable_type_table_length = p.read16()?;
    let mut type_table = vec![];
    for _ in 0..local_variable_type_table_length {
        type_table.push(parse_local_variable_type_table_entry(p)?);
    }
    Ok(AttributeType::LocalVariableTypeTable(LocalVariableTypeTable { type_table }))
}

fn parse_enclosing_method(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let class_index = p.read16()?;
    let method_index = p.read16()?;
    Ok(AttributeType::EnclosingMethod(EnclosingMethod { class_index, method_index }))
}

fn parse_synthetic(_p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    Ok(AttributeType::Synthetic(Synthetic {}))
}

fn parse_nest_host(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let host_class_index = p.read16()?;
    Ok(AttributeType::NestHost(NestHost { host_class_index }))
}

fn parse_nest_members(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let number_of_classes = p.read16()?;
    let mut classes = Vec::with_capacity(number_of_classes as usize);
    for _ in 0..number_of_classes {
        classes.push(p.read16()?);
    }
    Ok(AttributeType::NestMembers(NestMembers { classes }))
}

fn parse_constant_value_index(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let constant_value_index = p.read16()?;
    Ok(AttributeType::ConstantValue(ConstantValue { constant_value_index }))
}

fn parse_bootstrap_methods(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let num_bootstrap_methods = p.read16()?;
    let mut bootstrap_methods = Vec::with_capacity(num_bootstrap_methods as usize);
    for _ in 0..num_bootstrap_methods {
        bootstrap_methods.push(parse_bootstrap_method(p)?);
    }
    Ok(AttributeType::BootstrapMethods(BootstrapMethods { bootstrap_methods }))
}

fn parse_bootstrap_method(p: &mut dyn ParsingContext) -> Result<BootstrapMethod, ClassfileParsingError> {
    let bootstrap_method_ref = p.read16()?;
    let num_bootstrap_args = p.read16()?;
    let mut bootstrap_arguments = Vec::with_capacity(num_bootstrap_args as usize);
    for _ in 0..num_bootstrap_args {
        bootstrap_arguments.push(p.read16()?);
    }
    Ok(BootstrapMethod { bootstrap_arguments, bootstrap_method_ref })
}

fn parse_inner_class(p: &mut dyn ParsingContext) -> Result<InnerClass, ClassfileParsingError> {
    let inner_class_info_index = p.read16()?;
    let outer_class_info_index = p.read16()?;
    let inner_name_index = p.read16()?;
    let inner_class_access_flags = p.read16()?;
    Ok(InnerClass { inner_class_access_flags, inner_class_info_index, inner_name_index, outer_class_info_index })
}

fn parse_inner_classes(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let number_of_classes = p.read16()?;
    let mut classes = Vec::with_capacity(number_of_classes as usize);
    for _ in 0..number_of_classes {
        classes.push(parse_inner_class(p)?)
    }
    Ok(AttributeType::InnerClasses(InnerClasses { classes }))
}

fn parse_deprecated(_: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    Ok(AttributeType::Deprecated(Deprecated {}))
}

fn parse_exceptions(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let num_exceptions = p.read16()?;
    let mut exception_index_table = Vec::new();
    for _ in 0..num_exceptions {
        exception_index_table.push(p.read16()?);
    }
    Ok(AttributeType::Exceptions(Exceptions { exception_index_table }))
}

fn parse_signature(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    Ok(AttributeType::Signature(Signature { signature_index: p.read16()? }))
}

fn parse_element_value(p: &mut dyn ParsingContext) -> Result<ElementValue, ClassfileParsingError> {
    let tag = p.read8()? as char;
    Ok(match tag {
        'B' => ElementValue::Byte(p.read16()?),
        'C' => ElementValue::Char(p.read16()?),
        'D' => ElementValue::Double(p.read16()?),
        'F' => ElementValue::Float(p.read16()?),
        'I' => ElementValue::Int(p.read16()?),
        'J' => ElementValue::Long(p.read16()?),
        'S' => ElementValue::Short(p.read16()?),
        'Z' => ElementValue::Boolean(p.read16()?),
        's' => ElementValue::String(p.read16()?),
        'e' => ElementValue::EnumType(parse_enum_const_value(p)?),
        'c' => ElementValue::Class(ClassInfoIndex { class_info_index: p.read16()? }),
        '@' => ElementValue::AnnotationType(AnnotationValue { annotation: parse_annotation(p)? }),
        '[' => {
            let num_values = p.read16()?;
            let mut values = vec![];
            for _ in 0..num_values {
                values.push(parse_element_value(p)?);
            }
            ElementValue::ArrayType(ArrayValue { values })
        }
        _ => return Err(ClassfileParsingError::WrongTag),
    })
}

pub fn element_value_to_bytes(element_value: ElementValue) -> Vec<u8> {
    let mut res = vec![];
    match element_value {
        ElementValue::Byte(cp_index) => {
            res.push(b'B');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::Char(cp_index) => {
            res.push(b'C');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::Double(cp_index) => {
            res.push(b'D');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::Float(cp_index) => {
            res.push(b'F');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::Int(cp_index) => {
            res.push(b'I');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::Long(cp_index) => {
            res.push(b'J');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::Short(cp_index) => {
            res.push(b'S');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::Boolean(cp_index) => {
            res.push(b'Z');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::String(cp_index) => {
            res.push(b's');
            res.extend_from_slice(&cp_index.to_be_bytes());
        }
        ElementValue::EnumType(EnumConstValue { type_name_index, const_name_index }) => {
            res.push(b'e');
            res.extend_from_slice(&type_name_index.to_be_bytes());
            res.extend_from_slice(&const_name_index.to_be_bytes());
        }
        ElementValue::Class(ClassInfoIndex { class_info_index }) => {
            res.push(b'c');
            res.extend_from_slice(&class_info_index.to_be_bytes());
        }
        ElementValue::AnnotationType(AnnotationValue { annotation }) => {
            res.push(b'@');
            res.extend_from_slice(annotation_to_bytes(annotation).as_slice());
        }
        ElementValue::ArrayType(ArrayValue { values }) => {
            res.push(b'[');
            let num_bytes = values.len() as u16;
            res.extend_from_slice(&num_bytes.to_be_bytes());
            for value in values {
                res.extend_from_slice(element_value_to_bytes(value).as_slice());
            }
        }
    }
    res
}

fn parse_enum_const_value(p: &mut dyn ParsingContext) -> Result<EnumConstValue, ClassfileParsingError> {
    let type_name_index = p.read16()?;
    let const_name_index = p.read16()?;
    Ok(EnumConstValue { type_name_index, const_name_index })
}

fn parse_element_value_pair(p: &mut dyn ParsingContext) -> Result<ElementValuePair, ClassfileParsingError> {
    let element_name_index = p.read16()?;
    let value = parse_element_value(p)?;
    Ok(ElementValuePair { element_name_index, value })
}

fn parse_annotation(p: &mut dyn ParsingContext) -> Result<Annotation, ClassfileParsingError> {
    let type_index = p.read16()?;
    let num_element_value_pairs = p.read16()?;
    let mut element_value_pairs: Vec<ElementValuePair> = Vec::with_capacity(num_element_value_pairs as usize);
    for _ in 0..num_element_value_pairs {
        element_value_pairs.push(parse_element_value_pair(p)?);
    }
    Ok(Annotation { type_index, num_element_value_pairs, element_value_pairs })
}

pub fn annotation_to_bytes(annotation: Annotation) -> Vec<u8> {
    let mut res = vec![];
    let Annotation { type_index, num_element_value_pairs, element_value_pairs } = annotation;
    res.extend_from_slice(&type_index.to_be_bytes());
    res.extend_from_slice(&num_element_value_pairs.to_be_bytes());
    for ElementValuePair { element_name_index, value } in element_value_pairs {
        res.extend_from_slice(&element_name_index.to_be_bytes());
        res.extend_from_slice(&element_value_to_bytes(value));
    }
    res
}

fn parse_runtime_annotations_impl(p: &mut dyn ParsingContext) -> Result<Vec<Annotation>, ClassfileParsingError> {
    let num_annotations = p.read16()?;
    let mut annotations = Vec::with_capacity(num_annotations as usize);
    for _ in 0..num_annotations {
        annotations.push(parse_annotation(p)?);
    }
    Ok(annotations)
}

pub fn runtime_annotations_to_bytes(annotations: Vec<Annotation>) -> Vec<u8> {
    let mut res = vec![];
    let num_annotations = annotations.len() as u16;
    res.extend_from_slice(&num_annotations.to_be_bytes());
    for annotation in annotations {
        res.extend_from_slice(annotation_to_bytes(annotation).as_slice());
    }
    res
}

fn parse_runtime_visible_annotations(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let annotations = parse_runtime_annotations_impl(p)?;
    Ok(AttributeType::RuntimeVisibleAnnotations(RuntimeVisibleAnnotations { annotations }))
}

fn parse_runtime_invisible_annotations(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let annotations = parse_runtime_annotations_impl(p)?;
    Ok(AttributeType::RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotations { annotations }))
}

fn parse_runtime_parameter_annotations_impl(p: &mut dyn ParsingContext) -> Result<Vec<Vec<Annotation>>, ClassfileParsingError> {
    let mut parameter_annotations = vec![];
    let num_parameters = p.read8()?;
    for _ in 0..num_parameters {
        let num_annotations = p.read16()?;
        let mut annotations = vec![];
        for _ in 0..num_annotations {
            annotations.push(parse_annotation(p)?);
        }
        parameter_annotations.push(annotations);
    }
    Ok(parameter_annotations)
}

pub fn parameter_annotations_to_bytes(param_annotations: Vec<Vec<Annotation>>) -> Vec<u8> {
    let mut res = vec![];
    let num_parameters = param_annotations.len() as u8;
    res.push(num_parameters);
    for annotations in param_annotations {
        res.extend_from_slice(runtime_annotations_to_bytes(annotations).as_slice());
    }
    res
}


fn parse_runtime_visible_parameter_annotations(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let parameter_annotations = parse_runtime_parameter_annotations_impl(p)?;
    Ok(AttributeType::RuntimeVisibleParameterAnnotations(RuntimeVisibleParameterAnnotations { parameter_annotations }))
}

fn parse_runtime_invisible_parameter_annotations(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let parameter_annotations = parse_runtime_parameter_annotations_impl(p)?;
    Ok(AttributeType::RuntimeVisibleParameterAnnotations(RuntimeVisibleParameterAnnotations { parameter_annotations }))
}

fn parse_type_annotations_impl(p: &mut dyn ParsingContext) -> Result<Vec<TypeAnnotation>, ClassfileParsingError> {
    let num_annotations = p.read16()?;
    let mut annotations = vec![];
    for _ in 0..num_annotations {
        annotations.push(parse_type_annotation(p)?)
    }
    Ok(annotations)
}

fn parse_runtime_visible_type_annotations(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let annotations = parse_type_annotations_impl(p)?;
    Ok(AttributeType::RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotations { annotations }))
}

fn parse_runtime_invisible_type_annotations(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let annotations = parse_type_annotations_impl(p)?;
    Ok(AttributeType::RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotations { annotations }))
}

fn parse_type_annotation(p: &mut dyn ParsingContext) -> Result<TypeAnnotation, ClassfileParsingError> {
    let target_type_raw = p.read8()?;
    let target_type = match target_type_raw {
        0x00 | 0x01 => TargetInfo::TypeParameterTarget { type_parameter_index: p.read8()? },
        0x10 => TargetInfo::SuperTypeTarget { supertype_index: p.read16()? },
        0x11 | 0x12 => {
            let type_parameter_index = p.read8()?;
            let bound_index = p.read8()?;
            TargetInfo::TypeParameterBoundTarget { type_parameter_index, bound_index }
        }
        0x13 | 0x14 | 0x15 => TargetInfo::EmptyTarget,
        0x16 => TargetInfo::FormalParameterTarget { formal_parameter_index: p.read8()? },
        0x17 => TargetInfo::ThrowsTarget { throws_type_index: p.read16()? },
        0x40 | 0x41 => {
            let table_len = p.read16()?;
            let mut table = vec![];
            for _ in 0..table_len {
                let start_pc = p.read16()?;
                let length = p.read16()?;
                let index = p.read16()?;
                table.push(LocalVarTargetTableEntry { start_pc, length, index })
            }
            TargetInfo::LocalVarTarget { table }
        }
        0x42 => TargetInfo::CatchTarget { exception_table_entry: p.read16()? },
        0x43..=0x46 => TargetInfo::OffsetTarget { offset: p.read16()? },
        0x47..=0x4B => {
            let offset = p.read16()?;
            let type_argument_index = p.read8()?;
            TargetInfo::TypeArgumentTarget { offset, type_argument_index }
        }
        _ => return Err(ClassfileParsingError::WrongTag),
    };
    let target_path = parse_type_path(p)?;
    let type_index = p.read16()?;
    let mut element_value_pairs = vec![];
    let num_element_value_pairs = p.read16()?;
    for _ in 0..num_element_value_pairs {
        element_value_pairs.push(parse_element_value_pair(p)?);
    }
    Ok(TypeAnnotation { target_type, target_path, type_index, element_value_pairs })
}

fn parse_type_path(p: &mut dyn ParsingContext) -> Result<TypePath, ClassfileParsingError> {
    let length = p.read8()?;
    let mut path = vec![];
    for _ in 0..length {
        let type_path_kind = p.read8()?;
        let type_argument_index = p.read8()?;
        path.push(TypePathEntry { type_path_kind, type_argument_index });
    }
    Ok(TypePath { path })
}

fn parse_annotation_default(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let default_value = parse_element_value(p)?;
    Ok(AttributeType::AnnotationDefault(AnnotationDefault { default_value }))
}

pub fn annotation_default_to_bytes(annotations: AnnotationDefault) -> Vec<u8> {
    element_value_to_bytes(annotations.default_value)
}

fn parse_method_parameters(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let mut parameters = vec![];
    let parameters_count = p.read8()?;
    for _ in 0..parameters_count {
        let access_flags = p.read16()?;
        let name_index = access_flags;
        parameters.push(MethodParameter { name_index, access_flags })
    }
    Ok(AttributeType::MethodParameters(MethodParameters { parameters }))
}

fn parse_stack_map_table(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let number_of_entries = p.read16()?;
    let mut entries = Vec::with_capacity(number_of_entries as usize);
    for _ in 0..number_of_entries {
        entries.push(parse_stack_map_table_entry(p)?);
    }
    Ok(AttributeType::StackMapTable(StackMapTable { entries }))
}

fn parse_stack_map_table_entry(p: &mut dyn ParsingContext) -> Result<StackMapFrame, ClassfileParsingError> {
    let type_of_frame = p.read8()?;
    const SAME_FRAME_LOWER: u8 = 0;
    const SAME_FRAME_UPPER: u8 = 64;
    const SAME_LOCALS_1_STACK_LOWER: u8 = 64;
    const SAME_LOCALS_1_STACK_UPPER: u8 = 128;
    const RESERVED_LOWER: u8 = 128;
    const RESERVED_UPPER: u8 = 246;
    const SAME_LOCALS_1_STACK_ITEM_FRAME_EXTENDED: u8 = 247;
    const CHOP_FRAME_LOWER: u8 = 248;
    const CHOPE_FRAME_UPPER: u8 = 251;
    const SAME_FRAME_EXTENDED: u8 = 251;
    const APPEND_FRAME_LOWER: u8 = 252;
    const APPEND_FRAME_UPPER: u8 = 255;
    const FULL_FRAME: u8 = 255;
    Ok(match type_of_frame {
        SAME_FRAME_LOWER..SAME_FRAME_UPPER => StackMapFrame::SameFrame(SameFrame { offset_delta: type_of_frame as u16 }),
        SAME_LOCALS_1_STACK_LOWER..SAME_LOCALS_1_STACK_UPPER => StackMapFrame::SameLocals1StackItemFrame(SameLocals1StackItemFrame {
            offset_delta: (type_of_frame - SAME_LOCALS_1_STACK_LOWER) as u16,
            stack: parse_verification_type_info(p)?,
        }),
        RESERVED_LOWER..RESERVED_UPPER => {
            return Err(ClassfileParsingError::UsedReservedStackMapEntry);
        }
        SAME_LOCALS_1_STACK_ITEM_FRAME_EXTENDED => {
            let offset_delta = p.read16()?;
            let stack = parse_verification_type_info(p)?;
            StackMapFrame::SameLocals1StackItemFrameExtended(SameLocals1StackItemFrameExtended { offset_delta, stack })
        }
        CHOP_FRAME_LOWER..CHOPE_FRAME_UPPER => {
            let offset_delta = p.read16()?;
            let k_frames_to_chop = CHOPE_FRAME_UPPER - type_of_frame;
            StackMapFrame::ChopFrame(ChopFrame { offset_delta, k_frames_to_chop })
        }
        SAME_FRAME_EXTENDED => {
            let offset_delta = p.read16()?;
            StackMapFrame::SameFrameExtended(SameFrameExtended { offset_delta })
        }
        APPEND_FRAME_LOWER..APPEND_FRAME_UPPER => {
            let offset_delta = p.read16()?;
            let locals_size = type_of_frame - 251;
            let mut locals = Vec::with_capacity(locals_size as usize);
            for _ in 0..locals_size {
                locals.push(parse_verification_type_info(p)?)
            }
            StackMapFrame::AppendFrame(AppendFrame { offset_delta, locals })
        }
        FULL_FRAME => {
            let offset_delta = p.read16()?;
            let number_of_locals = p.read16()?;
            let mut locals = Vec::with_capacity(number_of_locals as usize);
            for _ in 0..number_of_locals {
                locals.push(parse_verification_type_info(p)?);
            }
            let number_of_stack_items = p.read16()?;
            let mut stack = Vec::with_capacity(number_of_stack_items as usize);
            for _ in 0..number_of_stack_items {
                stack.push(parse_verification_type_info(p)?);
            }
            StackMapFrame::FullFrame(FullFrame { offset_delta, number_of_locals, locals, number_of_stack_items, stack })
        }
        _ => {
            return Err(ClassfileParsingError::WrongStackMapFrameType);
        }
    })
}

fn parse_verification_type_info(p: &mut dyn ParsingContext) -> Result<PType, ClassfileParsingError> {
    let type_ = p.read8()?;
    const ITEM_TOP: u8 = 0;
    const ITEM_INTEGER: u8 = 1;
    const ITEM_FLOAT: u8 = 2;
    const ITEM_DOUBLE: u8 = 3;
    const ITEM_LONG: u8 = 4;
    const ITEM_NULL: u8 = 5;
    const ITEM_UNINITIALIZED_THIS: u8 = 6;
    const ITEM_OBJECT: u8 = 7;
    const ITEM_UNINITIALIZED: u8 = 8;
    Ok(match type_ {
        ITEM_TOP => PType::TopType,
        ITEM_INTEGER => PType::IntType,
        ITEM_FLOAT => PType::FloatType,
        ITEM_DOUBLE => PType::DoubleType,
        ITEM_LONG => PType::LongType,
        ITEM_NULL => PType::NullType,
        ITEM_UNINITIALIZED_THIS => PType::UninitializedThis,
        ITEM_OBJECT => {
            let original_index = p.read16()?;
            let index = match &p.constant_pool_borrow()[original_index as usize].kind {
                ConstantKind::Class(c) => c.name_index,
                _ => {
                    return Err(ClassfileParsingError::WromngCPEntry);
                }
            };
            let type_descriptor = p.constant_pool_borrow()[index as usize].extract_string_from_utf8().into_string()?;
            if type_descriptor.starts_with('[') {
                let res_descriptor = parse_field_descriptor(type_descriptor.as_str()).ok_or(ClassfileParsingError::WrongDescriptor)?;
                res_descriptor.field_type
            } else {
                PType::Ref(ReferenceType::Class(ClassName::Str(type_descriptor)))
            }
        }
        ITEM_UNINITIALIZED => PType::Uninitialized(UninitializedVariableInfo { offset: ByteCodeOffset(p.read16()?) }),
        _ => {
            return Err(ClassfileParsingError::WrongPtype);
        }
    })
}

fn parse_sourcefile(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let sourcefile_index = p.read16()?;
    Ok(AttributeType::SourceFile(SourceFile { sourcefile_index }))
}

fn parse_source_debug_extension(p: &mut dyn ParsingContext, len: u32) -> Result<AttributeType, ClassfileParsingError> {
    let mut debug_extension = vec![];
    for _ in 0..len {
        debug_extension.push(p.read8()?);
    }
    Ok(AttributeType::SourceDebugExtension(SourceDebugExtension { debug_extension }))
}

fn parse_local_variable_table(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let local_variable_table_length = p.read16()?;
    let mut local_variable_table = Vec::with_capacity(local_variable_table_length as usize);
    for _ in 0..local_variable_table_length {
        local_variable_table.push(read_local_variable_table_entry(p)?);
    }
    Ok(AttributeType::LocalVariableTable(LocalVariableTable { local_variable_table }))
}

fn read_local_variable_table_entry(p: &mut dyn ParsingContext) -> Result<LocalVariableTableEntry, ClassfileParsingError> {
    let start_pc = p.read16()?;
    let length = p.read16()?;
    let name_index = p.read16()?;
    let descriptor_index = p.read16()?;
    let index = p.read16()?;
    Ok(LocalVariableTableEntry { start_pc, length, name_index, descriptor_index, index })
}

fn parse_line_number_table(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let line_number_table_length = p.read16()?;
    let mut line_number_table = Vec::with_capacity(line_number_table_length as usize);
    for _ in 0..line_number_table_length {
        line_number_table.push(parse_line_number_table_entry(p)?);
    }
    Ok(AttributeType::LineNumberTable(LineNumberTable { line_number_table }))
}

fn parse_line_number_table_entry(p: &mut dyn ParsingContext) -> Result<LineNumberTableEntry, ClassfileParsingError> {
    let start_pc = ByteCodeOffset(p.read16()?);
    let line_number = LineNumber(p.read16()?);
    Ok(LineNumberTableEntry { start_pc, line_number })
}

fn parse_exception_table_entry(p: &mut dyn ParsingContext) -> Result<ExceptionTableElem, ClassfileParsingError> {
    let start_pc = ByteCodeOffset(p.read16()?);
    let end_pc = ByteCodeOffset(p.read16()?);
    let handler_pc = ByteCodeOffset(p.read16()?);
    let catch_type = p.read16()?;
    Ok(ExceptionTableElem { start_pc, end_pc, handler_pc, catch_type })
}

fn parse_code(p: &mut dyn ParsingContext) -> Result<AttributeType, ClassfileParsingError> {
    let max_stack = p.read16()?;
    let max_locals = p.read16()?;
    let code_length = p.read32()?;
    let mut code = Vec::with_capacity(code_length as usize);
    for _ in 0..code_length {
        code.push(p.read8()?);
    }
    let exception_table_length = p.read16()?;
    let mut exception_table = Vec::with_capacity(exception_table_length as usize);
    for _ in 0..exception_table_length {
        exception_table.push(parse_exception_table_entry(p)?);
    }
    let attributes_count = p.read16()?;
    let attributes = parse_attributes(p, attributes_count)?;

    let parsed_code = parse_code_raw(code.as_slice())?;
    Ok(AttributeType::Code(Code { max_stack, max_locals, code_raw: code, code: parsed_code, exception_table, attributes }))
}

pub fn parse_attributes(p: &mut dyn ParsingContext, num_attributes: u16) -> Result<Vec<AttributeInfo>, ClassfileParsingError> {
    let mut res = Vec::with_capacity(num_attributes as usize);
    for _ in 0..num_attributes {
        res.push(parse_attribute(p)?);
    }
    Ok(res)
}
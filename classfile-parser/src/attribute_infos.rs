use descriptor_parser::parse_field_descriptor;
use rust_jvm_common::classfile::{Annotation, AppendFrame, ArrayValue, AttributeInfo, AttributeType, BootstrapMethod, BootstrapMethods, ChopFrame, Code, ConstantKind, ConstantValue, Deprecated, ElementValue, ElementValuePair, Exceptions, ExceptionTableElem, FullFrame, InnerClass, InnerClasses, LineNumberTable, LineNumberTableEntry, LocalVariableTable, LocalVariableTableEntry, LocalVariableTypeTable, LocalVariableTypeTableEntry, NestHost, NestMembers, RuntimeVisibleAnnotations, SameFrame, SameFrameExtended, SameLocals1StackItemFrame, SameLocals1StackItemFrameExtended, Signature, SourceFile, StackMapFrame, StackMapTable, UninitializedVariableInfo};
use rust_jvm_common::classfile::EnclosingMethod;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::{PType, ReferenceType};

use crate::code::parse_code_raw;
use crate::constant_infos::is_utf8;
use crate::parsing_util::ParsingContext;

pub fn parse_attribute(p: &mut dyn ParsingContext) -> AttributeInfo {
    let attribute_name_index = p.read16();
    let attribute_length = p.read32();
    let name_pool = &p.constant_pool_borrow()[attribute_name_index as usize];
    assert!(is_utf8(&name_pool.kind).is_some());
    let name_struct = is_utf8(&name_pool.kind).expect("Classfile may be corrupted, invalid constant encountered.");
    let name = &name_struct.string;
    let attribute_type = if name == "Code" {
        parse_code(p)
    } else if name == "LineNumberTable" {
        parse_line_number_table(p)
    } else if name == "LocalVariableTable" {
        parse_local_variable_table(p)
    } else if name == "SourceFile" {
        parse_sourcefile(p)
    } else if name == "StackMapTable" {
        parse_stack_map_table(p)
    } else if name == "RuntimeVisibleAnnotations" {
        parse_runtime_visible_annotations(p)
    } else if name == "Signature" {
        parse_signature(p)
    } else if name == "Exceptions" {
        parse_exceptions(p)
    } else if name == "Deprecated" {
        parse_deprecated(p)
    } else if name == "InnerClasses" {
        parse_inner_classes(p)
    } else if name == "BootstrapMethods" {
        parse_bootstrap_methods(p)
    } else if name == "ConstantValue" {
        parse_constant_value_index(p)
    } else if name == "NestMembers" {
        //todo validate at most one constraints
        parse_nest_members(p)
    } else if name == "NestHost" {
        parse_nest_host(p)
    } else if name == "EnclosingMethod" {
        parse_enclosing_method(p)
    } else if name == "LocalVariableTypeTable" {
        parse_local_variable_type_table(p)
    } else {
        unimplemented!("{}", name);
    };
    AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type,
    }
}

fn parse_local_variable_type_table_entry(p: &mut dyn ParsingContext) -> LocalVariableTypeTableEntry {
    let start_pc = p.read16();
    let length = p.read16();
    let name_index = p.read16();
    let descriptor_index = p.read16();
    let index = p.read16();
    LocalVariableTypeTableEntry {
        start_pc,
        length,
        name_index,
        descriptor_index,
        index,
    }
}

fn parse_local_variable_type_table(p: &mut dyn ParsingContext) -> AttributeType {
    let local_variable_type_table_length = p.read16();
    let mut type_table = vec![];
    for _ in 0..local_variable_type_table_length {
        type_table.push(parse_local_variable_type_table_entry(p));
    }
    AttributeType::LocalVariableTypeTable(LocalVariableTypeTable { type_table })
}

fn parse_enclosing_method(p: &mut dyn ParsingContext) -> AttributeType {
    let class_index = p.read16();
    let method_index = p.read16();
    AttributeType::EnclosingMethod(EnclosingMethod { class_index, method_index })
}

fn parse_nest_host(p: &mut dyn ParsingContext) -> AttributeType {
    let host_class_index = p.read16();
    AttributeType::NestHost(NestHost { host_class_index })
}

fn parse_nest_members(p: &mut dyn ParsingContext) -> AttributeType {
    let number_of_classes = p.read16();
    let mut classes = Vec::with_capacity(number_of_classes as usize);
    for _ in 0..number_of_classes {
        classes.push(p.read16());
    }
    AttributeType::NestMembers(NestMembers { classes })
}

fn parse_constant_value_index(p: &mut dyn ParsingContext) -> AttributeType {
    let constant_value_index = p.read16();
    AttributeType::ConstantValue(ConstantValue {
        constant_value_index
    })
}

fn parse_bootstrap_methods(p: &mut dyn ParsingContext) -> AttributeType {
    let num_bootstrap_methods = p.read16();
    let mut bootstrap_methods = Vec::with_capacity(num_bootstrap_methods as usize);
    for _ in 0..num_bootstrap_methods {
        bootstrap_methods.push(parse_bootstrap_method(p));
    }
    AttributeType::BootstrapMethods(BootstrapMethods {
        bootstrap_methods
    })
}

fn parse_bootstrap_method(p: &mut dyn ParsingContext) -> BootstrapMethod {
    let bootstrap_method_ref = p.read16();
    let num_bootstrap_args = p.read16();
    let mut bootstrap_arguments = Vec::with_capacity(num_bootstrap_args as usize);
    for _ in 0..num_bootstrap_args {
        bootstrap_arguments.push(p.read16());
    }
    BootstrapMethod { bootstrap_arguments, bootstrap_method_ref }
}

fn parse_inner_class(p: &mut dyn ParsingContext) -> InnerClass {
    let inner_class_info_index = p.read16();
    let outer_class_info_index = p.read16();
    let inner_name_index = p.read16();
    let inner_class_access_flags = p.read16();
    InnerClass { inner_class_access_flags, inner_class_info_index, inner_name_index, outer_class_info_index }
}

fn parse_inner_classes(p: &mut dyn ParsingContext) -> AttributeType {
    let number_of_classes = p.read16();
    let mut classes = Vec::with_capacity(number_of_classes as usize);
    for _ in 0..number_of_classes {
        classes.push(parse_inner_class(p))
    }
    AttributeType::InnerClasses(
        InnerClasses {
            classes
        }
    )
}

fn parse_deprecated(_: &mut dyn ParsingContext) -> AttributeType {
    AttributeType::Deprecated(Deprecated {})
}

fn parse_exceptions(p: &mut dyn ParsingContext) -> AttributeType {
    let num_exceptions = p.read16();
    let mut exception_index_table = Vec::new();
    for _ in 0..num_exceptions {
        exception_index_table.push(p.read16());
    }
    AttributeType::Exceptions(Exceptions { exception_index_table })
}

fn parse_signature(p: &mut dyn ParsingContext) -> AttributeType {
    AttributeType::Signature(Signature { signature_index: p.read16() })
}

fn parse_element_value(p: &mut dyn ParsingContext) -> ElementValue {
    let tag = p.read8() as char;
    match tag {
        'B' => { unimplemented!() }
        'C' => { unimplemented!() }
        'S' => { unimplemented!() }
        's' => {
            ElementValue::String(p.read16())
        }
        '[' => {
            let num_values = p.read16();
            let mut values = vec![];
            for _ in 0..num_values {
                values.push(parse_element_value(p));
            }
            ElementValue::ArrayType(ArrayValue { values })
        }
        _ => { unimplemented!("{}", tag) }
    }
}

fn parse_element_value_pair(p: &mut dyn ParsingContext) -> ElementValuePair {
    let element_name_index = p.read16();
    let value = parse_element_value(p);
    ElementValuePair {
        element_name_index,
        value,
    }
}

fn parse_annotation(p: &mut dyn ParsingContext) -> Annotation {
    let type_index = p.read16();
    let num_element_value_pairs = p.read16();
    let mut element_value_pairs: Vec<ElementValuePair> = Vec::with_capacity(num_element_value_pairs as usize);
    for _ in 0..num_element_value_pairs {
        element_value_pairs.push(parse_element_value_pair(p));
    }
    Annotation {
        type_index,
        num_element_value_pairs,
        element_value_pairs,
    }
}

fn parse_runtime_visible_annotations(p: &mut dyn ParsingContext) -> AttributeType {
    let num_annotations = p.read16();
    let mut annotations = Vec::with_capacity(num_annotations as usize);
    for _ in 0..num_annotations {
        annotations.push(parse_annotation(p));
    }
    AttributeType::RuntimeVisibleAnnotations(RuntimeVisibleAnnotations { annotations })
}


fn parse_stack_map_table(p: &mut dyn ParsingContext) -> AttributeType {
    let number_of_entries = p.read16();
    let mut entries = Vec::with_capacity(number_of_entries as usize);
    for _ in 0..number_of_entries {
        entries.push(parse_stack_map_table_entry(p));
    }
    AttributeType::StackMapTable(StackMapTable { entries })
}

fn parse_stack_map_table_entry(p: &mut dyn ParsingContext) -> StackMapFrame {
    let type_of_frame = p.read8();
    //todo magic constants
//    match type_of_frame {
    if type_of_frame <= 63 {//todo <= or <
        StackMapFrame::SameFrame(SameFrame { offset_delta: type_of_frame as u16 })
    } else if 64 <= type_of_frame && type_of_frame <= 127 {
        StackMapFrame::SameLocals1StackItemFrame(SameLocals1StackItemFrame {
            offset_delta: (type_of_frame - 64) as u16,
            stack: parse_verification_type_info(p),
        })
    } else if 252 <= type_of_frame && type_of_frame <= 254 { //todo <= or <
        let offset_delta = p.read16();
        let locals_size = type_of_frame - 251;
        let mut locals = Vec::with_capacity(locals_size as usize);
        for _ in 0..locals_size {
            locals.push(parse_verification_type_info(p))
        }
        StackMapFrame::AppendFrame(AppendFrame {
            offset_delta,
            locals,
        })
    } else if type_of_frame == 255 {
        let offset_delta = p.read16();
        let number_of_locals = p.read16();
        let mut locals = Vec::with_capacity(number_of_locals as usize);
        for _ in 0..number_of_locals {
            locals.push(parse_verification_type_info(p));
        }
        let number_of_stack_items = p.read16();
        let mut stack = Vec::with_capacity(number_of_stack_items as usize);
        for _ in 0..number_of_stack_items {
            stack.push(parse_verification_type_info(p));
        }
        StackMapFrame::FullFrame(FullFrame {
            offset_delta,
            number_of_locals,
            locals,
            number_of_stack_items,
            stack,
        })
    } else if type_of_frame >= 248 && type_of_frame <= 250 {
        let offset_delta = p.read16();
        let k_frames_to_chop = 251 - type_of_frame;
        StackMapFrame::ChopFrame(ChopFrame { offset_delta, k_frames_to_chop })
    } else if type_of_frame == 251 {
        let offset_delta = p.read16();
        StackMapFrame::SameFrameExtended(SameFrameExtended { offset_delta })
    } else if type_of_frame == 247 {
        let offset_delta = p.read16();
        let stack = parse_verification_type_info(p);
        StackMapFrame::SameLocals1StackItemFrameExtended(SameLocals1StackItemFrameExtended { offset_delta, stack })
    } else {
        unimplemented!("{}", type_of_frame)
    }
}

fn parse_verification_type_info(p: &mut dyn ParsingContext) -> PType {
    let type_ = p.read8();
    //todo magic constants
    match type_ {
        0 => PType::TopType,
        1 => PType::IntType,
        2 => PType::FloatType,
        3 => PType::DoubleType,
        4 => PType::LongType,
        5 => PType::NullType,
        6 => PType::UninitializedThis,
        7 => {
            let original_index = p.read16();
            let index = match &p.constant_pool_borrow()[original_index as usize].kind {
                ConstantKind::Utf8(_u) => { panic!();/*original_index */ }
                ConstantKind::Class(c) => c.name_index,
                ConstantKind::String(_c) => panic!(),
                _ => { panic!() }
            };
            let type_descriptor = p.constant_pool_borrow()[index as usize].extract_string_from_utf8();
            if type_descriptor.starts_with('[') {
                let res_descriptor = parse_field_descriptor(type_descriptor.as_str()).unwrap();
                res_descriptor.field_type
            } else {
                PType::Ref(ReferenceType::Class(ClassName::Str(type_descriptor)))
            }
        }
        8 => { PType::Uninitialized(UninitializedVariableInfo { offset: p.read16() }) }
        _ => { unimplemented!("{}", type_) }
    }
}

fn parse_sourcefile(p: &mut dyn ParsingContext) -> AttributeType {
    let sourcefile_index = p.read16();
    AttributeType::SourceFile(
        SourceFile {
            sourcefile_index
        }
    )
}

fn parse_local_variable_table(p: &mut dyn ParsingContext) -> AttributeType {
    let local_variable_table_length = p.read16();
    let mut local_variable_table = Vec::with_capacity(local_variable_table_length as usize);
    for _ in 0..local_variable_table_length {
        local_variable_table.push(read_local_variable_table_entry(p));
    }
    AttributeType::LocalVariableTable(
        LocalVariableTable {
            local_variable_table,
        }
    )
}

fn read_local_variable_table_entry(p: &mut dyn ParsingContext) -> LocalVariableTableEntry {
    let start_pc = p.read16();
    let length = p.read16();
    let name_index = p.read16();
    let descriptor_index = p.read16();
    let index = p.read16();
    LocalVariableTableEntry {
        start_pc,
        length,
        name_index,
        descriptor_index,
        index,
    }
}

fn parse_line_number_table(p: &mut dyn ParsingContext) -> AttributeType {
    let line_number_table_length = p.read16();
    let mut line_number_table = Vec::with_capacity(line_number_table_length as usize);
    for _ in 0..line_number_table_length {
        line_number_table.push(parse_line_number_table_entry(p));
    }
    AttributeType::LineNumberTable(
        LineNumberTable {
            line_number_table,
        }
    )
}

fn parse_line_number_table_entry(p: &mut dyn ParsingContext) -> LineNumberTableEntry {
    let start_pc = p.read16();
    let line_number = p.read16();
    LineNumberTableEntry {
        start_pc,
        line_number,
    }
}

fn parse_exception_table_entry(p: &mut dyn ParsingContext) -> ExceptionTableElem {
    let start_pc = p.read16();
    let end_pc = p.read16();
    let handler_pc = p.read16();
    let catch_type = p.read16();
    ExceptionTableElem { start_pc, end_pc, handler_pc, catch_type }
}

fn parse_code(p: &mut dyn ParsingContext) -> AttributeType {
    let max_stack = p.read16();
    let max_locals = p.read16();
    let code_length = p.read32();
    let mut code = Vec::with_capacity(code_length as usize);
    for _ in 0..code_length {
        code.push(p.read8());
    }
    let exception_table_length = p.read16();
    let mut exception_table = Vec::with_capacity(exception_table_length as usize);
    for _ in 0..exception_table_length {
        exception_table.push(parse_exception_table_entry(p));
    }
    let attributes_count = p.read16();
    let attributes = parse_attributes(p, attributes_count);

    let parsed_code = parse_code_raw(code.as_slice());
    //todo add empty stackmap table
    AttributeType::Code(Code {
        max_stack,
        max_locals,
        code_raw: code,
        code: parsed_code,
        exception_table,
        attributes,
    })
}


pub fn parse_attributes(p: &mut dyn ParsingContext, num_attributes: u16) -> Vec<AttributeInfo> {
    let mut res = Vec::with_capacity(num_attributes as usize);
    for _ in 0..num_attributes {
        res.push(parse_attribute(p));
    }
    res
}
use crate::classfile::parsing_util::{ParsingContext, read16, read32, read8};
use rust_jvm_common::classfile::{AttributeInfo, NestHost, AttributeType, BootstrapMethods, ConstantValue, BootstrapMethod, InnerClass, InnerClasses, Deprecated, Exceptions, Signature, ElementValue, ElementValuePair, Annotation, RuntimeVisibleAnnotations, StackMapTable, StackMapFrame, SameFrame, AppendFrame, FullFrame, SameLocals1StackItemFrameExtended, ConstantKind, SourceFile, LocalVariableTable, LocalVariableTableEntry, LineNumberTable, LineNumberTableEntry, ExceptionTableElem, Code, SameFrameExtended, ChopFrame, SameLocals1StackItemFrame, NestMembers};
use crate::classfile::constant_infos::is_utf8;
use crate::classfile::code::parse_code_raw;
use rust_jvm_common::unified_types::UnifiedType;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::utils::extract_string_from_utf8;

pub fn parse_attribute(p: &mut ParsingContext) -> AttributeInfo {
    let attribute_name_index = read16(p);
    let attribute_length = read32(p);
    let name_pool = &p.constant_pool[attribute_name_index as usize];
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
    } else {
        unimplemented!("{}", name);
    };
    AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type,
    }
}

fn parse_nest_host(p: &mut ParsingContext) -> AttributeType {
    let host_class_index = read16(p);
    AttributeType::NestHost(NestHost { host_class_index })
}

fn parse_nest_members(p: &mut ParsingContext) -> AttributeType {
    let number_of_classes = read16(p);
    let mut classes = Vec::with_capacity(number_of_classes as usize);
    for _ in 0..number_of_classes {
        classes.push(read16(p));
    }
    AttributeType::NestMembers(NestMembers { classes })
}

fn parse_constant_value_index(p: &mut ParsingContext) -> AttributeType {
    let constant_value_index = read16(p);
    AttributeType::ConstantValue(ConstantValue {
        constant_value_index
    })
}

fn parse_bootstrap_methods(p: &mut ParsingContext) -> AttributeType {
    let num_bootstrap_methods = read16(p);
    let mut bootstrap_methods = Vec::with_capacity(num_bootstrap_methods as usize);
    bootstrap_methods.push(parse_bootstrap_method(p));
    AttributeType::BootstrapMethods(BootstrapMethods {
        bootstrap_methods
    })
}

fn parse_bootstrap_method(p: &mut ParsingContext) -> BootstrapMethod {
    let bootstrap_method_ref = read16(p);
    let num_bootstrap_args = read16(p);
    let mut bootstrap_arguments = Vec::with_capacity(num_bootstrap_args as usize);
    for _ in 0..num_bootstrap_args {
        bootstrap_arguments.push(read16(p));
    }
    BootstrapMethod { bootstrap_arguments, bootstrap_method_ref }
}

fn parse_inner_class(p: &mut ParsingContext) -> InnerClass {
    let inner_class_info_index = read16(p);
    let outer_class_info_index = read16(p);
    let inner_name_index = read16(p);
    let inner_class_access_flags = read16(p);
    InnerClass { inner_class_access_flags, inner_class_info_index, inner_name_index, outer_class_info_index }
}

fn parse_inner_classes(p: &mut ParsingContext) -> AttributeType {
    let number_of_classes = read16(p);
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

fn parse_deprecated(_: &mut ParsingContext) -> AttributeType {
    AttributeType::Deprecated(Deprecated {})
}

fn parse_exceptions(p: &mut ParsingContext) -> AttributeType {
    let num_exceptions = read16(p);
    let mut exception_index_table = Vec::new();
    for _ in 0..num_exceptions {
        exception_index_table.push(read16(p));
    }
    AttributeType::Exceptions(Exceptions { exception_index_table })
}

fn parse_signature(p: &mut ParsingContext) -> AttributeType {
    return AttributeType::Signature(Signature { signature_index: read16(p) });
}

fn parse_element_value(p: &mut ParsingContext) -> ElementValue {
    let tag = read8(p) as char;
    match tag {
        'B' => { unimplemented!() }
        'C' => { unimplemented!() }
        'S' => { unimplemented!() }
        's' => {
            ElementValue::String(read16(p))
        }
        _ => { unimplemented!("{}", tag) }
    }
}

fn parse_element_value_pair(p: &mut ParsingContext) -> ElementValuePair {
    let element_name_index = read16(p);
    let value = parse_element_value(p);
    return ElementValuePair {
        element_name_index,
        value,
    };
}

fn parse_annotation(p: &mut ParsingContext) -> Annotation {
    let type_index = read16(p);
    let num_element_value_pairs = read16(p);
    let mut element_value_pairs: Vec<ElementValuePair> = Vec::with_capacity(num_element_value_pairs as usize);
    for _ in 0..num_element_value_pairs {
        element_value_pairs.push(parse_element_value_pair(p));
    }
    return Annotation {
        type_index,
        num_element_value_pairs,
        element_value_pairs,
    };
}

fn parse_runtime_visible_annotations(p: &mut ParsingContext) -> AttributeType {
    let num_annotations = read16(p);
    let mut annotations = Vec::with_capacity(num_annotations as usize);
    for _ in 0..num_annotations {
        annotations.push(parse_annotation(p));
    }
    return AttributeType::RuntimeVisibleAnnotations(RuntimeVisibleAnnotations { annotations });
}


fn parse_stack_map_table(p: &mut ParsingContext) -> AttributeType {
    let number_of_entries = read16(p);
    let mut entries = Vec::with_capacity(number_of_entries as usize);
    for _ in 0..number_of_entries {
        entries.push(parse_stack_map_table_entry(p));
    }
    return AttributeType::StackMapTable(StackMapTable { entries });
}

fn parse_stack_map_table_entry(p: &mut ParsingContext) -> StackMapFrame {
    let type_of_frame = read8(p);
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
        let offset_delta = read16(p);
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
        let offset_delta = read16(p);
        let number_of_locals = read16(p);
        let mut locals = Vec::with_capacity(number_of_locals as usize);
        for _ in 0..number_of_locals {
            locals.push(parse_verification_type_info(p));
        }
        let number_of_stack_items = read16(p);
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
        let offset_delta = read16(p);
        let k_frames_to_chop = 251 - type_of_frame;
        StackMapFrame::ChopFrame(ChopFrame { offset_delta, k_frames_to_chop })
    } else if type_of_frame == 251 {
        let offset_delta = read16(p);
        StackMapFrame::SameFrameExtended(SameFrameExtended { offset_delta })
    } else if type_of_frame == 247 {
        let offset_delta = read16(p);
        let stack = parse_verification_type_info(p);
        StackMapFrame::SameLocals1StackItemFrameExtended(SameLocals1StackItemFrameExtended { offset_delta, stack })
    } else {
        unimplemented!("{}", type_of_frame)
    }
}

fn parse_verification_type_info(p: &mut ParsingContext) -> UnifiedType {
    let type_ = read8(p);
    //todo magic constants
    match type_ {
        0 => UnifiedType::TopType,
        1 => UnifiedType::IntType,
        2 => UnifiedType::FloatType,
        3 => UnifiedType::DoubleType,
        4 => UnifiedType::LongType,
        7 => {
            let original_index = read16(p);
            let index = match &p.constant_pool[original_index as usize].kind {
                ConstantKind::Utf8(u) => { panic!();original_index },
                ConstantKind::Class(c) => c.name_index,
                ConstantKind::String(c) => panic!(),
                _ => { panic!() }
            };
            UnifiedType::Class(ClassName::Str(extract_string_from_utf8(&p.constant_pool[index as usize])))
//            UnifiedType::ReferenceType(ClassName::Ref(NameReference { class_file:Rc::downgrade(classfile), index }))
        }

        _ => { unimplemented!("{}", type_) }
    }
}

fn parse_sourcefile(p: &mut ParsingContext) -> AttributeType {
    let sourcefile_index = read16(p);
    return AttributeType::SourceFile(
        SourceFile {
            sourcefile_index
        }
    );
}

fn parse_local_variable_table(p: &mut ParsingContext) -> AttributeType {
    let local_variable_table_length = read16(p);
    let mut local_variable_table = Vec::with_capacity(local_variable_table_length as usize);
    for _ in 0..local_variable_table_length {
        local_variable_table.push(read_local_variable_table_entry(p));
    }
    return AttributeType::LocalVariableTable(
        LocalVariableTable {
            local_variable_table,
        }
    );
}

fn read_local_variable_table_entry(p: &mut ParsingContext) -> LocalVariableTableEntry {
    let start_pc = read16(p);
    let length = read16(p);
    let name_index = read16(p);
    let descriptor_index = read16(p);
    let index = read16(p);
    return LocalVariableTableEntry {
        start_pc,
        length,
        name_index,
        descriptor_index,
        index,
    };
}

fn parse_line_number_table(p: &mut ParsingContext) -> AttributeType {
    let line_number_table_length = read16(p);
    let mut line_number_table = Vec::with_capacity(line_number_table_length as usize);
    for _ in 0..line_number_table_length {
        line_number_table.push(parse_line_number_table_entry(p));
    }
    return AttributeType::LineNumberTable(
        LineNumberTable {
            line_number_table,
        }
    );
}

fn parse_line_number_table_entry(p: &mut ParsingContext) -> LineNumberTableEntry {
    let start_pc = read16(p);
    let line_number = read16(p);
    return LineNumberTableEntry {
        start_pc,
        line_number,
    };
}

fn parse_exception_table_entry(p: &mut ParsingContext) -> ExceptionTableElem {
    let start_pc = read16(p);
    let end_pc = read16(p);
    let handler_pc = read16(p);
    let catch_type = read16(p);
    return ExceptionTableElem { start_pc, end_pc, handler_pc, catch_type };
}

fn parse_code(p: &mut ParsingContext) -> AttributeType {
    let max_stack = read16(p);
    let max_locals = read16(p);
    let code_length = read32(p);
    let mut code = Vec::with_capacity(code_length as usize);
    for _ in 0..code_length {
        code.push(read8(p));
    }
    let exception_table_length = read16(p);
    let mut exception_table = Vec::with_capacity(exception_table_length as usize);
    for _ in 0..exception_table_length {
        exception_table.push(parse_exception_table_entry(p));
    }
    let attributes_count = read16(p);
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


pub fn parse_attributes(p: &mut ParsingContext, num_attributes: u16) -> Vec<AttributeInfo> {
    let mut res = Vec::with_capacity(num_attributes as usize);
    for _ in 0..num_attributes {
        res.push(parse_attribute(p));
    }
    return res;
}
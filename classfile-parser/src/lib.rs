use std::io::{BufReader, Read};

use rust_jvm_common::classfile::{AttributeType, Classfile, Code, FieldInfo, MethodInfo, StackMapTable};

use crate::attribute_infos::parse_attributes;
use crate::constant_infos::parse_constant_infos;
use crate::parsing_util::ParsingContext;
use crate::parsing_util::ReadParsingContext;

pub mod attribute_infos;
pub mod code;
pub mod constant_infos;

pub fn stack_map_table_attribute(code: &Code) -> Option<&StackMapTable> {
    for attr in code.attributes.iter() {
        match &attr.attribute_type {
            AttributeType::StackMapTable(table) => {
                return Some(table);
            }
            _ => {}
        }
    }
    None
}


const EXPECTED_CLASSFILE_MAGIC: u32 = 0xCAFEBABE;


mod parsing_util;

pub fn parse_interfaces(p: &mut dyn ParsingContext, interfaces_count: u16) -> Vec<u16> {
    let mut res = Vec::with_capacity(interfaces_count as usize);
    for _ in 0..interfaces_count {
        res.push(p.read16())
    }
    return res;
}

pub fn parse_field(p: &mut dyn ParsingContext) -> FieldInfo {
    let access_flags = p.read16();
    let name_index = p.read16();
    let descriptor_index = p.read16();
    let attributes_count = p.read16();
    let attributes = parse_attributes(p, attributes_count);
    return FieldInfo { access_flags, name_index, descriptor_index, attributes };
}

pub fn parse_field_infos(p: &mut dyn ParsingContext, fields_count: u16) -> Vec<FieldInfo> {
    let mut res = Vec::with_capacity(fields_count as usize);
    for _ in 0..fields_count {
        res.push(parse_field(p))
    }
    return res;
}

pub fn parse_method(p: &mut dyn ParsingContext) -> MethodInfo {
    let access_flags = p.read16();
    let name_index = p.read16();
    let descriptor_index = p.read16();
    let attributes_count = p.read16();
    let attributes = parse_attributes(p, attributes_count);
    MethodInfo { access_flags, name_index, descriptor_index, attributes }
}

pub fn parse_methods(p: &mut dyn ParsingContext, methods_count: u16) -> Vec<MethodInfo> {
    let mut res = Vec::with_capacity(methods_count as usize);
    for _ in 0..methods_count {
        res.push(parse_method(p))
    }
    return res;
}

pub fn parse_class_file(read: &mut dyn Read) -> Classfile {
    let mut p = ReadParsingContext { constant_pool: None, read: &mut BufReader::new(read) };
    let mut class_file = parse_from_context(&mut p);
    class_file.constant_pool = p.constant_pool();//todo to avoid this yuckiness two pass parsing could be used
    class_file
}

fn parse_from_context(p: &mut dyn ParsingContext) -> Classfile {
    let magic: u32 = p.read32();
    // assert_eq!(magic, EXPECTED_CLASSFILE_MAGIC);
    let minor_version: u16 = p.read16();
    let major_version: u16 = p.read16();
    let constant_pool_count: u16 = p.read16();
    let constant_pool = parse_constant_infos(p, constant_pool_count);
    p.set_constant_pool(constant_pool);
    let access_flags = p.read16();
    let this_class = p.read16();
    let super_class = p.read16();
    let interfaces_count = p.read16();
    let interfaces = parse_interfaces(p, interfaces_count);
    let fields_count = p.read16();
    let fields = parse_field_infos(p, fields_count);
    let methods_count = p.read16();
    let methods = parse_methods(p, methods_count);
    let attributes_count = p.read16();
    let attributes = parse_attributes(p, attributes_count);
    let res = Classfile {
        magic,
        minor_version,
        major_version,
        constant_pool: vec![],
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    };
    // validate_parsed(&mut res);
    return res;
}

pub mod parse_validation;

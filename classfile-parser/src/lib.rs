pub mod attribute_infos;
pub mod code;
pub mod constant_infos;
use std::sync::Arc;
use std::fs::File;
use rust_jvm_common::classfile::{Code, AttributeType, StackMapTable, MethodInfo, ACC_ABSTRACT, FieldInfo, Classfile, ACC_NATIVE};
use crate::parsing_util::{ParsingContext, read16, read32};
use crate::attribute_infos::parse_attributes;
use crate::constant_infos::parse_constant_infos;


pub fn stack_map_table_attribute(code: &Code) -> Option<&StackMapTable> {
    for attr in code.attributes.iter() {
        match &attr.attribute_type {
            AttributeType::StackMapTable(table) => {
                return Some(table);//todo
            }
            _ => {}
        }
    }
    None
}

pub fn code_attribute(method_info: &MethodInfo) -> Option<&Code> {
    /*
    If the method is either native or abstract , and is not a class or interface
initialization method, then its method_info structure must not have a Code attribute
in its attributes table.
    */

    if (method_info.access_flags & ACC_ABSTRACT) > 0 || (method_info.access_flags & ACC_NATIVE) > 0 {
        return None;
    }

    for attr in method_info.attributes.iter() {
        match &attr.attribute_type {
            AttributeType::Code(code) => {
                return Some(code);
            }
            _ => {}
        }
    }
    panic!("Method has no code attribute, which is unusual given code is sorta the point of a method.")
}

const EXPECTED_CLASSFILE_MAGIC: u32 = 0xCAFEBABE;


mod parsing_util;

pub fn parse_interfaces(p: &mut ParsingContext, interfaces_count: u16) -> Vec<u16> {
    let mut res = Vec::with_capacity(interfaces_count as usize);
    for _ in 0..interfaces_count {
        res.push(read16(p))
    }
    return res;
}

pub fn parse_field(p: &mut ParsingContext) -> FieldInfo {
    let access_flags = read16(p);
    let name_index = read16(p);
    let descriptor_index = read16(p);
    let attributes_count = read16(p);
    let attributes = parse_attributes(p, attributes_count);
    return FieldInfo { access_flags, name_index, descriptor_index, attributes };
}

pub fn parse_field_infos(p: &mut ParsingContext, fields_count: u16) -> Vec<FieldInfo> {
    let mut res = Vec::with_capacity(fields_count as usize);
    for _ in 0..fields_count {
        res.push(parse_field(p))
    }
    return res;
}

pub fn parse_method(p: &mut ParsingContext) -> MethodInfo {
    let access_flags = read16(p);
    let name_index = read16(p);
    let descriptor_index = read16(p);
    let attributes_count = read16(p);
    let attributes = parse_attributes(p, attributes_count);
    MethodInfo { access_flags, name_index, descriptor_index, attributes }
}

pub fn parse_methods(p: &mut ParsingContext, methods_count: u16) -> Vec<MethodInfo> {
    let mut res = Vec::with_capacity(methods_count as usize);
    for _ in 0..methods_count {
        res.push(parse_method(p))
    }
    return res;
}

pub fn parse_class_file(f: File) -> Arc<Classfile> {
    let mut p = ParsingContext { constant_pool:&vec![] ,f};
    let magic: u32 = read32(&mut p);
    assert_eq!(magic, EXPECTED_CLASSFILE_MAGIC);
    let minor_version: u16 = read16(&mut p);
    let major_version: u16 = read16(&mut p);
    let constant_pool_count: u16 = read16(&mut p);
    let constant_pool = parse_constant_infos(&mut p, constant_pool_count);
    p.constant_pool = &constant_pool;
    let access_flags = read16(&mut p);
    let this_class = read16(&mut p);
    let super_class = read16(&mut p);
    let interfaces_count = read16(&mut p);
    let interfaces = parse_interfaces(&mut p, interfaces_count);
    let fields_count = read16(&mut p);
    let fields = parse_field_infos(&mut p, fields_count);
    let methods_count = read16(&mut p);
    let methods = parse_methods(&mut p, methods_count);
    let attributes_count = read16(&mut p);
    let attributes = parse_attributes(&mut p, attributes_count);
    let res = Arc::new(Classfile {
        magic,
        minor_version,
        major_version,
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    });
    return res;
}
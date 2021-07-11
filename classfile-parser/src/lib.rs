#![feature(exclusive_range_pattern)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::io::{BufReader, Read};

use rust_jvm_common::classfile::{Classfile, FieldInfo, MethodInfo};
use rust_jvm_common::loading::ClassfileParsingError;
use rust_jvm_common::loading::ClassfileParsingError::WrongMagic;
use sketch_jvm_version_of_utf8::ValidationError;

use crate::attribute_infos::parse_attributes;
use crate::constant_infos::parse_constant_infos;
use crate::parsing_util::ParsingContext;
use crate::parsing_util::ReadParsingContext;

pub mod attribute_infos;
pub mod code;
pub mod constant_infos;

const EXPECTED_CLASSFILE_MAGIC: u32 = 0xCAFEBABE;


mod parsing_util;

pub fn parse_interfaces(p: &mut dyn ParsingContext, interfaces_count: u16) -> Result<Vec<u16>, ClassfileParsingError> {
    let mut res = Vec::with_capacity(interfaces_count as usize);
    for _ in 0..interfaces_count {
        res.push(p.read16()?)
    }
    Ok(res)
}

pub fn parse_field(p: &mut dyn ParsingContext) -> Result<FieldInfo, ClassfileParsingError> {
    let access_flags = p.read16()?;
    let name_index = p.read16()?;
    let descriptor_index = p.read16()?;
    let attributes_count = p.read16()?;
    let attributes = parse_attributes(p, attributes_count)?;
    Ok(FieldInfo { access_flags, name_index, descriptor_index, attributes })
}

pub fn parse_field_infos(p: &mut dyn ParsingContext, fields_count: u16) -> Result<Vec<FieldInfo>, ClassfileParsingError> {
    let mut res = Vec::with_capacity(fields_count as usize);
    for _ in 0..fields_count {
        res.push(parse_field(p)?)
    }
    Ok(res)
}

pub fn parse_method(p: &mut dyn ParsingContext) -> Result<MethodInfo, ClassfileParsingError> {
    let access_flags = p.read16()?;
    let name_index = p.read16()?;
    let descriptor_index = p.read16()?;
    let attributes_count = p.read16()?;
    let attributes = parse_attributes(p, attributes_count)?;
    Ok(MethodInfo { access_flags, name_index, descriptor_index, attributes })
}

pub fn parse_methods(p: &mut dyn ParsingContext, methods_count: u16) -> Result<Vec<MethodInfo>, ClassfileParsingError> {
    let mut res = Vec::with_capacity(methods_count as usize);
    for _ in 0..methods_count {
        res.push(parse_method(p)?)
    }
    Ok(res)
}

pub fn parse_class_file(read: &mut dyn Read) -> Result<Classfile, ClassfileParsingError> {
    let mut p = ReadParsingContext { constant_pool: None, read: &mut BufReader::new(read) };
    let mut class_file = parse_from_context(&mut p)?;
    class_file.constant_pool = p.constant_pool();
    Ok(class_file)
}


fn parse_from_context(p: &mut dyn ParsingContext) -> Result<Classfile, ClassfileParsingError> {
    let magic: u32 = p.read32()?;
    if magic != EXPECTED_CLASSFILE_MAGIC {
        return Err(WrongMagic);
    }
    let minor_version: u16 = p.read16()?;
    let major_version: u16 = p.read16()?;
    let constant_pool_count: u16 = p.read16()?;
    let constant_pool = parse_constant_infos(p, constant_pool_count)?;
    p.set_constant_pool(constant_pool);
    let access_flags = p.read16()?;
    let this_class = p.read16()?;
    let super_class = p.read16()?;
    let interfaces_count = p.read16()?;
    let interfaces = parse_interfaces(p, interfaces_count)?;
    let fields_count = p.read16()?;
    let fields = parse_field_infos(p, fields_count)?;
    let methods_count = p.read16()?;
    let methods = parse_methods(p, methods_count)?;
    let attributes_count = p.read16()?;
    let attributes = parse_attributes(p, attributes_count)?;
    Ok(Classfile {
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
    })
}

pub mod parse_validation;

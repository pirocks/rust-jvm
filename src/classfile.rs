use std::hash::Hasher;

use classfile::attribute_infos::{AttributeType, Code, parse_attributes, StackMapTable};
use classfile::constant_infos::{ConstantInfo, parse_constant_infos};
use classfile::parsing_util::{read16, read32};
use classfile::parsing_util::ParsingContext;
use std::cell::RefCell;
use std::sync::Mutex;
use std::sync::RwLock;
use core::borrow::Borrow;
use std::sync::Arc;
use std::fs::File;

pub mod constant_infos;
pub mod attribute_infos;
pub mod code;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub attribute_type: AttributeType,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

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


//#[repr(u16)]
//pub enum ClassAccessFlags {
//TODO THIS NEEDS TO BE DIFFERENT FOR DIFFERNT TYPES
//maybe not but at very least is incomplete
pub const ACC_PUBLIC: u16 = 0x0001;
pub const ACC_PRIVATE: u16 = 0x0002;
pub const ACC_PROTECTED: u16 = 0x0004;
pub const ACC_STATIC: u16 = 0x0008;
pub const ACC_FINAL: u16 = 0x0010;
pub const ACC_SUPER: u16 = 0x0020;
pub const ACC_BRIDGE: u16 = 0x0040;
pub const ACC_VOLATILE: u16 = 0x0040;
pub const ACC_TRANSIENT: u16 = 0x0080;
pub const ACC_NATIVE: u16 = 0x0100;
pub const ACC_INTERFACE: u16 = 0x0200;
pub const ACC_ABSTRACT: u16 = 0x0400;
pub const ACC_STRICT: u16 = 0x0800;
pub const ACC_SYNTHETIC: u16 = 0x1000;
pub const ACC_ANNOTATION: u16 = 0x2000;
pub const ACC_ENUM: u16 = 0x4000;
pub const ACC_MODULE: u16 = 0x8000;
//}


#[derive(Debug)]
//#[derive(Eq)]
//#[derive(Copy, Clone)]
pub struct Classfile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantInfo>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

impl std::cmp::PartialEq for Classfile {
    fn eq(&self, other: &Self) -> bool {
        self.magic == other.magic &&
            self.minor_version == other.minor_version &&
            self.major_version == other.major_version &&
            self.constant_pool == other.constant_pool &&
            self.access_flags == other.access_flags &&
            self.this_class == other.this_class &&
            self.super_class == other.super_class &&
            self.interfaces == other.interfaces &&
            self.fields == other.fields &&
            self.methods == other.methods &&
            self.attributes == other.attributes
    }
}

impl std::hash::Hash for Classfile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.magic);
        state.write_u16(self.minor_version);
        state.write_u16(self.major_version);
        //todo constant_pool
        state.write_u16(self.access_flags);
        state.write_u16(self.this_class);
        state.write_u16(self.super_class);
        //todo interfaces
        //todo fields
        //todo methods
        //todo attributes
    }
}

pub mod parsing_util {
    use std::fs::File;
    use std::io::prelude::*;
    use classfile::constant_infos::ConstantInfo;

    pub struct ParsingContext<'l> {
        pub f: File,
        pub constant_pool : &'l Vec<ConstantInfo>
    }

    const IO_ERROR_MSG: &str = "Some sort of error in reading a classfile";

    pub fn read8(p: &mut ParsingContext) -> u8 {
        let mut buffer = [0; 1];
        let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
        assert_eq!(bytes_read, 1);
        return buffer[0];
    }

    pub fn read16(p: &mut ParsingContext) -> u16 {
        let mut buffer = [0; 2];
        let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
        assert_eq!(bytes_read, 2);
        return u16::from_be(((buffer[1] as u16) << 8) | buffer[0] as u16);
    }

    pub fn read32(p: &mut ParsingContext) -> u32 {
        let mut buffer = [0; 4];
        let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
        assert_eq!(bytes_read, 4);
        return u32::from_be(((buffer[0] as u32) << 0) +
            ((buffer[1] as u32) << 8) +
            ((buffer[2] as u32) << 16) +
            ((buffer[3] as u32) << 24));
    }
}

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
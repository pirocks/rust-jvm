use classfile::attribute_infos::parse_attributes;
use classfile::constant_infos::{ConstantInfo, parse_constant_infos};
use classfile::parsing_util::{read16, read32};
use classfile::parsing_util::ParsingContext;

mod constant_infos;
mod attribute_infos;

#[derive(Debug)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub attribute_type: attribute_infos::AttributeType,
}

#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>
}

#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>
}


const EXPECTED_CLASSFILE_MAGIC: u32 = 0xCAFEBABE;

//bitflag! {
//    pub struct ClassAccessFlags{
//        //TODO THIS NEEDS TO BE DIFFERENT FOR DIFFERNT TYPES
//        //todo probably should just use u16 + arithmeti
//    //maybe not but at very least is incomplete
//    ACC_PUBLIC = 0X0001,
//    ACC_PRIVATE = 0x0002,
//    ACC_PROTECTED = 0x0004,
//    ACC_STATIC = 0x0008,
//    ACC_FINAL = 0X0010,
//    ACC_SUPER = 0X0020,
//    ACC_BRIDGE = 0X0040,
//    ACC_VOLATILE = 0x0040,
//    ACC_TRANSIENT = 0x0080,
//    ACC_NATIVE = 0x0100,
//    ACC_INTERFACE = 0X0200,
//    ACC_ABSTRACT = 0X0400,
//    ACC_STRICT = 0x0800,
//    ACC_SYNTHETIC = 0X1000,
//    ACC_ANNOTATION = 0X2000,
//    ACC_ENUM = 0X4000,
//    ACC_MODULE = 0X8000
//    }
//}

#[derive(Debug)]
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


pub mod parsing_util {
    use std::fs::File;
    use std::io::prelude::*;

    pub struct ParsingContext {
        pub f: File
    }

    const IO_ERROR_MSG: &str = "Some sort of error in reading a classfile";

    pub fn read8(p: &mut ParsingContext) -> u8 {
        let mut buffer = [0; 1];
        let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
        assert!(bytes_read == 1);
        return buffer[0];
    }

    pub fn read16(p: &mut ParsingContext) -> u16 {
        let mut buffer = [0; 2];
        let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
        assert!(bytes_read == 2);
        return u16::from_be(((buffer[1] as u16) << 8) | buffer[0] as u16);
    }

    pub fn read32(p: &mut ParsingContext) -> u32 {
        let mut buffer = [0; 4];
        let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
        assert!(bytes_read == 4);
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

pub fn parse_field(p: &mut ParsingContext, constant_pool: &Vec<ConstantInfo>) -> FieldInfo {
    let access_flags = read16(p);
    let name_index = read16(p);
    let descriptor_index = read16(p);
    let attributes_count = read16(p);
    let attributes = parse_attributes(p, attributes_count,constant_pool);
    return FieldInfo { access_flags, name_index, descriptor_index, attributes }
}

pub fn parse_field_infos(p: &mut ParsingContext, fields_count: u16, constant_pool: &Vec<ConstantInfo>) -> Vec<FieldInfo> {
    let mut res = Vec::with_capacity(fields_count as usize);
    for _ in 0..fields_count {
        res.push(parse_field(p,constant_pool))
    }
    return res;
}

pub fn parse_method(p: &mut ParsingContext,  constant_pool: &Vec<ConstantInfo>) -> MethodInfo{
    let access_flags = read16(p);
    let name_index = read16(p);
    let descriptor_index = read16(p);
    let attributes_count = read16(p);
    let attributes = parse_attributes(p,attributes_count, constant_pool);
    return MethodInfo { access_flags, name_index, descriptor_index, attributes }
}

pub fn parse_methods(p: &mut ParsingContext, methods_count: u16,  constant_pool: &Vec<ConstantInfo>) -> Vec<MethodInfo> {
    let mut res = Vec::with_capacity(methods_count as usize);
    for _ in 0..methods_count {
        res.push(parse_method(p,constant_pool))
    }
    return res;
}

pub fn parse_class_file(p: &mut ParsingContext) -> Classfile {
    let magic: u32 = read32(p);
    assert!(magic == EXPECTED_CLASSFILE_MAGIC);
    let minor_version: u16 = read16(p);
    let major_version: u16 = read16(p);
    let constant_pool_count: u16 = read16(p);
    dbg!(minor_version,major_version,constant_pool_count);
    let constant_pool = parse_constant_infos(p, constant_pool_count);
    let access_flags: u16 = read16(p);
    let this_class: u16 = read16(p);
    let super_class: u16 = read16(p);
    let interfaces_count: u16 = read16(p);
    let interfaces: Vec<u16> = parse_interfaces(p, interfaces_count);
    let fields_count: u16 = read16(p);
    let fields: Vec<FieldInfo> = parse_field_infos(p, fields_count,&constant_pool);
    let methods_count: u16 = read16(p);
    let methods: Vec<MethodInfo> = parse_methods(p, methods_count,&constant_pool);
    let attributes_count: u16 = read16(p);
    let attributes: Vec<AttributeInfo> = parse_attributes(p, attributes_count,&constant_pool);
    return Classfile {
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
    };
}
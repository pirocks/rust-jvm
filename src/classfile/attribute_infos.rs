use std::borrow::Borrow;

use classfile::AttributeInfo;
use classfile::constant_infos::{ConstantInfo, is_utf8};
use classfile::parsing_util::{ParsingContext, read16, read32, read8};

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SourceFile{
    //todo
    pub sourcefile_index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InnerClasses{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct EnclosingMethod{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SourceDebugExtension{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct BootstrapMethods{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Module{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct NestHost{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ConstantValue{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Code{
    //todo
    pub attributes: Vec<AttributeInfo>,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableElem>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ExceptionTableElem {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Exceptions{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeVisibleParameterAnnotations{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeInvisibleParameterAnnotations{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct AnnotationDefault{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct MethodParameters{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Synthetic{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Deprecated{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct Signature{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeVisibleAnnotations{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeInvisibleAnnotations{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LineNumberTable{
    //todo
    pub line_number_table: Vec<LineNumberTableEntry>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LocalVariableTable{
    //todo
    pub local_variable_table: Vec<LocalVariableTableEntry>
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LocalVariableTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct LocalVariableTypeTable{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct StackMapTable{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeVisibleTypeAnnotations{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct RuntimeInvisibleTypeAnnotations{
    //todo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum AttributeType{
    SourceFile(SourceFile),
    InnerClasses(InnerClasses),
    EnclosingMethod(EnclosingMethod),
    SourceDebugExtension(SourceDebugExtension),
    BootstrapMethods(BootstrapMethods),
    Module(Module),
    NestHost(NestHost),
    ConstantValue(ConstantValue),
    Code(Code),
    Exceptions(Exceptions),
    RuntimeVisibleParameterAnnotations(RuntimeVisibleParameterAnnotations),
    RuntimeInvisibleParameterAnnotations(RuntimeInvisibleParameterAnnotations),
    AnnotationDefault(AnnotationDefault),
    MethodParameters(MethodParameters),
    Synthetic(Synthetic),
    Deprecated(Deprecated),
    Signature(Signature),
    RuntimeVisibleAnnotations(RuntimeVisibleAnnotations),
    RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotations),
    LineNumberTable(LineNumberTable),
    LocalVariableTable(LocalVariableTable),
    LocalVariableTypeTable(LocalVariableTypeTable),
    StackMapTable(StackMapTable),
    RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotations),
    RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotations),
}

pub fn parse_attribute(p: &mut ParsingContext, constant_pool: &Vec<ConstantInfo>) -> AttributeInfo {
    let attribute_name_index = read16(p);
    let attribute_length = read32(p);
//    uint64_t cur = ;
    let name_pool = constant_pool[attribute_name_index as usize].borrow();
    assert!(is_utf8(&name_pool.kind).is_some());
    let name_struct = is_utf8(&name_pool.kind).expect("Classfile may be corrupted, invalid constant encountered.");
    let name = &name_struct.string;
    if name == "Code" {
        return parse_code(p, attribute_name_index, attribute_length,constant_pool)
    } else if name == "LineNumberTable" {
        return parse_line_number_table(p, attribute_name_index, attribute_length)
    } else if name == "LocalVariableTable" {
        return parse_local_variable_table(p, attribute_name_index, attribute_length)
    } else if name == "SourceFile" {
        return parse_sourcefile(p, attribute_name_index, attribute_length)
    } else {
        unimplemented!()
    }
}

fn parse_sourcefile(p: &mut ParsingContext, attribute_name_index: u16, attribute_length: u32) -> AttributeInfo {
    let sourcefile_index = read16(p);
    return AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type: AttributeType::SourceFile(
            SourceFile {
                sourcefile_index
            }
        ),
    }
}

fn parse_local_variable_table(p: &mut ParsingContext, attribute_name_index: u16, attribute_length: u32) -> AttributeInfo {
    let local_variable_table_length = read16(p);
    let mut local_variable_table = Vec::with_capacity(local_variable_table_length as usize);
    for _ in 0..local_variable_table_length {
        local_variable_table.push(read_local_variable_table_entry(p));
    }
    return AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type: AttributeType::LocalVariableTable(
            LocalVariableTable {
                local_variable_table,
            }
        ),
    }
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

fn parse_line_number_table(p: &mut ParsingContext, attribute_name_index: u16, attribute_length: u32) -> AttributeInfo {
    let line_number_table_length = read16(p);
    let mut line_number_table = Vec::with_capacity(line_number_table_length as usize);
    for _ in 0..line_number_table_length {
        line_number_table.push(parse_line_number_table_entry(p));
    }
    return AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type: AttributeType::LineNumberTable(
            LineNumberTable {
                line_number_table,
            }
        ),
    }
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
    return ExceptionTableElem { start_pc, end_pc, handler_pc, catch_type }
}

fn parse_code(p: &mut ParsingContext, attribute_name_index: u16, attribute_length: u32, constant_pool: &Vec<ConstantInfo>) -> AttributeInfo {
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
    let attributes = parse_attributes(p, attributes_count,constant_pool);
    return AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type: AttributeType::Code(Code {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        }),
    }
}


pub fn parse_attributes(p: &mut ParsingContext, num_attributes: u16, constant_pool: &Vec<ConstantInfo>) -> Vec<AttributeInfo> {
    let mut res = Vec::with_capacity(num_attributes as usize);
    for _ in 0..num_attributes {
        res.push(parse_attribute(p,constant_pool));
    }
    return res;
}
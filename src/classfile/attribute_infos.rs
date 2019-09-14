use std::borrow::Borrow;
use std::fs::read;

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
    pub signature_index: u16
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
//pub struct UninitializedThisVariableInfo {}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ObjectVariableInfo {
    pub cpool_index: Option<u16>,
    pub class_name: String,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayVariableInfo {
    pub sub_type: Box<VerificationTypeInfo>
}

//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//pub struct TopVariableInfo {}
//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//pub struct IntegerVariableInfo {}
//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//pub struct FloatVariableInfo {}
//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//pub struct LongVariableInfo {}
//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//pub struct DoubleVariableInfo {}
//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//pub struct NullVariableInfo {}
//
//#[derive(Debug)]
//#[derive(Eq, PartialEq)]
//#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct UninitializedVariableInfo {
    pub offset: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitializedThis,
    Object(ObjectVariableInfo),
    Uninitialized(UninitializedVariableInfo),
    Array(ArrayVariableInfo),

}



#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SameFrame {
    pub offset_delta: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SameLocals1StackItemFrame {
    pub offset_delta: u16,
    pub stack: VerificationTypeInfo
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SameLocals1StackItemFrameExtended {
    pub offset_delta: u16,
    pub stack: VerificationTypeInfo,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ChopFrame {
    pub offset_delta: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct SameFrameExtended {
    pub offset_delta: u16
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct AppendFrame {
    pub offset_delta: u16,
    pub locals: Vec<VerificationTypeInfo>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct FullFrame {
    pub offset_delta: u16,
    pub number_of_locals: u16,
    pub locals: Vec<VerificationTypeInfo>,
    pub number_of_stack_items: u16,
    pub stack: Vec<VerificationTypeInfo>,
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum StackMapFrame {
    SameFrame(SameFrame),
    SameLocals1StackItemFrame(SameLocals1StackItemFrame),
    SameLocals1StackItemFrameExtended(SameLocals1StackItemFrameExtended),
    ChopFrame(ChopFrame),
    SameFrameExtended(SameFrameExtended),
    AppendFrame(AppendFrame),
    FullFrame(FullFrame),
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct StackMapTable{
    pub entries: Vec<StackMapFrame>
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
    } else if name == "StackMapTable" {
        return parse_stack_map_table(p, attribute_name_index, attribute_length)
    } else if name == "RuntimeVisibleAnnotations" {
        return parse_runtime_visible_annotations(p, attribute_name_index, attribute_length)
    } else if name == "Signature" {
        return parse_signature(p, attribute_name_index, attribute_length)
    } else {
        unimplemented!("{}", name);
    }
}

fn parse_signature(p: &mut ParsingContext, attribute_name_index: u16, attribute_length: u32) -> AttributeInfo {
    return AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type: AttributeType::Signature(Signature { signature_index: read16(p) }),
    }
}

type CPIndex = u16;//todo use this more

pub struct EnumConstValue {
    pub type_name_index: u16,
    pub const_name_index: u16,
}

pub struct ClassInfoIndex {
    pub class_info_index: u16
}

pub struct AnnotationValue {
    pub annotation: Annotation
}

pub struct ArrayValue {
    pub values: Vec<ElementValue>
}

pub enum ElementValue {
    Byte(CPIndex),
    Char(CPIndex),
    Double(CPIndex),
    Float(CPIndex),
    Int(CPIndex),
    Long(CPIndex),
    Short(CPIndex),
    Boolean(CPIndex),
    String(CPIndex),
    EnumType(EnumConstValue),
    Class(ClassInfoIndex),
    AnnotationType(AnnotationValue),
    ArrayType(ArrayValue),
}

pub struct ElementValuePair {
    pub element_name_index: u16,
    pub value: ElementValue,
}

pub struct Annotation {
    pub type_index: u16,
    pub num_element_value_pairs: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}

fn parse_element_value(p: &mut ParsingContext) -> ElementValue {
    let tag = read8(p) as char;
    match tag {
        'B' => { unimplemented!() }
        'C' => { unimplemented!() }
        _ => { unimplemented!("{}", tag) }
    }
}

fn parse_element_value_pair(p: &mut ParsingContext) -> ElementValuePair {
    let element_name_index = read16(p);
    let value = parse_element_value(p);
    return ElementValuePair {
        element_name_index,
        value,
    }
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
    }
}

fn parse_runtime_visible_annotations(p: &mut ParsingContext, attribute_name_index: u16, attribute_length: u32) -> AttributeInfo {
    let num_annotations = read16(p);
    let mut annotations = Vec::with_capacity(num_annotations as usize);
    for _ in 0..num_annotations {
        annotations.push(parse_annotation(p));
    }
    return AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type: AttributeType::RuntimeVisibleAnnotations(RuntimeVisibleAnnotations {}),
    }
}


fn parse_stack_map_table(p: &mut ParsingContext, attribute_name_index: u16, attribute_length: u32) -> AttributeInfo {
    let number_of_entries = read16(p);
    let mut entries = Vec::with_capacity(number_of_entries as usize);
    for _ in 0..number_of_entries {
        entries.push(parse_stack_map_table_entry(p));
    }
    return AttributeInfo {
        attribute_name_index,
        attribute_length,
        attribute_type: AttributeType::StackMapTable(StackMapTable { entries }),
    }
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
    } else {
        unimplemented!("{}", type_of_frame)
    }
}

fn parse_verification_type_info(p: &mut ParsingContext) -> VerificationTypeInfo{
    let type_ = read8(p);
    //todo magic constants
    match type_ {
        1 => VerificationTypeInfo::Integer,
        _ => { unimplemented!("{}", type_) }
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
    //todo add empty stackmap table
    AttributeInfo {
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
use rust_jvm_common::classfile::{Class, ConstantInfo, ConstantKind, Fieldref, Float, Integer, InterfaceMethodref, InvalidConstant, InvokeDynamic, Long, MethodHandle, Methodref, MethodType, NameAndType, String_, Utf8};
use rust_jvm_common::classfile::*;
use rust_jvm_common::classfile::Double;
use rust_jvm_common::classfile::ReferenceKind::{GetField, GetStatic, InvokeInterface, InvokeSpecial, InvokeStatic, InvokeVirtual, NewInvokeSpecial, PutField, PutStatic};
use sketch_jvm_version_of_utf8::PossiblyJVMString;

use crate::ClassfileParsingError;
use crate::parsing_util::ParsingContext;

pub(crate) fn is_utf8(utf8: &ConstantKind) -> Option<&Utf8> {
    match utf8 {
        ConstantKind::Utf8(s) => { Some(s) }
        _ => { None }
    }
}

const UTF8_CONST_NUM: u8 = 1;
const INTEGER_CONST_NUM: u8 = 3;
const FLOAT_CONST_NUM: u8 = 4;
const LONG_CONST_NUM: u8 = 5;
const DOUBLE_CONST_NUM: u8 = 6;
const CLASS_CONST_NUM: u8 = 7;
const STRING_CONST_NUM: u8 = 8;
const FIELDREF_CONST_NUM: u8 = 9;
const METHODREF_CONST_NUM: u8 = 10;
const INTERFACE_METHODREF_CONST_NUM: u8 = 11;
const NAME_AND_TYPE_CONST_NUM: u8 = 12;
const METHOD_HANDLE_CONST_NUM: u8 = 15;
const METHOD_TYPE_CONST_NUM: u8 = 16;
const INVOKE_DYNAMIC_CONST_NUM: u8 = 18;

pub fn parse_constant_info(p: &mut dyn ParsingContext, _debug: bool) -> Result<ConstantInfo, ClassfileParsingError> {
    let kind = p.read8()?;
    let result_kind: ConstantKind = match kind {
        UTF8_CONST_NUM => {
            let length = p.read16()?;
            let mut buffer = Vec::new();
            for _ in 0..length {
                buffer.push(p.read8()?)
            }
            let str_ = PossiblyJVMString::new(buffer).validate(true)?.to_wtf8();
            ConstantKind::Utf8(Utf8 { length, string: str_ })
        }
        INTEGER_CONST_NUM => {
            let bytes = p.read32()?;
            ConstantKind::Integer(Integer { bytes })
        }
        FLOAT_CONST_NUM => {
            let bytes = p.read32()?;
            ConstantKind::Float(Float { bytes })
        }
        LONG_CONST_NUM => {
            let high_bytes = p.read32()?;
            let low_bytes = p.read32()?;
            ConstantKind::Long(Long { high_bytes, low_bytes })
        }
        DOUBLE_CONST_NUM => {
            let high_bytes = p.read32()?;
            let low_bytes = p.read32()?;
            ConstantKind::Double(Double {
                high_bytes,
                low_bytes,
            })
        }
        CLASS_CONST_NUM => {
            let name_index = p.read16()?;
            ConstantKind::Class(Class { name_index })
        }
        STRING_CONST_NUM => {
            let string_index = p.read16()?;
            ConstantKind::String(String_ { string_index })
        }
        FIELDREF_CONST_NUM => {
            let class_index = p.read16()?;
            let name_and_type_index = p.read16()?;
            ConstantKind::Fieldref(Fieldref { class_index, name_and_type_index })
        }
        METHODREF_CONST_NUM => {
            let class_index = p.read16()?;
            let name_and_type_index = p.read16()?;
            ConstantKind::Methodref(Methodref { class_index, name_and_type_index })
        }
        INTERFACE_METHODREF_CONST_NUM => {
            let class_index = p.read16()?;
            let nt_index = p.read16()?;
            ConstantKind::InterfaceMethodref(InterfaceMethodref { class_index, nt_index })
        }
        NAME_AND_TYPE_CONST_NUM => {
            let name_index = p.read16()?;
            let descriptor_index = p.read16()?;
            ConstantKind::NameAndType(NameAndType { name_index, descriptor_index })
        }
        METHOD_HANDLE_CONST_NUM => {

            //1 REF_getField getfield C.f:T
            // 2 REF_getStatic getstatic C.f:T
            // 3 REF_putField putfield C.f:T
            // 4 REF_putStatic putstatic C.f:T
            // 5 REF_invokeVirtual invokevirtual C.m:(A*)T
            // 6 REF_invokeStatic invokestatic C.m:(A*)T
            // 7 REF_invokeSpecial invokespecial C.m:(A*)T
            // 8 REF_newInvokeSpecial new
            // C;
            // dup;
            // C.<init>:(A*)V
            // 9 REF_invokeInterface invokeinterface C.m:(A*)T
            let reference_kind = match p.read8()? {
                REF_GET_FIELD => GetField,
                REF_GET_STATIC => GetStatic,
                REF_PUT_FIELD => PutField,
                REF_PUT_STATIC => PutStatic,
                REF_INVOKE_VIRTUAL => InvokeVirtual,
                REF_INVOKE_STATIC => InvokeStatic,
                REF_INVOKE_SPECIAL => InvokeSpecial,
                REF_NEW_INVOKE_SPECIAL => NewInvokeSpecial,
                REF_INVOKE_INTERFACE => InvokeInterface,
                _ => {
                    return Err(ClassfileParsingError::WrongTag);
                }
            };
            let reference_index = p.read16()?;
            ConstantKind::MethodHandle(MethodHandle {
                reference_kind,
                reference_index,
            })
        }
        METHOD_TYPE_CONST_NUM => {
            let descriptor_index = p.read16()?;
            ConstantKind::MethodType(MethodType {
                descriptor_index
            })
        }
        INVOKE_DYNAMIC_CONST_NUM => {
            let bootstrap_method_attr_index = p.read16()?;
            let name_and_type_index = p.read16()?;
            ConstantKind::InvokeDynamic(InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            })
        }
        _ => {
            return Err(ClassfileParsingError::WrongTag);
        }
    };
    Ok(ConstantInfo { kind: result_kind })
}


pub fn parse_constant_infos(p: &mut dyn ParsingContext, constant_pool_count: u16) -> Result<Vec<ConstantInfo>, ClassfileParsingError> {
    let mut constants = Vec::with_capacity(constant_pool_count as usize);
    let mut skip_next_iter = true;
    //skip first loop iteration b/c the first element of the constant pool isn't a thing
    for i in 0..constant_pool_count {
        if skip_next_iter {
            constants.push(ConstantInfo { kind: (ConstantKind::InvalidConstant(InvalidConstant {})) });
            skip_next_iter = false;
            continue;
        }
        // dbg!(i);
        let constant_info = parse_constant_info(p, i == 473)?;
        match constant_info.kind {
            ConstantKind::Long(_) | ConstantKind::Double(_) => {
                skip_next_iter = true;
            }
            _ => {}
        }
        constants.push(constant_info);
    }
    Ok(constants)
}

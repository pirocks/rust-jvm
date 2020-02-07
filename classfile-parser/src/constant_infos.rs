use crate::parsing_util::ParsingContext;
use rust_jvm_common::classfile::{ConstantKind, Utf8, Integer, Float, Long, Fieldref, Methodref, MethodType, NameAndType, InterfaceMethodref, MethodHandle, InvokeDynamic, ConstantInfo, InvalidConstant, Class, String_};
use rust_jvm_common::classfile::Double;

pub(crate) fn is_utf8(utf8: &ConstantKind) -> Option<&Utf8> {
    return match utf8 {
        ConstantKind::Utf8(s) => { Some(s) }
        _ => { None }
    };
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
const DYNAMIC_CONST_NUM: u8 = 17;
const INVOKE_DYNAMIC_CONST_NUM: u8 = 18;
const MODULE_CONST_NUM: u8 = 19;
const PACKAGE_CONST_NUM: u8 = 20;
const INVALID_CONSTANT_CONST_NUM: u8 = 21;

pub fn parse_constant_info(p: &mut dyn ParsingContext) -> ConstantInfo {
    let kind = p.read8();
    let result_kind: ConstantKind = match kind {
        UTF8_CONST_NUM => {
            let length = p.read16();
            let mut buffer = Vec::new();
            for _ in 0..length {
                buffer.push(p.read8())
            }
            let str_ = unsafe {String::from_utf8_unchecked(buffer)};//.expect("Invalid utf8 in constant pool");
            ConstantKind::Utf8(Utf8 { length, string: str_ })
        }
        INTEGER_CONST_NUM => {
            let bytes = p.read32();
            ConstantKind::Integer(Integer { bytes })
        }
        FLOAT_CONST_NUM => {
            let bytes = p.read32();
            ConstantKind::Float(Float { bytes })
        }
        LONG_CONST_NUM => {
            let high_bytes = p.read32();
            let low_bytes = p.read32();
            ConstantKind::Long(Long { high_bytes, low_bytes })
        }
        DOUBLE_CONST_NUM => {
            let high_bytes = p.read32();
            let low_bytes = p.read32();
            ConstantKind::Double(Double {
                high_bytes,
                low_bytes,
            })
        }
        CLASS_CONST_NUM => {
            let name_index = p.read16();
            ConstantKind::Class(Class { name_index })
        }
        STRING_CONST_NUM => {
            let string_index = p.read16();
            ConstantKind::String(String_ { string_index })
        }
        FIELDREF_CONST_NUM => {
            let class_index = p.read16();
            let name_and_type_index = p.read16();
            ConstantKind::Fieldref(Fieldref { class_index, name_and_type_index })
        }
        METHODREF_CONST_NUM => {
            let class_index = p.read16();
            let name_and_type_index = p.read16();
            ConstantKind::Methodref(Methodref { class_index, name_and_type_index })
        }
        INTERFACE_METHODREF_CONST_NUM => {
            let class_index = p.read16();
            let nt_index = p.read16();
            ConstantKind::InterfaceMethodref(InterfaceMethodref { class_index, nt_index })
        }
        NAME_AND_TYPE_CONST_NUM => {
            let name_index = p.read16();
            let descriptor_index = p.read16();
            ConstantKind::NameAndType(NameAndType { name_index, descriptor_index })
        }
        METHOD_HANDLE_CONST_NUM => {
            let reference_kind = p.read8();
            let reference_index = p.read16();
            ConstantKind::MethodHandle(MethodHandle {
                reference_kind,
                reference_index,
            })
        }
        METHOD_TYPE_CONST_NUM => {
            let descriptor_index = p.read16();
            ConstantKind::MethodType(MethodType {
                descriptor_index
            })
        }
        DYNAMIC_CONST_NUM => { unimplemented!() }
        INVOKE_DYNAMIC_CONST_NUM => {
            let bootstrap_method_attr_index = p.read16();
            let name_and_type_index = p.read16();
            ConstantKind::InvokeDynamic(InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            })
        }
        MODULE_CONST_NUM => { unimplemented!() }
        PACKAGE_CONST_NUM => { unimplemented!() }
        INVALID_CONSTANT_CONST_NUM => {
            assert!(false);
            unimplemented!();
        }
        _ => {
            dbg!(kind);
            assert!(false);
            unimplemented!();
        }
    };
    return ConstantInfo { kind: result_kind };
}


pub fn parse_constant_infos(p: &mut dyn ParsingContext, constant_pool_count: u16) -> Vec<ConstantInfo> {
    let mut constants = Vec::with_capacity(constant_pool_count as usize);
    let mut skip_next_iter = true;
    //skip first loop iteration b/c the first element of the constant pool isn't a thing
//    let mut i = 1;
    for _ in 0..constant_pool_count {
        if skip_next_iter {
            constants.push(ConstantInfo { kind: (ConstantKind::InvalidConstant(InvalidConstant {})) });
            skip_next_iter = false;
            continue;
        }
//        dbg!(i);
        let constant_info = parse_constant_info(p);
//        i += 1;
//        dbg!(&constant_info);
        match constant_info.kind {
            ConstantKind::Long(_) | ConstantKind::Double(_) => {
                skip_next_iter = true;
            }
            _ => {}
        }
        constants.push(constant_info);
    }
    return constants;
}

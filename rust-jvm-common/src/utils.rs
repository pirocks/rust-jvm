use crate::classfile::{ConstantInfo, ConstantKind};
use crate::classfile::Classfile;
use crate::classfile::ACC_FINAL;
use crate::classfile::ACC_INTERFACE;
use std::sync::Arc;
use crate::classfile::Class;
use crate::classfile::MethodInfo;
use crate::classnames::ClassName;
use crate::classfile::Code;
use crate::classfile::ACC_NATIVE;
use crate::classfile::AttributeType;
use crate::classfile::ACC_ABSTRACT;

pub fn extract_string_from_utf8(utf8: &ConstantInfo) -> String {
    match &(utf8).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        }
        other => {
            dbg!(other);
            panic!()
        }
    }
}


pub fn has_super_class(class: &Classfile) -> bool {
    return class.super_class != 0;
}

pub fn is_interface(class: &Classfile) -> bool {
    return (class.access_flags & ACC_INTERFACE) > 0;
}

pub fn is_final(class: &Classfile) -> bool {
    return (class.access_flags & ACC_FINAL) > 0;
}






pub fn name_and_type_extractor(i: u16, class_file: &Arc<Classfile>) -> (String, String) {
    let nt;
    match &class_file.constant_pool[i as usize].kind {
        ConstantKind::NameAndType(nt_) => {
            nt = nt_;
        }
        _ => { panic!("Ths a bug.") }
    }
    let descriptor = extract_string_from_utf8(&class_file.constant_pool[nt.descriptor_index as usize]);
    let method_name = extract_string_from_utf8(&class_file.constant_pool[nt.name_index as usize]);
    return (method_name, descriptor);
}

pub fn extract_class_from_constant_pool(i: u16, classfile: &Classfile) -> &Class {
    match &classfile.constant_pool[i as usize].kind {
        ConstantKind::Class(c) => {
            return c;
        }
        _ => {
            panic!();
        }
    }
}

pub fn get_super_class_name(class: &Classfile) -> ClassName {
    let class_info = match &(class.constant_pool[class.super_class as usize]).kind {
        ConstantKind::Class(c) => {
            c
        }
        _ => { panic!() }
    };
    match &(class.constant_pool[class_info.name_index as usize]).kind {
        ConstantKind::Utf8(s) => {
            return ClassName::Str(s.string.clone());
        }
        _ => { panic!() }
    }
}

pub fn method_name(class_file: &Classfile, method_info: &MethodInfo) -> String {
    let method_name_utf8 = &class_file.constant_pool[method_info.name_index as usize];
    let method_name = extract_string_from_utf8(method_name_utf8);
    method_name
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
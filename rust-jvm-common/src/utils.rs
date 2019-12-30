use crate::classfile::{ConstantInfo, ConstantKind};
use crate::classfile::Classfile;
use crate::classfile::ACC_FINAL;
use crate::classfile::ACC_INTERFACE;
use std::sync::Arc;
use crate::classfile::Class;
use crate::classfile::MethodInfo;

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

pub fn extract_class_from_constant_pool(i: u16, classfile: &Arc<Classfile>) -> &Class {
    match &classfile.constant_pool[i as usize].kind {
        ConstantKind::Class(c) => {
            return c;
        }
        _ => {
            panic!();
        }
    }
}

pub fn get_super_class_name(class: &Classfile) -> String {
    let class_info = match &(class.constant_pool[class.super_class as usize]).kind {
        ConstantKind::Class(c) => {
            c
        }
        _ => { panic!() }
    };
    match &(class.constant_pool[class_info.name_index as usize]).kind {
        ConstantKind::Utf8(s) => {
            return s.string.clone();
        }
        _ => { panic!() }
    }
}

pub fn method_name(class_file: &Classfile, method_info: &MethodInfo) -> String {
    let method_name_utf8 = &class_file.constant_pool[method_info.name_index as usize];
    let method_name = extract_string_from_utf8(method_name_utf8);
    method_name
}

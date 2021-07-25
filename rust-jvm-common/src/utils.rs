use wtf8::Wtf8Buf;

use crate::classfile::{ACC_STATIC, ConstantInfo, ConstantKind, CPIndex, Exceptions, FieldInfo, MethodInfo};
use crate::classfile::ACC_ABSTRACT;
use crate::classfile::ACC_FINAL;
use crate::classfile::ACC_INTERFACE;
use crate::classfile::ACC_NATIVE;
use crate::classfile::AttributeType;
use crate::classfile::Class;
use crate::classfile::Classfile;
use crate::classfile::Code;
use crate::classnames::ClassName;
use crate::descriptor_parser::parse_class_name;
use crate::ptype::ReferenceType;

impl ConstantInfo {
    pub fn extract_string_from_utf8(&self) -> Wtf8Buf {
        match &(self).kind {
            ConstantKind::Utf8(s) => {
                s.string.clone()
            }
            other => {
                dbg!(other);
                panic!()
            }
        }
    }
}

impl Classfile {
    pub fn has_super_class(&self) -> bool {
        self.super_class != 0
    }

    pub fn is_interface(&self) -> bool {
        (self.access_flags & ACC_INTERFACE) > 0
    }

    pub fn is_final(&self) -> bool {
        (self.access_flags & ACC_FINAL) > 0
    }


    pub fn name_and_type_extractor(&self, i: u16) -> (String, String) {
        let nt;
        match &self.constant_pool[i as usize].kind {
            ConstantKind::NameAndType(nt_) => {
                nt = nt_;
            }
            _ => { panic!("Ths a bug.") }
        }
        let descriptor = self.constant_pool[nt.descriptor_index as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo");
        let method_name = self.constant_pool[nt.name_index as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo");
        (method_name, descriptor)
    }

    //todo this could be better used to reduce duplication
    //todo needs to correctly parse res, duplication.
    pub fn extract_class_from_constant_pool_name(&self, i: u16) -> ReferenceType {
        let name_index = match &self.constant_pool[i as usize].kind {
            ConstantKind::Class(c) => {
                c.name_index
            }
            entry => {
                dbg!(i);
                dbg!(entry);
                panic!();
            }
        };
        let name_entry = &self.constant_pool[name_index as usize];
        let string = name_entry.extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo");
        parse_class_name(string.as_str()).unwrap_ref_type()
    }

    pub fn extract_class_from_constant_pool(&self, i: u16) -> &Class {
        match &self.constant_pool[i as usize].kind {
            ConstantKind::Class(c) => {
                c
            }
            _ => {
                panic!();
            }
        }
    }

    pub fn super_class_name(&self) -> Option<ClassName> {
        let super_i = self.super_class;
        if super_i == 0 {
            return None;
        }
        let class_info = match &(self.constant_pool[super_i as usize]).kind {
            ConstantKind::Class(c) => {
                c
            }
            a => {
                dbg!(a);
                panic!()
            }
        };
        match &(self.constant_pool[class_info.name_index as usize]).kind {
            ConstantKind::Utf8(s) => {
                ClassName::Str(s.string.clone().into_string().expect("should have validated this earlier maybe todo")).into()
            }
            _ => { panic!() }
        }
    }

    pub fn lookup_method(&self, name: String, descriptor: String) -> Option<(usize, &MethodInfo)> {
        for (i, m) in self.methods.iter().enumerate() {
            // dbg!(&name);
            // dbg!(&m.method_name(self));
            // dbg!(&descriptor);
            // dbg!(&m.descriptor_str(self));
            if m.method_name(self) == name && m.descriptor_str(self) == descriptor {
                return Some((i, m));
            }
        }
        None
    }

    pub fn lookup_method_name(&self, name: &str) -> Vec<(usize, &MethodInfo)> {
        self.methods.iter().enumerate().filter(|(_i, m)| {
            m.method_name(self) == name
        }).collect()
    }

    pub fn lookup_method_name_owned(self, self_ref: &Self, name: String) -> Vec<(usize, MethodInfo)> {
        let mut res = vec![];
        for (i, m) in self.methods.into_iter().enumerate() {
            if m.method_name(self_ref) == name {
                res.push((i, m));
            }
        }
        res
    }
}

impl MethodInfo {
    pub fn method_name(&self, class_file: &Classfile) -> String {
        let method_name_utf8 = &class_file.constant_pool[self.name_index as usize];
        method_name_utf8.extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo")
    }

    pub fn code_attribute(&self) -> Option<&Code> {
        /*
        If the method is either native or abstract , and is not a class or interface
    initialization method, then its method_info structure must not have a Code attribute
    in its attributes table.
        */

        if (self.access_flags & ACC_ABSTRACT) > 0 || (self.access_flags & ACC_NATIVE) > 0 {
            return None;
        }

        for attr in self.attributes.iter() {
            if let AttributeType::Code(code) = &attr.attribute_type {
                return Some(code);
            }
        }
        panic!("Method has no code attribute, which is unusual given code is sorta the point of a method.")
    }

    pub fn exception_attribute(&self) -> Option<&Exceptions> {
        for attr in self.attributes.iter() {
            if let AttributeType::Exceptions(exceptions) = &attr.attribute_type {
                return Some(exceptions);
            }
        }
        None
    }

    pub fn descriptor_str(&self, class_file: &Classfile) -> String {
        class_file.constant_pool[self.descriptor_index as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo")
    }

    pub fn is_static(&self) -> bool {
        self.access_flags & ACC_STATIC > 0
    }

    pub fn is_abstract(&self) -> bool {
        self.access_flags & ACC_ABSTRACT > 0
    }

    pub fn is_native(&self) -> bool {
        self.access_flags & ACC_NATIVE > 0
    }
}


impl FieldInfo {
    pub fn constant_value_attribute_i(&self) -> Option<CPIndex> {
        for attr in &self.attributes {
            if let AttributeType::ConstantValue(c) = &attr.attribute_type {
                return Some(c.constant_value_index);
            }
        }
        None
    }

    pub fn name(&self, class: &Classfile) -> String {
        class.constant_pool[self.name_index as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo")
    }
}
use crate::classfile::{ConstantInfo, ConstantKind, MethodInfo, FieldInfo, CPIndex, ACC_STATIC};
use crate::classfile::Classfile;
use crate::classfile::ACC_FINAL;
use crate::classfile::ACC_INTERFACE;
use crate::classfile::Class;
use crate::classnames::ClassName;
use crate::classfile::Code;
use crate::classfile::ACC_NATIVE;
use crate::classfile::AttributeType;
use crate::classfile::ACC_ABSTRACT;

impl ConstantInfo {
    pub fn extract_string_from_utf8(&self) -> String {
        match &(self).kind {
            ConstantKind::Utf8(s) => {
                return s.string.clone();
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
        return self.super_class != 0;
    }

    pub fn is_interface(&self) -> bool {
        return (self.access_flags & ACC_INTERFACE) > 0;
    }

    pub fn is_final(&self) -> bool {
        return (self.access_flags & ACC_FINAL) > 0;
    }


    pub fn name_and_type_extractor(&self, i: u16) -> (String, String) {
        let nt;
        match &self.constant_pool[i as usize].kind {
            ConstantKind::NameAndType(nt_) => {
                nt = nt_;
            }
            _ => { panic!("Ths a bug.") }
        }
        let descriptor = self.constant_pool[nt.descriptor_index as usize].extract_string_from_utf8();
        let method_name = self.constant_pool[nt.name_index as usize].extract_string_from_utf8();
        return (method_name, descriptor);
    }

    pub fn extract_class_from_constant_pool(&self, i: u16) -> &Class {
        match &self.constant_pool[i as usize].kind {
            ConstantKind::Class(c) => {
                return c;
            }
            _ => {
                panic!();
            }
        }
    }

    pub fn super_class_name(&self) -> ClassName {
        let class_info = match &(self.constant_pool[self.super_class as usize]).kind {
            ConstantKind::Class(c) => {
                c
            }
            _ => { panic!() }
        };
        match &(self.constant_pool[class_info.name_index as usize]).kind {
            ConstantKind::Utf8(s) => {
                return ClassName::Str(s.string.clone());
            }
            _ => { panic!() }
        }
    }

    pub fn lookup_method(&self, name : String, descriptor : String) -> Option<(usize,&MethodInfo)>{
        for (i,m) in self.methods.iter().enumerate() {
            if m.method_name(self) == name && m.descriptor_str(self)  == descriptor {
                return Some((i,m))
            }
        }
        None
    }



    pub fn lookup_method_name(&self, name : String) -> Vec<(usize, &MethodInfo)> {
        self.methods.iter().enumerate().filter(|(_i,m)| {
            m.method_name(self) == name
        }).collect()
    }
}

impl MethodInfo {
    pub fn method_name(&self, class_file: &Classfile) -> String {
        let method_name_utf8 = &class_file.constant_pool[self.name_index as usize];
        let method_name = method_name_utf8.extract_string_from_utf8();
        method_name
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
            match &attr.attribute_type {
                AttributeType::Code(code) => {
                    return Some(code);
                }
                _ => {}
            }
        }
        panic!("Method has no code attribute, which is unusual given code is sorta the point of a method.")
    }

    pub fn descriptor_str(&self, class_file: &Classfile) -> String {
        class_file.constant_pool[self.descriptor_index as usize].extract_string_from_utf8()
    }

    pub fn is_static(&self) -> bool{
        self.access_flags & ACC_STATIC > 0
    }
    //todo need a find method function
}


impl FieldInfo {
    pub fn constant_value_attribute_i(&self) -> Option<CPIndex> {
        for attr in &self.attributes {
            match &attr.attribute_type {
                AttributeType::ConstantValue(c) => {
                    return Some(c.constant_value_index);
                }
                _ => {}
            }
        }
        None
    }
}
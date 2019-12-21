use std::sync::{Weak, Arc};
use crate::classfile::{Classfile, ConstantKind};
use crate::utils::extract_string_from_utf8;

#[derive(Debug)]
pub struct NameReference{
    pub class_file: Weak<Classfile>,
    pub index : u16,
}

impl Eq for NameReference {}

//pub fn rc_ptr_eq<T: ?Sized>(this: &Rc<T>, other: &Rc<T>) -> bool {
//    unsafe  {
//        let this_ptr: *const T = &*this;
//        let other_ptr: *const T = &*other;
//        this_ptr == other_ptr
//    }
//}

impl PartialEq for NameReference{
    fn eq(&self, other: &NameReference) -> bool{
        self.class_file.ptr_eq(&other.class_file) && self.index == other.index
    }
}

#[derive(Debug)]
#[derive(Eq)]
pub enum ClassName {
    Ref(NameReference),
    Str(String)
}

impl PartialEq for ClassName {
    fn eq(&self, other: &ClassName) -> bool{
        match self{
            ClassName::Ref(r1) => match other {
                ClassName::Ref(r2) => {
                    //todo how is equality for classfiles defined here?
                    r1.class_file.ptr_eq(&r2.class_file) && r1.index == r2.index
                }
                ClassName::Str(s) => {
                    &get_referred_name(self) == s
                }
            },
            ClassName::Str(s1) => match other {
                ClassName::Str(s2) => {
                    s1 == s2
                }
                ClassName::Ref(_) => {
                    unimplemented!()
                }
            },
        }
    }
}

impl std::clone::Clone for ClassName {
    fn clone(&self) -> Self {
        match self{
            ClassName::Ref(r) => {
                ClassName::Ref(NameReference {
                    index:  r.index,
                    class_file: r.class_file.clone()
                })
            },
            ClassName::Str(s) => {
                ClassName::Str(s.clone())//todo fix
            },
        }
    }
}

pub fn get_referred_name(ref_ : &ClassName) -> String{
    match ref_{
        ClassName::Ref(r) => {
            let upgraded_class_ref = match r.class_file.upgrade() {
                None => {panic!()},
                Some(s) => s
            };
            return extract_string_from_utf8(&upgraded_class_ref.constant_pool[r.index as usize])
        },
        ClassName::Str(s) => {s.clone()},//todo this clone may be expensive, ditch?
    }
}


pub fn class_name_legacy(class: &Classfile) -> String {
    let class_info_entry = match &(class.constant_pool[class.this_class as usize]).kind {
        ConstantKind::Class(c) => { c }
        _ => { panic!() }
    };
    return extract_string_from_utf8(&class.constant_pool[class_info_entry.name_index as usize]);
}

pub fn class_name(class: &Arc<Classfile>) -> ClassName {
    let class_info_entry = match &(class.constant_pool[class.this_class as usize]).kind {
        ConstantKind::Class(c) => { c }
        _ => { panic!() }
    };

    return ClassName::Ref(NameReference {
        class_file:Arc::downgrade(&class),
        index: class_info_entry.name_index
    });
}
use std::sync::{Weak, Arc};
use crate::classfile::{Classfile, ConstantKind};
use std::fmt::Formatter;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use crate::string_pool::StringPoolEntry;
use std::ops::Deref;

#[derive(Debug)]
pub struct NameReference {
    pub class_file: Weak<Classfile>,
    pub index: u16,
}

impl Eq for NameReference {}

impl PartialEq for NameReference {
    fn eq(&self, other: &NameReference) -> bool {
        self.class_file.ptr_eq(&other.class_file) && self.index == other.index
    }
}

//#[derive(Debug)]
#[derive(Eq)]
//#[derive(Hash)]
pub enum ClassName {
//    Ref(NameReference),
    Str(String),//todo deprecate
    SharedStr(Arc<StringPoolEntry>)
}

impl ClassName {
    pub fn new(str_: &str) -> Self {
        ClassName::Str(str_.to_string())
    }

    pub fn object() -> Self {
        ClassName::new("java/lang/Object")
    }

    pub fn class() -> Self {
        ClassName::new("java/lang/Class")
    }

    pub fn string() -> Self {
        ClassName::new("java/lang/String")
    }

    pub fn throwable() -> Self {
        ClassName::new("java/lang/Throwable")
    }
}

impl Hash for ClassName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.get_referred_name().as_bytes())
    }
}

impl PartialEq for ClassName {
    fn eq(&self, other: &ClassName) -> bool {
        self.get_referred_name() == other.get_referred_name()
    }
}

impl std::clone::Clone for ClassName {
    fn clone(&self) -> Self {
        match self {
            /*ClassName::Ref(r) => {
                ClassName::Ref(NameReference {
                    index: r.index,
                    class_file: r.class_file.clone(),
                })
            }*/
            ClassName::Str(s) => {
                ClassName::Str(s.clone())//todo fix
            }
            ClassName::SharedStr(s) => ClassName::SharedStr(s.clone())
        }
    }
}


impl std::fmt::Debug for ClassName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_referred_name())
    }
}

impl ClassName {
    pub fn get_referred_name(&self) -> &String {
        match self {
            ClassName::Str(s) => { s }//todo this clone may be expensive, ditch?
            ClassName::SharedStr(s) => { s.deref() }
        }
    }
}


pub fn class_name(class: &Arc<Classfile>) -> ClassName {
    let class_info_entry = match &(class.constant_pool[class.this_class as usize]).kind {
        ConstantKind::Class(c) => { c }
        _ => { panic!() }
    };

    unimplemented!()
    /*return ClassName::Ref(NameReference {
        class_file: Arc::downgrade(&class),
        index: class_info_entry.name_index,
    });*/
}

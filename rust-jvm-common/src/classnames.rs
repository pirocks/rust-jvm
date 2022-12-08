use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Weak;

use crate::classfile::Classfile;
use crate::ptype::ReferenceType;

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

#[derive(Eq)]
pub enum ClassName {
    Str(String),
}

impl ClassName {
    pub fn is_raw(&self) -> bool {
        &Self::raw_byte() == self || &Self::raw_char() == self || &Self::raw_double() == self || &Self::raw_float() == self || &Self::raw_int() == self || &Self::raw_long() == self || &Self::raw_short() == self || &Self::raw_boolean() == self || &Self::raw_void() == self
    }

    pub fn raw_byte() -> Self {
        ClassName::Str("byte".to_string())
    }
    pub fn raw_char() -> Self {
        ClassName::Str("char".to_string())
    }
    pub fn raw_double() -> Self {
        ClassName::Str("double".to_string())
    }
    pub fn raw_float() -> Self {
        ClassName::Str("float".to_string())
    }
    pub fn raw_int() -> Self {
        ClassName::Str("int".to_string())
    }
    pub fn raw_long() -> Self {
        ClassName::Str("long".to_string())
    }
    pub fn raw_short() -> Self {
        ClassName::Str("short".to_string())
    }
    pub fn raw_boolean() -> Self {
        ClassName::Str("boolean".to_string())
    }
    pub fn raw_void() -> Self {
        ClassName::Str("void".to_string())
    }

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

    pub fn float() -> Self {
        Self::new("java/lang/Float")
    }

    pub fn double() -> Self {
        Self::new("java/lang/Double")
    }
    pub fn int() -> Self {
        Self::new("java/lang/Integer")
    }
    pub fn long() -> Self {
        Self::new("java/lang/Long")
    }

    pub fn character() -> Self {
        Self::new("java/lang/Character")
    }

    pub fn boolean() -> Self {
        Self::new("java/lang/Boolean")
    }

    pub fn byte() -> Self {
        Self::new("java/lang/Byte")
    }

    pub fn short() -> Self {
        Self::new("java/lang/Short")
    }

    pub fn void() -> Self {
        Self::new("java/lang/Void")
    }

    pub fn method_type() -> Self {
        Self::new("java/lang/invoke/MethodType")
    }

    pub fn method_type_form() -> Self {
        Self::new("java/lang/invoke/MethodTypeForm")
    }

    pub fn method_handle() -> Self {
        Self::new("java/lang/invoke/MethodHandle")
    }

    pub fn method_handles() -> Self {
        Self::new("java/lang/invoke/MethodHandles")
    }

    pub fn lookup() -> Self {
        Self::new("java/lang/invoke/MethodHandles$Lookup")
    }

    pub fn direct_method_handle() -> Self {
        Self::new("java/lang/invoke/DirectMethodHandle")
    }

    pub fn member_name() -> Self {
        Self::new("java/lang/invoke/MemberName")
    }

    pub fn method() -> Self {
        Self::new("java/lang/reflect/Method")
    }

    pub fn system() -> Self {
        Self::new("java/lang/System")
    }

    pub fn serializable() -> Self {
        Self::new("java/io/Serializable")
    }

    pub fn cloneable() -> Self {
        Self::new("java/lang/Cloneable")
    }

    pub fn unsafe_() -> Self {
        Self::new("sun/misc/Unsafe")
    }

    pub fn field() -> Self {
        Self::new("java/lang/reflect/Field")
    }

    pub fn properties() -> Self {
        Self::new("java/util/Properties")
    }

    pub fn thread() -> Self {
        Self::new("java/lang/Thread")
    }

    pub fn thread_group() -> Self {
        Self::new("java/lang/ThreadGroup")
    }

    pub fn constructor() -> Self {
        Self::new("java/lang/reflect/Constructor")
    }

    pub fn classloader() -> Self {
        Self::new("java/lang/ClassLoader")
    }
    pub fn stack_trace_element() -> Self {
        Self::new("java/lang/StackTraceElement")
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

impl Clone for ClassName {
    fn clone(&self) -> Self {
        match self {
            ClassName::Str(s) => {
                ClassName::Str(s.clone()) //todo fix
            }
        }
    }
}

impl fmt::Debug for ClassName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_referred_name())
    }
}

impl ClassName {
    pub fn get_referred_name(&self) -> &String {
        match self {
            ClassName::Str(s) => s,
        }
    }
}

pub fn class_name(class: &Classfile) -> ClassName {
    match class.extract_class_from_constant_pool_name(class.this_class) {
        ReferenceType::Class(c) => c,
        ReferenceType::Array(_) => todo!(),
    }
}

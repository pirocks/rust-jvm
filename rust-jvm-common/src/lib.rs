#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(arbitrary_enum_discriminant)]
#![allow(unreachable_code)]
#![allow(dead_code)]

use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

pub mod classfile;
pub mod classnames;
pub mod compressed_classfile;
pub mod descriptor_parser;
pub mod loading;
pub mod ptype;
pub mod runtime_type;
pub mod string_pool;
pub mod test_utils;
pub mod utils;
pub mod vtype;
pub mod cpdtype_table;
pub mod opaque_id_table;
pub mod method_shape;
pub mod global_consts;


pub const EXPECTED_CLASSFILE_MAGIC: u32 = 0xCAFEBABE;

pub type JavaThreadId = i64;
pub type MethodTableIndex = usize;
pub type MethodId = MethodTableIndex;

pub type FieldTableIndex = usize;
pub type FieldId = usize;


#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub struct ByteCodeOffset(pub u16);//todo unify this with bytecode offset

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ByteCodeIndex(pub u16);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct InheritanceMethodID(pub u64);

pub type MethodI = u16;

#[deprecated = "native java value causes word tearing and is therefore is considered harmful"]
#[derive(Copy, Clone)]
pub union NativeJavaValue<'gc> {
    pub byte: i8,
    pub boolean: u8,
    pub short: i16,
    pub char: u16,
    pub int: i32,
    pub long: i64,
    pub float: f32,
    pub double: f64,
    pub object: *mut c_void,
    phantom_data: PhantomData<&'gc ()>,
    pub as_u64: u64,
}

impl Debug for NativeJavaValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "NativeJavaValue({:?})", self.object) }
    }
}
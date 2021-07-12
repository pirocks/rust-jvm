#![feature(box_syntax)]
#![allow(unreachable_code)]
#![allow(dead_code)]

pub mod classfile;
pub mod ptype;
pub mod classnames;
pub mod utils;
pub mod test_utils;
pub mod string_pool;
pub mod descriptor_parser;
pub mod vtype;
pub mod loading;
pub mod compressed_classfile;
pub mod runtime_type;

pub const EXPECTED_CLASSFILE_MAGIC: u32 = 0xCAFEBABE;

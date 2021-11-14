#![feature(box_syntax)]
#![feature(box_patterns)]
#![allow(unreachable_code)]
#![allow(dead_code)]

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

pub const EXPECTED_CLASSFILE_MAGIC: u32 = 0xCAFEBABE;

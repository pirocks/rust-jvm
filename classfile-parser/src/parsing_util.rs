use std::fs::File;
use std::io::prelude::*;
use rust_jvm_common::classfile::ConstantInfo;
use rust_jvm_common::loading::Loader;
use std::sync::Arc;

pub struct ParsingContext<'l> {
    pub f: File,
    pub constant_pool : &'l Vec<ConstantInfo>,
    pub loader: Arc<Loader>
}

const IO_ERROR_MSG: &str = "Some sort of error in reading a classfile";

pub fn read8(p: &mut ParsingContext) -> u8 {
    let mut buffer = [0; 1];
    let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
    assert_eq!(bytes_read, 1);
    return buffer[0];
}

pub fn read16(p: &mut ParsingContext) -> u16 {
    let mut buffer = [0; 2];
    let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
    assert_eq!(bytes_read, 2);
    return u16::from_be(((buffer[1] as u16) << 8) | buffer[0] as u16);
}

pub fn read32(p: &mut ParsingContext) -> u32 {
    let mut buffer = [0; 4];
    let bytes_read = p.f.read(&mut buffer).expect(IO_ERROR_MSG);
    assert_eq!(bytes_read, 4);
    return u32::from_be(((buffer[0] as u32) << 0) +
        ((buffer[1] as u32) << 8) +
        ((buffer[2] as u32) << 16) +
        ((buffer[3] as u32) << 24));
}

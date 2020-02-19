use std::io::prelude::*;
use rust_jvm_common::classfile::ConstantInfo;

pub trait ParsingContext {
    fn read8(&mut self) -> u8;
    fn read16(&mut self) -> u16;
    fn read32(&mut self) -> u32;
    fn set_constant_pool(&mut self, constant_pool: Vec<ConstantInfo>);
    fn constant_pool(self) -> Vec<ConstantInfo>;
    fn constant_pool_borrow(&self) -> &Vec<ConstantInfo>;
}

pub(crate) struct FileParsingContext<'l> {
    pub(crate) read: &'l mut dyn Read,
    pub(crate) constant_pool: Option<Vec<ConstantInfo>>,
}

const IO_ERROR_MSG: &str = "Some sort of error in reading a classfile";

impl ParsingContext for FileParsingContext<'_> {
    fn read8(&mut self) -> u8 {
        let mut buffer = [0; 1];
        let bytes_read = self.read.read(&mut buffer).expect(IO_ERROR_MSG);
        assert_eq!(bytes_read, 1);
        return buffer[0];
    }

    fn read16(&mut self) -> u16 {
        let mut buffer = [0; 2];
        let bytes_read = self.read.read(&mut buffer).expect(IO_ERROR_MSG);
        assert_eq!(bytes_read, 2);
        return u16::from_be(((buffer[1] as u16) << 8) | buffer[0] as u16);
    }

    fn read32(&mut self) -> u32 {
        let mut buffer = [0; 4];
        let bytes_read = self.read.read(&mut buffer).expect(IO_ERROR_MSG);
        assert_eq!(bytes_read, 4);
        return u32::from_be(((buffer[0] as u32) << 0) +
            ((buffer[1] as u32) << 8) +
            ((buffer[2] as u32) << 16) +
            ((buffer[3] as u32) << 24));
    }

    fn set_constant_pool(&mut self, constant_pool: Vec<ConstantInfo>) {
        self.constant_pool = Some(constant_pool);
    }

    fn constant_pool(self) -> Vec<ConstantInfo> {
        self.constant_pool.unwrap()
    }

    fn constant_pool_borrow(&self) -> &Vec<ConstantInfo> {
        self.constant_pool.as_ref().unwrap()
    }
}


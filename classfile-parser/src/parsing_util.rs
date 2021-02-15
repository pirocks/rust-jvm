use std::io::prelude::*;

use rust_jvm_common::classfile::ConstantInfo;

use crate::ClassfileParsingError;

pub trait ParsingContext {
    fn read8(&mut self) -> Result<u8, ClassfileParsingError>;
    fn read16(&mut self) -> Result<u16, ClassfileParsingError>;
    fn read32(&mut self) -> Result<u32, ClassfileParsingError>;
    fn set_constant_pool(&mut self, constant_pool: Vec<ConstantInfo>);
    fn constant_pool(self) -> Vec<ConstantInfo>;
    fn constant_pool_borrow(&self) -> &Vec<ConstantInfo>;
}

pub(crate) struct ReadParsingContext<'l> {
    pub(crate) read: &'l mut dyn Read,
    pub(crate) constant_pool: Option<Vec<ConstantInfo>>,
}

impl ParsingContext for ReadParsingContext<'_> {
    fn read8(&mut self) -> Result<u8, ClassfileParsingError> {
        let mut buffer = [0; 1];
        let bytes_read = self.read.read(&mut buffer).map_err(|_| ClassfileParsingError::EOF)?;
        assert_eq!(bytes_read, 1);
        Ok(buffer[0])
    }

    fn read16(&mut self) -> Result<u16, ClassfileParsingError> {
        let mut buffer = [0; 2];
        buffer[0] = self.read8()?;
        buffer[1] = self.read8()?;
        Ok(u16::from_be(((buffer[1] as u16) << 8) | buffer[0] as u16))
    }

    fn read32(&mut self) -> Result<u32, ClassfileParsingError> {
        let mut buffer = [0; 4];
        buffer[0] = self.read8()?;
        buffer[1] = self.read8()?;
        buffer[2] = self.read8()?;
        buffer[3] = self.read8()?;

        Ok(u32::from_be((buffer[0] as u32) +
            ((buffer[1] as u32) << 8) +
            ((buffer[2] as u32) << 16) +
            ((buffer[3] as u32) << 24)))
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


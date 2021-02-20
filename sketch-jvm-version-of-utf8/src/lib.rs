pub struct JVMString {
    buf: Vec<u8>
}

pub struct PossiblyJVMString {
    buf: Vec<u8>
}

pub enum ValidationError {}

impl PossiblyJVMString {
    fn validate(self) -> Result<JVMString, ValidationError> {
        String::from_utf8(self.to_regular_utf8())?;
        Ok(JVMString { buf: self.buf })
    }

    fn to_regular_utf8(&self) -> Vec<u8> {
        let mut res = vec![];
        let mut x = None;
        let mut y = None;
        let mut z = None;
        let mut u = None;
        let mut v = None;
        let mut w = None;
        for current_byte in self.buf {
            todo!()
        }
        res
    }
}
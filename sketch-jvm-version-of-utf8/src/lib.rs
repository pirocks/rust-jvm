use std::string::FromUtf8Error;

use itertools::Itertools;
use wtf8::{CodePoint, Wtf8Buf};

pub mod wtf8_pool;
#[cfg(test)]
pub mod test;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JVMString {
    pub buf: Vec<u8>,
}

//todo if issues with this go dumpster diving in history there where some issues merging to master.

impl JVMString {
    pub fn from_regular_string(str: &str) -> Self {
        let buf = str
            .chars()
            .flat_map(|char_| {
                let char_ = char_ as u32;
                if (0x1..=0x7F).contains(&char_) {
                    vec![char_ as u8]
                } else if (0x80..=0x7FF).contains(&char_) {
                    let x = 0b1100_0000 | (0b0001_1111 & ((char_ >> 6) as u8));
                    let y = 0b1000_0000 | (0b0011_1111 & (char_ as u8));
                    vec![x, y]
                } else if (0x800..=0xFFFF).contains(&char_) {
                    let x = 0b1110_0000 | (0b0000_1111 & ((char_ >> 12) as u8));
                    let y = 0b1000_0000 | (0b0011_1111 & ((char_ >> 6) as u8));
                    let z = 0b1000_0000 | (0b0011_1111 & (char_ as u8));
                    vec![x, y, z]
                } else {
                    assert!(dbg!(char_) > 0xFFFF);
                    let u = 0b1110_1101;
                    let v = 0b1010_0000 | (0b1111_0000 & ((char_ >> 16) as u8 - 1));
                    let w = 0b1000_0000 | (0b1100_0000 & ((char_ >> 10) as u8));
                    let x = 0b1110_1101;
                    let y = 0b1011_0000 | (0b0000_1111 & ((char_ >> 6) as u8));
                    let z = 0b1000_0000 | (0b0011_1111 & (char_ as u8));
                    vec![u, v, w, x, y, z]
                }
            })
            .collect::<Vec<_>>();
        Self { buf }
    }

    pub fn to_string_validated(&self) -> String {
        String::from_utf8(self.buf.clone()).unwrap()
    }

    pub fn to_wtf8(&self) -> Wtf8Buf {
        PossiblyJVMString { buf: self.buf.clone() }.to_regular_utf8(true).unwrap().unwrap_wtf8()
    }

    pub fn to_string(&self) -> Result<String, ValidationError> {
        let buf = PossiblyJVMString { buf: self.buf.clone() }.to_regular_utf8(false)?;
        Ok(buf.unwrap_utf8())
    }
}

pub struct PossiblyJVMString {
    buf: Vec<u8>,
}

#[derive(Debug)]
pub enum ValidationError {
    UnexpectedEndOfString,
    UnexpectedBits,
    UTfError(FromUtf8Error),
    InvalidCodePoint,
}

impl From<FromUtf8Error> for ValidationError {
    fn from(err: FromUtf8Error) -> Self {
        Self::UTfError(err)
    }
}

impl PossiblyJVMString {
    pub fn new(buf: Vec<u8>) -> Self {
        Self { buf }
    }

    pub fn validate(self, allow_wtf8: bool) -> Result<JVMString, ValidationError> {
        self.to_regular_utf8(allow_wtf8)?;
        Ok(JVMString { buf: self.buf })
    }

    fn to_regular_utf8(&self, is_wtf8: bool) -> Result<Utf8OrWtf8, ValidationError> {
        assert!(is_wtf8);//this parsing only parses to wtf8
        let mut res = if is_wtf8 { Utf8OrWtf8::new_wtf() } else { Utf8OrWtf8::new_utf() };
        let mut buf_iter = self.buf.iter().multipeek();
        loop {
            let x = match buf_iter.next() {
                None => break,
                Some(x) => *x,
            };
            if (x >> 7) == 0 {
                let codepoint = x as u32;
                res.push_codepoint(codepoint)?;
            } else {
                let y = buf_iter.next().cloned().ok_or::<ValidationError>(ValidationError::UnexpectedEndOfString)?;
                if (x >> 5) == 0b110 && (y >> 6) == 0b10 {
                    let x_u16 = x as u16;
                    let y_u16 = y as u16;
                    let codepoint = ((x_u16 & 0x1f) << 6) + (y_u16 & 0x3f);
                    res.push_codepoint(codepoint as u32)?;
                } else if (x >> 4) == 0b1110 {
                    if (y >> 6) != 0b10 {
                        dbg!(y);
                        eprintln!("{:b}", y);
                        panic!()
                        // return Err(ValidationError::UnexpectedBits);
                    }
                    let z = buf_iter.next().cloned().ok_or::<ValidationError>(ValidationError::UnexpectedEndOfString)?;
                    if (z >> 6) != 0b10 {
                        dbg!(z);
                        panic!()
                        // return Err(ValidationError::UnexpectedBits);
                    }
                    let x_u16 = x as u16;
                    let y_u16 = y as u16;
                    let z_u16 = z as u16;
                    let codepoint = ((x_u16 & 0xf) << 12) + ((y_u16 & 0x3f) << 6) + (z_u16 & 0x3f);
                    res.push_codepoint(codepoint as u32)?;
                }
            }
        }
        Ok(res)
    }
}

pub enum Utf8OrWtf8 {
    Wtf(Wtf8Buf),
    Utf(String),
}

impl Utf8OrWtf8 {
    pub fn new_utf() -> Self {
        Self::Utf(String::new())
    }
    pub fn new_wtf() -> Self {
        Self::Wtf(Wtf8Buf::new())
    }
    pub fn push_codepoint(&mut self, to_push: u32) -> Result<(), ValidationError> {
        match self {
            Utf8OrWtf8::Wtf(wtf) => wtf.push(CodePoint::from_u32(to_push as u32).ok_or_else(|| panic!("panic for now for debugging todo")).unwrap()),
            Utf8OrWtf8::Utf(utf) => utf.push(char::from_u32(to_push as u32).ok_or(ValidationError::InvalidCodePoint)?),
        }
        Ok(())
    }

    pub fn unwrap_wtf8(self) -> Wtf8Buf {
        match self {
            Utf8OrWtf8::Wtf(res) => res,
            Utf8OrWtf8::Utf(str) => Wtf8Buf::from_string(str),
        }
    }

    pub fn unwrap_utf8(self) -> String {
        match self {
            Utf8OrWtf8::Wtf(_) => panic!(),
            Utf8OrWtf8::Utf(str) => str,
        }
    }
}

pub struct InvalidCodepoint {}

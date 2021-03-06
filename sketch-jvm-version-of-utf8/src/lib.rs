use std::char::from_u32;
use std::string::FromUtf8Error;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JVMString {
    pub buf: Vec<u8>
}

impl JVMString {
    pub fn from_regular_string(str: &str) -> Self {
        let buf = str.chars().flat_map(|char_| {
            let char_ = char_ as u32;
            if char_ >= 0x1 && char_ <= 0x7F {
                vec![char_ as u8]
            } else if char_ >= 0x80 && char_ <= 0x7FF {
                let x = 0b1100_0000 | (0b0001_1111 & ((char_ >> 6) as u8));
                let y = 0b1000_0000 | (0b0011_1111 & (char_ as u8));
                vec![x, y]
            } else if char_ >= 0x800 && char_ <= 0xFFFF {
                let x = 0b1110_0000 | (0b0000_1111 & ((char_ >> 12) as u8));
                let y = 0b1000_0000 | (0b0011_1111 & ((char_ >> 6) as u8));
                let z = 0b1000_0000 | (0b0011_1111 & (char_ as u8));
                vec![x, y, z]
            } else {
                assert!(char_ > 0xFFFF);
                let u = 0b1110_1101;
                let v = 0b1010_0000 | (0b1111_0000 & ((char_ >> 16) as u8 - 1));
                let w = 0b1000_0000 | (0b1100_0000 & ((char_ >> 10) as u8));
                let x = 0b1110_1101;
                let y = 0b1011_0000 | (0b0000_1111 & ((char_ >> 6) as u8));
                let z = 0b1000_0000 | (0b0011_1111 & (char_ as u8));
                vec![u, v, w, x, y, z]
            }
        }).collect::<Vec<_>>();
        Self {
            buf
        }
    }

    pub fn to_string_validated(&self) -> String {
        String::from_utf8(self.buf.clone()).unwrap()
    }

    pub fn to_string(&self) -> Result<String, ValidationError> {
        let buf = PossiblyJVMString { buf: self.buf.clone() }.to_regular_utf8()?;
        Ok(String::from_utf8(buf)?)
    }
}

pub struct PossiblyJVMString {
    buf: Vec<u8>
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
        Self {
            buf
        }
    }

    pub fn validate(self) -> Result<JVMString, ValidationError> {
        String::from_utf8(self.to_regular_utf8()?)?;
        Ok(JVMString { buf: self.buf })
    }

    pub fn to_regular_utf8(&self) -> Result<Vec<u8>, ValidationError> {
        let mut res = String::new();
        let mut buf_iter = self.buf.iter();
        loop {
            let x = match buf_iter.next() {
                None => break,
                Some(x) => *x,
            };
            if (x >> 7) == 0 {
                let codepoint = x as u32;
                res.push(from_u32(codepoint).ok_or(ValidationError::InvalidCodePoint)?);
            } else if (x >> 5) == 0b110 {
                let y = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if (y >> 6) != 0b10 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let x_u16 = x as u16;
                let y_u16 = y as u16;
                let codepoint = (x_u16 & 0x1f << 6) + (y_u16 & 0x3f);
                res.push(from_u32(codepoint as u32).ok_or(ValidationError::InvalidCodePoint)?);
            } else if x != 0b11101101 {
                let y = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if (y >> 6) != 0b10 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let z = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if (z >> 6) != 0b10 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let x_u16 = x as u16;
                let y_u16 = y as u16;
                let z_u16 = z as u16;
                let codepoint = ((x_u16 & 0xf) << 12) + ((y_u16 & 0x3f) << 6) + (z_u16 & 0x3f);
                res.push(from_u32(codepoint as u32).ok_or(ValidationError::InvalidCodePoint)?);
            } else {
                let _u = x;
                let v = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if (v >> 4) != 0b1010 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let w = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if (w >> 6) != 0b10 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let x = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if x != 0b11101101 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let y = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if (y >> 4) != 0b1010 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let z = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if (z >> 6) != 0b10 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let y_u32 = y as u32;
                let z_u32 = z as u32;
                let v_u32 = v as u32;
                let w_u32 = w as u32;
                let codepoint = 0x10000 + ((v_u32 & 0x0f) << 16) + ((w_u32 & 0x3f) << 10) + ((y_u32 & 0x0f) << 6) + (z_u32 & 0x3f);
                res.push(from_u32(codepoint).ok_or(ValidationError::InvalidCodePoint)?);
            }
        }
        Ok(res.into_bytes())
    }
}
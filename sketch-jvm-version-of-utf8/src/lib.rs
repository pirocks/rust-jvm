pub struct JVMString {
    buf: Vec<u8>
}

pub struct PossiblyJVMString {
    buf: Vec<u8>
}

pub enum ValidationError {
    UnexpectedEndOfString,
    UnexpectedBits,
}

impl PossiblyJVMString {
    fn validate(self) -> Result<JVMString, ValidationError> {
        String::from_utf8(self.to_regular_utf8()?)?;
        Ok(JVMString { buf: self.buf })
    }

    fn to_regular_utf8(&self) -> Result<Vec<u8>, ValidationError> {
        let mut res = String::new();
        let mut buf_iter = self.buf.iter();
        loop {
            let x = match buf_iter.next() {
                None => break,
                Some(x) => *x,
            };
            if (x >> 7) == 0 {
                let codepoint = x as u16;
                res.push(codepoint as char);
            } else if (x >> 5) == 0b110 {
                let y = buf_iter.next().cloned().ok_or(ValidationError::UnexpectedEndOfString)?;
                if (y >> 6) != 0b10 {
                    return Err(ValidationError::UnexpectedBits);
                }
                let x_u16 = x as u16;
                let y_u16 = y as u16;
                let codepoint = (x_u16 & 0x1f << 6) + (y_u16 & 0x3f);
                res.push(codepoint as char);
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
                res.push(codepoint as char);
            } else {
                let u = x;
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
                res.push(codepoint as char);
            }
        }
        Ok(res.into_bytes())
    }
}
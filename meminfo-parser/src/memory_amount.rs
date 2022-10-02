use std::str::FromStr;

use crate::ParseError;

#[derive(Debug)]
pub enum MemoryAmount {
    KiloBytes(usize)
}

impl PartialEq for MemoryAmount {
    fn eq(&self, other: &Self) -> bool {
        match self {
            MemoryAmount::KiloBytes(self_kb) => {
                match other {
                    MemoryAmount::KiloBytes(other_kb) => {
                        self_kb == other_kb
                    }
                }
            }
        }
    }
}

impl MemoryAmount {
    pub fn parse(from: impl AsRef<str>) -> Result<MemoryAmount, ParseError> {
        let trimmed = from.as_ref().trim();
        if trimmed.ends_with("kB") {
            let without_units = trimmed.strip_suffix("kB").unwrap();
            let number_str = without_units.trim();
            return Ok(MemoryAmount::KiloBytes(usize::from_str(number_str)?));
        }
        Err(ParseError::InvalidMemoryAmount { line: trimmed.to_string() })
    }

    pub fn kilobytes(&self) -> usize {
        match self {
            MemoryAmount::KiloBytes(kb) => {
                *kb
            }
        }
    }
}


use std::cmp::Ordering;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug)]
pub enum MemoryAmountParseError {
    ParseIntError {
        err: ParseIntError,
        line: String,
    },
    InvalidMemoryAmount {
        line: String
    },
}


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

impl PartialOrd for MemoryAmount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.bytes().partial_cmp(&other.bytes())
    }
}

impl MemoryAmount {
    pub fn parse(from: impl AsRef<str>) -> Result<MemoryAmount, MemoryAmountParseError> {
        let trimmed = from.as_ref().trim();
        if trimmed.ends_with("kB") {
            let without_units = trimmed.strip_suffix("kB").unwrap();
            let number_str = without_units.trim();
            let parsed_number = match usize::from_str(number_str) {
                Ok(parsed_number) => parsed_number,
                Err(err) => {
                    return Err(MemoryAmountParseError::ParseIntError { err, line: number_str.to_string() });
                }
            };
            return Ok(MemoryAmount::KiloBytes(parsed_number));
        }
        Err(MemoryAmountParseError::InvalidMemoryAmount { line: trimmed.to_string() })
    }

    pub fn kilobytes(&self) -> usize {
        match self {
            MemoryAmount::KiloBytes(kb) => {
                *kb
            }
        }
    }

    pub fn bytes(&self) -> usize {
        match self {
            MemoryAmount::KiloBytes(kb) => *kb * 1024
        }
    }
}


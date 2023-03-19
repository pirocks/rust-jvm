//https://docs.kernel.org/accounting/psi.html
//https://github.com/uprt/memory-pressure

use std::io;
use std::num::ParseFloatError;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub struct PressureStallLine {
    pub avg10: f64,
    pub avg60: f64,
    pub avg300: f64,
    pub total: f64,
}

impl PressureStallLine {
    fn parse_entry(entry: &str, entry_prefix: &str) -> Result<Option<f64>, PSIParseError> {
        if entry.starts_with(entry_prefix) {
            return Ok(Some(f64::from_str(&entry[entry_prefix.len()..])?));
        }
        Ok(None)
    }


    pub fn parse(line: impl AsRef<str>) -> Result<Self, PSIParseError> {
        let line = line.as_ref();
        let mut avg10 = None;
        let mut avg60 = None;
        let mut avg300 = None;
        let mut total = None;
        for entry in line.split(" ") {
            if let Some(avg10_inner) = Self::parse_entry(entry, "avg10=")? {
                avg10 = Some(avg10_inner);
            }
            if let Some(avg60_inner) = Self::parse_entry(entry, "avg10=")? {
                avg60 = Some(avg60_inner);
            }
            if let Some(avg300_inner) = Self::parse_entry(entry, "avg10=")? {
                avg300 = Some(avg300_inner);
            }
            if let Some(total_inner) = Self::parse_entry(entry, "avg10=")? {
                total = Some(total_inner);
            }
        }
        Ok(PressureStallLine {
            avg10: avg10.ok_or(PSIParseError::MissingLineEntry)?,
            avg60: avg60.ok_or(PSIParseError::MissingLineEntry)?,
            avg300: avg300.ok_or(PSIParseError::MissingLineEntry)?,
            total: total.ok_or(PSIParseError::MissingLineEntry)?,
        })
    }
}


#[derive(Debug)]
pub struct PressureStallInformation {
    pub some: PressureStallLine,
    pub full: PressureStallLine,
}

#[derive(Debug)]
pub enum PSIParseError {
    TwoSomeLines,
    TwoFullLines,
    MissingSomeLine,
    MissingFullLine,
    MissingLineEntry,
    UnexpectedLine(String),
    ParseFloatError(ParseFloatError),
}

impl From<ParseFloatError> for PSIParseError {
    fn from(value: ParseFloatError) -> Self {
        PSIParseError::ParseFloatError(value)
    }
}

impl FromStr for PressureStallInformation {
    type Err = PSIParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut some_line = None;
        let mut full_line = None;
        for line in s.split("\n") {
            if line.starts_with("some ") {
                if some_line.is_some() {
                    return Err(PSIParseError::TwoSomeLines);
                }
                some_line = Some(PressureStallLine::parse(&line["some ".len()..])?)
            } else if line.starts_with("full ") {
                if full_line.is_some() {
                    return Err(PSIParseError::TwoFullLines);
                }
                full_line = Some(PressureStallLine::parse(&line["full ".len()..])?)
            } else {
                return Err(PSIParseError::UnexpectedLine(line.to_string()));
            }
        }
        Ok(Self {
            some: some_line.ok_or(PSIParseError::MissingSomeLine)?,
            full: full_line.ok_or(PSIParseError::MissingFullLine)?,
        })
    }
}

#[derive(Debug)]
pub enum PSIReadError{
    IO(io::Error),
    Parse(PSIParseError)
}

impl From<PSIParseError> for PSIReadError{
    fn from(value: PSIParseError) -> Self {
        PSIReadError::Parse(value)
    }
}

impl From<io::Error> for PSIReadError{
    fn from(value: io::Error) -> Self {
        PSIReadError::IO(value)
    }
}

impl PressureStallInformation{
    fn pressure(path: impl AsRef<Path>) -> Result<Self, PSIReadError>{
        let to_parse = std::fs::read_to_string(path.as_ref())?;
        Ok(Self::from_str(to_parse.as_str())?)
    }

    pub fn cpu_pressure() -> Result<Self, PSIReadError>{
        Self::pressure("/proc/pressure/cpu")
    }

    pub fn io_pressure() -> Result<Self, PSIReadError>{
        Self::pressure("/proc/pressure/io")
    }

    pub fn mem_pressure() -> Result<Self, PSIReadError>{
        Self::pressure("/proc/pressure/memory")
    }
}

#[cfg(test)]
pub mod test {
    use std::str::FromStr;
    use crate::PressureStallInformation;

    const SAMPLE: &str = "some avg10=0.00 avg60=0.00 avg300=0.00 total=0
full avg10=0.00 avg60=0.00 avg300=0.00 total=0";

    #[test]
    pub fn test_sample_parse() {
        let psi = PressureStallInformation::from_str(SAMPLE).unwrap();
        assert_eq!(psi.full.avg10, 0.00);
        assert_eq!(psi.full.avg60, 0.00);
        assert_eq!(psi.full.avg300, 0.00);
        assert_eq!(psi.full.total, 0.00);
        assert_eq!(psi.some.avg10, 0.00);
        assert_eq!(psi.some.avg60, 0.00);
        assert_eq!(psi.some.avg300, 0.00);
        assert_eq!(psi.some.total, 0.00);
    }

    #[test]
    pub fn test_fetch_real_pressure() {
        let _ = PressureStallInformation::io_pressure().unwrap();
        let _ = PressureStallInformation::cpu_pressure().unwrap();
        let _ = PressureStallInformation::mem_pressure().unwrap();
    }
}

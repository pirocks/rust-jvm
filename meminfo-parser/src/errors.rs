use std::num::ParseIntError;

pub enum SyncError {
    IO(IOError),
    Parse(ParseError),
}

pub enum AsyncError {
    IO(IOError),
    Parse(ParseError),
}

#[derive(Debug)]
pub enum ParseError {
    MultipleColonsPerLine {
        line: String
    },
    InvalidMemoryAmount {
        line: String
    },
    MissingTotal,
    MissingFree,
    MissingAvailable,
    ParseIntError{
        err: ParseIntError
    }
}


pub struct IOError {
    err: std::io::Error,
}


impl From<std::io::Error> for SyncError {
    fn from(std_io_error: std::io::Error) -> Self {
        SyncError::IO(IOError {
            err: std_io_error
        })
    }
}

impl From<ParseError> for AsyncError {
    fn from(parse_error: ParseError) -> Self {
        AsyncError::Parse(parse_error)
    }
}

impl From<ParseError> for SyncError {
    fn from(parse_error: ParseError) -> Self {
        SyncError::Parse(parse_error)
    }
}

impl From<std::io::Error> for AsyncError {
    fn from(std_io_error: std::io::Error) -> Self {
        AsyncError::IO(IOError {
            err: std_io_error
        })
    }
}

impl From<ParseIntError> for ParseError{
    fn from(parse_int_error: ParseIntError) -> Self {
        Self::ParseIntError { err: parse_int_error }
    }
}
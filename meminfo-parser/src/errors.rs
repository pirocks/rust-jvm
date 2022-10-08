use memory_amount::MemoryAmountParseError;

pub enum SyncError {
    IO(IOError),
    Parse(ParseError),
}

#[derive(Debug)]
pub enum AsyncError {
    IO(IOError),
    Parse(ParseError),
}

#[derive(Debug)]
pub enum ParseError {
    MultipleColonsPerLine {
        line: String
    },
    InvalidMemoryAmount(MemoryAmountParseError),
    MissingTotal,
    MissingFree,
    MissingAvailable,
}

#[derive(Debug)]
#[allow(unused)]
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

impl From<MemoryAmountParseError> for ParseError{
    fn from(memory_amount: MemoryAmountParseError) -> Self {
        Self::InvalidMemoryAmount(memory_amount)
    }
}
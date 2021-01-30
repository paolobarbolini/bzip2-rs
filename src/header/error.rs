use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;

/// An error returned by [`Header`][crate::header::Header]
#[derive(Debug, Clone, PartialEq)]
pub enum HeaderError {
    InvalidSignature,
    UnsupportedVersion,
    InvalidBlockSize,
}

impl Display for HeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HeaderError::InvalidSignature => "invalid file signature",
            HeaderError::UnsupportedVersion => "unsupported bzip2 version",
            HeaderError::InvalidBlockSize => "invalid block-size",
        })
    }
}

impl StdError for HeaderError {}

impl From<HeaderError> for io::Error {
    fn from(err: HeaderError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, err)
    }
}

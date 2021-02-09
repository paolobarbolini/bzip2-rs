use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;

use crate::block::BlockError;
use crate::header::HeaderError;

/// An error returned by [`Decoder`] or [`DecoderReader`]
///
/// [`Decoder`]: crate::decoder::Decoder
/// [`DecoderReader`]: crate::decoder::DecoderReader
#[derive(Debug, Clone, PartialEq)]
pub enum DecoderError {
    /// An error was returned by the `Header` decoder
    Header(HeaderError),
    /// An error was returned by the `Block` decoder
    Block(BlockError),
}

impl Display for DecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DecoderError::Header(err) => write!(f, "header: {}", err),
            DecoderError::Block(err) => write!(f, "block: {}", err),
        }
    }
}

impl From<HeaderError> for DecoderError {
    fn from(err: HeaderError) -> Self {
        DecoderError::Header(err)
    }
}

impl From<BlockError> for DecoderError {
    fn from(err: BlockError) -> Self {
        DecoderError::Block(err)
    }
}

impl StdError for DecoderError {}

impl From<DecoderError> for io::Error {
    fn from(err: DecoderError) -> io::Error {
        match err {
            DecoderError::Header(err) => err.into(),
            DecoderError::Block(err) => err.into(),
        }
    }
}

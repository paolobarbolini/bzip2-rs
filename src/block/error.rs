use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;

/// An error returned by the block decoder
///
/// At the moment it's not possible to find out what
/// error occurred other than through the `Display`
/// implementation, this will change in a future release.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockError {
    reason: &'static str,
}

impl BlockError {
    #[inline(always)]
    pub(super) fn new(reason: &'static str) -> Self {
        Self { reason }
    }
}

impl Display for BlockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.reason)
    }
}

impl StdError for BlockError {}

impl From<BlockError> for io::Error {
    fn from(err: BlockError) -> io::Error {
        // TODO: do better at this
        io::Error::new(io::ErrorKind::Other, err)
    }
}

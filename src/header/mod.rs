//! bzip2 low-level header APIs

pub use self::error::HeaderError;

mod error;

/// A bzip2 header
#[derive(Clone)]
pub struct Header {
    raw_blocksize: u8,
    max_blocksize: u32,
}

impl Header {
    /// Parse a bzip2 header
    pub fn parse(buf: [u8; 4]) -> Result<Self, HeaderError> {
        let signature = &buf[..2];
        if signature != b"BZ" {
            return Err(HeaderError::InvalidSignature);
        }

        let version = buf[2];
        if version != b'h' {
            return Err(HeaderError::UnsupportedVersion);
        }

        let hundred_k_blocksize = buf[3];
        match hundred_k_blocksize {
            b'1'..=b'9' => {
                let raw_blocksize = hundred_k_blocksize - b'0';
                Self::from_raw_blocksize(raw_blocksize)
            }
            _ => Err(HeaderError::InvalidBlockSize),
        }
    }

    /// Write header into the buffer
    pub fn write(&self, buf: &mut [u8; 4]) {
        buf[0] = b'B';
        buf[1] = b'Z';
        buf[2] = b'h';
        buf[3] = self.raw_blocksize() + b'0';
    }

    /// Construct `Header` from the raw blocksize
    ///
    /// # Errors
    ///
    /// Returns [`HeaderError::InvalidBlockSize`] if `raw_blocksize`
    /// isn't `1..=9`
    pub fn from_raw_blocksize(raw_blocksize: u8) -> Result<Self, HeaderError> {
        if raw_blocksize < 1 || raw_blocksize > 9 {
            return Err(HeaderError::InvalidBlockSize);
        }

        let max_blocksize = 100 * 1000 * u32::from(raw_blocksize);
        Ok(Self {
            raw_blocksize,
            max_blocksize,
        })
    }

    /// The raw blocksize, as declared in the bzip2 header
    ///
    /// The returned value is always `1..=9`
    pub fn raw_blocksize(&self) -> u8 {
        self.raw_blocksize
    }

    /// The maximum blocksize
    ///
    /// The returned value is always `100000..=900000`
    pub fn max_blocksize(&self) -> u32 {
        self.max_blocksize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_9k() {
        let header = Header::parse(*b"BZh9").unwrap();
        assert_eq!(header.raw_blocksize(), 9);
        assert_eq!(header.max_blocksize(), 900000);
    }
}

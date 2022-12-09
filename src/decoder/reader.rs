#[cfg(feature = "nightly")]
use std::io::BorrowedBuf;
use std::io::{self, Read, Result};
#[cfg(feature = "nightly")]
use std::mem::MaybeUninit;

use super::{Decoder, ReadState};

/// A high-level **single-threaded** decoder that wraps a [`Read`] and implements [`Read`], yielding decompressed bytes
///
/// ```rust
/// use std::fs::File;
/// use std::io;
///
/// use bzip2_rs::DecoderReader;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut compressed_file = File::open("tests/samplefiles/sample1.bz2")?;
/// # fn no_run() -> Result<(), Box<dyn std::error::Error>> {
/// let mut output = File::create("output.ref")?;
/// # Ok(())
/// # }
/// # let mut output = Vec::new();
///
/// let mut reader = DecoderReader::new(compressed_file);
/// io::copy(&mut reader, &mut output)?;
/// #
/// # let expected = std::fs::read("tests/samplefiles/sample1.ref")?;
/// # assert_eq!(expected, output);
/// #
/// # Ok(())
/// # }
/// ```
pub struct DecoderReader<R> {
    decoder: Decoder,

    reader: R,
}

impl<R> DecoderReader<R> {
    /// Construct a new decoder from something implementing [`Read`]
    pub fn new(reader: R) -> Self {
        Self {
            decoder: Decoder::new(),

            reader,
        }
    }
}

impl<R: Read> Read for DecoderReader<R> {
    /// Decompress bzip2 data from the underlying reader
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut read_zero = false;
        #[cfg(not(feature = "nightly"))]
        let mut tmp_buf = [0; 1024];
        #[cfg(feature = "nightly")]
        let mut tmp_buf = [MaybeUninit::uninit(); 1024];
        #[cfg(feature = "nightly")]
        let mut read_buf = BorrowedBuf::from(tmp_buf.as_mut_slice());

        loop {
            match self.decoder.read(buf)? {
                ReadState::NeedsWrite => {
                    #[cfg(feature = "nightly")]
                    let read = {
                        read_buf.clear();
                        self.reader.read_buf(read_buf.unfilled())?;
                        read_buf.filled()
                    };
                    #[cfg(not(feature = "nightly"))]
                    let read = {
                        let n = self.reader.read(&mut tmp_buf)?;
                        &tmp_buf[..n]
                    };

                    if read_zero && self.decoder.header_block.is_none() {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "The reader is empty?",
                        ));
                    }
                    read_zero = read.is_empty();

                    self.decoder.write(read);
                }
                ReadState::Read(n) => return Ok(n),
                ReadState::Eof => return Ok(0),
            }
        }
    }
}

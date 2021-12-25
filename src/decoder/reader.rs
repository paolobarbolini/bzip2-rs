use std::io::{self, Read, Result};

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
        let mut tmp_buf = [0; 1024];

        loop {
            match self.decoder.read(buf)? {
                ReadState::NeedsWrite => {
                    let read = self.reader.read(&mut tmp_buf)?;

                    if read_zero && self.decoder.header_block.is_none() {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "The reader is empty?",
                        ));
                    }
                    read_zero = read == 0;

                    self.decoder.write(&tmp_buf[..read]);
                }
                ReadState::Read(n) => return Ok(n),
                ReadState::Eof => return Ok(0),
            }
        }
    }
}

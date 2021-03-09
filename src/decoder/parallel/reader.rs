use std::io::{self, Read, Result};

use super::{ParallelDecoder, ReadState, ThreadPool};

/// A high-level **multi-threaded** decoder that wraps a [`Read`] and implements [`Read`], yielding decompressed bytes
///
/// ```rust
/// use std::fs::File;
/// use std::io;
///
/// use bzip2_rs::{ParallelDecoderReader, RayonThreadPool};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut compressed_file = File::open("tests/samplefiles/sample1.bz2")?;
/// # fn no_run() -> Result<(), Box<dyn std::error::Error>> {
/// let mut output = File::create("output.ref")?;
/// # Ok(())
/// # }
/// # let mut output = Vec::new();
///
/// let mut reader = ParallelDecoderReader::new(RayonThreadPool, 1024 * 1024, compressed_file);
/// io::copy(&mut reader, &mut output)?;
/// #
/// # let expected = std::fs::read("tests/samplefiles/sample1.ref")?;
/// # assert_eq!(expected, output);
/// #
/// # Ok(())
/// # }
/// ```
pub struct ParallelDecoderReader<R, P> {
    decoder: ParallelDecoder<P>,

    reader: R,
}

impl<R, P> ParallelDecoderReader<R, P> {
    /// Construct a new decoder from something implementing [`Read`]
    pub fn new(pool: P, max_preread_len: usize, reader: R) -> Self {
        Self {
            decoder: ParallelDecoder::new(pool, max_preread_len),

            reader,
        }
    }
}

impl<R: Read, P: ThreadPool> Read for ParallelDecoderReader<R, P> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut read_zero = false;
        let mut tmp_buf = [0; 1024];

        loop {
            match self.decoder.read(buf)? {
                ReadState::NeedsWrite(space) => {
                    let read = self.reader.read(&mut tmp_buf[..space.min(1024)])?;

                    if read_zero && self.decoder.header.is_none() {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "The reader is empty?",
                        ));
                    }
                    read_zero = read == 0;

                    self.decoder.write(&tmp_buf[..read])?;
                }
                ReadState::Read(n) => return Ok(n),
                ReadState::Eof => return Ok(0),
            }
        }
    }
}

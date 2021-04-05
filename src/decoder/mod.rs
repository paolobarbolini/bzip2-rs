//! bzip2 decoding APIs

use std::convert::TryInto;

use self::block::Block;
pub use self::error::DecoderError;
pub use self::parallel::{ParallelDecoder, ParallelDecoderReader};
pub use self::reader::DecoderReader;
pub use self::state::{ReadState, WriteState};
use crate::bitreader::BitReader;
use crate::header::Header;

pub mod block;
mod error;
mod parallel;
mod reader;
mod state;

/// A low-level **single-threaded** decoder implementation
///
/// This decoder does no IO by itself, instead enough data
/// has to be written to it in order for it to be able
/// to decode the next block. After that the decompressed content
/// for the block can be read until all of the data from the block
/// has been exhausted.
/// Repeating this process for every block in sequence will result
/// into the entire file being decompressed.
///
/// ```rust
/// use bzip2_rs::decoder::{Decoder, ReadState, WriteState};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut compressed_file: &[u8] = include_bytes!("../../tests/samplefiles/sample1.bz2").as_ref();
/// let mut output = Vec::new();
///
/// let mut decoder = Decoder::new();
///
/// assert!(
///     !compressed_file.is_empty(),
///     "empty files will cause the following loop to spin forever"
/// );
///
/// let mut buf = [0; 1024];
/// loop {
///     match decoder.read(&mut buf)? {
///         ReadState::NeedsWrite(space) => {
///             // `Decoder` needs more data to be written to it before it
///             // can decode the next block.
///             // If we reached the end of the file `compressed_file.len()` will be 0,
///             // signaling to the `Decoder` that the last block is smaller and it can
///             // proceed with reading.
///             match decoder.write(&compressed_file)? {
///                 WriteState::NeedsRead => unreachable!(),
///                 WriteState::Written(written) => compressed_file = &compressed_file[written..],
///             };
///         }
///         ReadState::Read(n) => {
///             // `n` uncompressed bytes have been read into `buf`
///             output.extend_from_slice(&buf[..n]);
///         }
///         ReadState::Eof => {
///             // we reached the end of the file
///             break;
///         }
///     }
/// }
///
/// // `output` contains the decompressed file
/// let decompressed_file: &[u8] = include_bytes!("../../tests/samplefiles/sample1.ref").as_ref();
/// assert_eq!(output, decompressed_file);
/// #
/// # Ok(())
/// # }
/// ```
pub struct Decoder {
    header_block: Option<(Header, Block)>,

    skip_bits: usize,
    in_buf: Vec<u8>,

    eof: bool,
}

impl Decoder {
    /// Construct a new [`Decoder`], ready to decompress a new bzip2 file
    pub fn new() -> Self {
        Self {
            header_block: None,

            skip_bits: 0,
            in_buf: Vec::new(),

            eof: false,
        }
    }

    fn space(&self) -> usize {
        match &self.header_block {
            Some((_, block)) if block.is_reading() => 0,
            Some((header, _)) => {
                let max_length = header.max_blocksize() as usize + (self.skip_bits / 8) + 1;
                max_length - self.in_buf.len()
            }
            None => {
                Header::from_raw_blocksize(1)
                    .expect("blocksize is valid")
                    .max_blocksize() as usize
                    + 4
            }
        }
    }

    /// Write more compressed data into this [`Decoder`]
    ///
    /// See the documentation for [`WriteState`] to decide
    /// what to do next.
    pub fn write(&mut self, buf: &[u8]) -> Result<WriteState, DecoderError> {
        let space = self.space();

        match &mut self.header_block {
            Some((_, block)) if block.is_reading() => Ok(WriteState::NeedsRead),
            Some((header, block)) => {
                let written = space.min(buf.len());

                self.in_buf.extend_from_slice(&buf[..written]);

                let minimum = (self.skip_bits / 8) + header.max_blocksize() as usize;
                if buf.is_empty() || self.in_buf.len() >= minimum {
                    block.set_ready_for_read();
                }

                Ok(WriteState::Written(written))
            }
            None => {
                let written = space.min(buf.len());
                self.in_buf.extend_from_slice(&buf[..written]);

                if self.in_buf.len() < 4 {
                    return Ok(WriteState::Written(buf.len()));
                }

                let header = Header::parse(self.in_buf[..4].try_into().unwrap())?;
                let block = Block::new(header.clone());
                self.header_block = Some((header, block));

                self.skip_bits = 4 * 8;

                if written == buf.len() {
                    return Ok(WriteState::Written(written));
                }

                match self.write(&buf[written..])? {
                    WriteState::NeedsRead => unreachable!(),
                    WriteState::Written(n) => Ok(WriteState::Written(n + written)),
                }
            }
        }
    }

    /// Read more decompressed data from this [`Decoder`]
    ///
    /// See the documentation for [`ReadState`] to decide
    /// what to do next.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<ReadState, DecoderError> {
        match &mut self.header_block {
            Some(_) if self.eof => Ok(ReadState::Eof),
            Some((_, block)) if block.is_not_ready() => Ok(ReadState::NeedsWrite(self.space())),
            Some((_, block)) => {
                let mut reader = BitReader::new(&self.in_buf);
                reader.advance_by(self.skip_bits);

                let ready_for_read = block.is_ready_for_read();

                let read = block.read(&mut reader, buf)?;

                if read == 0 {
                    if !buf.is_empty() {
                        self.eof = ready_for_read;
                    }

                    return Ok(ReadState::NeedsWrite(self.space()));
                }

                if read == 0 && !buf.is_empty() {
                    self.eof = true;
                }

                self.skip_bits = reader.position();

                if block.is_not_ready() {
                    let bytes = self.skip_bits / 8;

                    self.in_buf.drain(..bytes);

                    self.skip_bits -= bytes * 8;
                }

                Ok(ReadState::Read(read))
            }
            None => Ok(ReadState::NeedsWrite(self.space())),
        }
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

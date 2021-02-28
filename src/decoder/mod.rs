//! bzip2 decoding APIs

use std::convert::TryInto;

use self::block::Block;
pub use self::error::DecoderError;
pub use self::reader::DecoderReader;
use crate::bitreader::BitReader;
use crate::header::Header;

pub mod block;
mod error;
mod reader;

/// A low-level decoder implementation
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

    state: State,
}

/// State returned by [`Decoder::write`]
pub enum WriteState {
    /// Enough data has already been written to [`Decoder`]
    /// in order for it to be able to decode the next block.
    /// Now call [`Decoder::read`] to read the decompressed data.
    NeedsRead,
    /// N. number of bytes have been written.
    Written(usize),
}

/// State returned by [`Decoder::read`]
pub enum ReadState {
    /// Not enough data has been written to the underlying [`Decoder`]
    /// in order to allow the next block to be decoded. Call
    /// [`Decoder::write`] to write more data. If the end of the file
    /// has been reached, call [`Decoder::write`] with an empty buffer.
    NeedsWrite(usize),
    /// N. number of data has been read
    Read(usize),
    /// The end of the compressed file has been reached and
    /// there is no more data to read
    Eof,
}

#[derive(Copy, Clone)]
enum State {
    Writing,
    NeedsBlock,
    Reading,
    Eof,
}

impl Decoder {
    /// Construct a new [`Decoder`], ready to decompress a new bzip2 file
    pub fn new() -> Self {
        Self {
            header_block: None,

            skip_bits: 0,
            in_buf: Vec::new(),

            state: State::Writing,
        }
    }

    fn space(&self) -> usize {
        match (&self.header_block, self.state) {
            (Some((header, _)), State::Writing) => {
                let max_length = header.max_blocksize() as usize + (self.skip_bits / 8) + 1;
                max_length - self.in_buf.len()
            }
            (Some(_), _) => 0,
            (None, _) => {
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

        match (&mut self.header_block, self.state) {
            (Some((header, _)), State::Writing) => {
                let written = space.min(buf.len());

                self.in_buf.extend_from_slice(&buf[..written]);

                let minimum = header.max_blocksize() as usize + (self.skip_bits / 8);
                if buf.is_empty() || self.in_buf.len() >= minimum {
                    self.state = State::NeedsBlock;
                }

                Ok(WriteState::Written(written))
            }
            (Some(_), _) => Ok(WriteState::NeedsRead),
            (None, _) => {
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
                    WriteState::Written(n) => Ok(WriteState::Written(n + written)),
                    WriteState::NeedsRead => unreachable!(),
                }
            }
        }
    }

    /// Read more decompressed data from this [`Decoder`]
    ///
    /// See the documentation for [`ReadState`] to decide
    /// what to do next.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<ReadState, DecoderError> {
        match (&mut self.header_block, self.state) {
            (Some((_, block)), State::NeedsBlock) => {
                // decode a new block
                let mut reader = BitReader::new(&self.in_buf);
                reader.advance_by(self.skip_bits);

                let r = block.read_block(&mut reader)?;

                self.skip_bits = reader.position();

                // drain the fully read bytes
                let bytes = self.skip_bits / 8;
                self.in_buf.drain(..bytes);
                self.skip_bits -= bytes * 8;

                // set the new state
                match r {
                    Some(()) => {
                        self.state = State::Reading;
                        self.read(buf)
                    }
                    None => {
                        self.state = State::Eof;
                        Ok(ReadState::Eof)
                    }
                }
            }
            (Some((_, block)), State::Reading) => {
                let read = block.read_from_block(buf);

                if buf.is_empty() || read != 0 {
                    Ok(ReadState::Read(read))
                } else {
                    block.check_crc()?;

                    self.state = State::Writing;
                    Ok(ReadState::NeedsWrite(self.space()))
                }
            }
            (Some(_), State::Writing) => Ok(ReadState::NeedsWrite(self.space())),
            (Some(_), State::Eof) => Ok(ReadState::Eof),
            (None, _) => Ok(ReadState::NeedsWrite(self.space())),
        }
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

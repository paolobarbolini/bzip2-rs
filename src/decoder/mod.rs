//! bzip2 decoding APIs

use std::convert::TryInto;
use std::mem;

use self::block::{Decoder as BlockDecoder, Reader as BlockReader};
pub use self::error::DecoderError;
pub use self::parallel::{ParallelDecoder, ParallelDecoderReader};
pub use self::reader::DecoderReader;
pub use self::state::ReadState;
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
    state: State,

    skip_bits: usize,
    in_buf: Vec<u8>,

    ever_read_block: bool,
    write_eof: bool,
}

enum State {
    Uninit,
    Decoding(BlockDecoder),
    Reading(BlockReader),
    Eof,

    Poisoned,
}

impl Decoder {
    /// Construct a new [`Decoder`], ready to decompress a new bzip2 file
    pub fn new() -> Self {
        Self {
            state: State::Uninit,

            skip_bits: 0,
            in_buf: Vec::new(),

            ever_read_block: false,
            write_eof: false,
        }
    }

    /// Read more decompressed data from this [`Decoder`]
    ///
    /// This method should be called in a loop and based on the
    /// returned [`ReadState`] act upon it. Here's a simple version
    /// of what should be done based on the returned [`ReadState`]
    ///
    /// * [`ReadState::NeedsWrite`]: more data must be provided by calling
    ///   [`Decoder::write`], or [`Decoder::write_eof`] if EOF has been reached.
    /// * [`ReadState::Read(bytes)`]: `&buf[..bytes]` contains decoded data.
    ///   Handle it and loop
    /// * [`ReadState::Eof`]: [`Decoder::write_eof`] has been called and all
    ///   remaining data has been read already. Break out of the loop.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<ReadState, DecoderError> {
        debug_assert!(self.skip_bits / 8 <= self.in_buf.len());

        match &mut self.state {
            State::Uninit => {
                debug_assert_eq!(self.skip_bits % 8, 0);

                let in_buf_len = self.in_buf.len() - (self.skip_bits / 8);
                if in_buf_len >= 4 {
                    let array4: [u8; 4] =
                        self.in_buf[self.skip_bits / 8..][..4].try_into().unwrap();
                    let header = Header::parse(array4)?;

                    let decoder = BlockDecoder::new(header);
                    self.state = State::Decoding(decoder);
                    self.skip_bits = 4 * 8;

                    self.read(buf)
                } else if self.write_eof {
                    Err(DecoderError::NoBlocks)
                } else {
                    Ok(ReadState::NeedsWrite)
                }
            }
            State::Decoding(decoder) => {
                let in_buf_len = self.in_buf.len() - (self.skip_bits / 8);
                if (self.write_eof && in_buf_len > 0)
                    || in_buf_len > (decoder.header().max_blocksize() as usize)
                {
                    self.ever_read_block = true;

                    let state = mem::replace(&mut self.state, State::Poisoned);
                    let decoder = match state {
                        State::Decoding(decoder) => decoder,
                        _ => unreachable!(),
                    };

                    let mut bitreader = BitReader::new(&self.in_buf, self.skip_bits);

                    let decoded = decoder.decode(&mut bitreader)?;
                    self.skip_bits = bitreader.bit_position();

                    // we need to skip at least one byte
                    if self.skip_bits >= 8 {
                        self.in_buf.drain(..self.skip_bits / 8);
                        self.skip_bits %= 8;
                    }

                    match decoded {
                        Some(reader) => {
                            self.state = State::Reading(reader);

                            self.read(buf)
                        }
                        None if self.write_eof => {
                            self.state = State::Eof;
                            Ok(ReadState::Eof)
                        }
                        None => {
                            self.state = State::Uninit;
                            self.read(buf)
                        }
                    }
                } else if self.write_eof {
                    if self.ever_read_block {
                        Ok(ReadState::Eof)
                    } else {
                        Err(DecoderError::NoBlocks)
                    }
                } else {
                    Ok(ReadState::NeedsWrite)
                }
            }
            State::Reading(reader) => match reader.read(buf) {
                0 if !buf.is_empty() => {
                    let result = reader.check_crc();

                    let state = mem::replace(&mut self.state, State::Poisoned);
                    let reader = match state {
                        State::Reading(reader) => reader,
                        _ => unreachable!(),
                    };
                    self.state = State::Decoding(reader.recycle());

                    match result {
                        Ok(()) => self.read(buf),
                        Err(err) => Err(err.into()),
                    }
                }
                read => Ok(ReadState::Read(read)),
            },
            State::Eof if !self.ever_read_block => Err(DecoderError::NoBlocks),
            State::Eof => Ok(ReadState::Eof),
            State::Poisoned => {
                unreachable!("Decoder is in a Poisoned State");
            }
        }
    }

    /// Write more compressed data into this [`Decoder`]
    /// 
    /// The entire `buf` will always be written. This method
    /// implements **no flow control**. Writing 1 MB always guarantees
    /// that [`Decoder::read`] will decode at least 1 full block, so
    /// it doesn't make sense to write more unless you want to OOM I guess.
    ///
    /// # Panics
    /// 
    /// This method panics if called _after_ the stream has been
    /// declared EOF by calling [`Decoder::write_eof`].
    pub fn write(&mut self, buf: &[u8]) {
        assert!(
            !self.write_eof,
            "Attempted to write after calling `Decoder::write_eof`"
        );

        self.in_buf.extend_from_slice(buf);
    }

    /// Declare that the end of the compressed file has been reached
    ///
    /// After this no more calls to [`Decoder::write`] can be made.
    /// [`Decoder::read`] can still be called in order the read the
    /// remaining data.
    pub fn write_eof(&mut self) {
        self.write_eof = true;
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

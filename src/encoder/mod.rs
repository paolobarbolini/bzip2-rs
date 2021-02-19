//! bzip2 encoding APIs

pub use self::error::EncoderError;
use crate::encblock::Block;
use crate::header::Header;
use std::convert::TryFrom;

mod error;

/// Low level compressor implementation
///
/// This compresor does no IO by itself, instead we first need to `write` enough data into this
/// compressor, so it is able to produce a compressed block, after which you can `read` the
/// compressed data until it is exhausted. Repeating this process multiple times
/// compresses the entire file.

pub enum WriteState {
    NeedsRead,
    Written(usize),
}

pub enum ReadState {
    NeedsWrite(usize),
    Read(usize),
    Eof,
}

pub struct Compressor {
    header: Header,
    block: Block,

    state: State,
}

/// Indicates which section of the block needs to be read
enum State {
    /// Header
    FirstHeader,
    /// Block
    Block,
    /// Final checksum header
    FinalHeader,
    /// EOF
    Finished,
}

impl Compressor {
    /// Create [`Compressor`] with custom blocksize
    pub fn new(raw_blocksize: u8, work_factor: u8) -> Self {
        if work_factor > 250 {
            work_factor = 250;
        }
        if work_factor == 0 {
            work_factor = 30;
        }

        let header = Header::from_raw_blocksize(raw_blocksize).expect("is valid block size");
        let block = Block::new(header.clone(), work_factor);

        Self {
            header,
            block,
            state: State::FirstHeader,
        }
    }

    /// Create [`Compressor`] with best blocksize
    pub fn default_new() -> Self {
        Self::new(9, 0)
    }

    /// Write uncompressed data into this [`Compressor`]
    pub fn write(&mut self, buf: &[u8]) -> WriteState {
        if self.block.is_writing() {
            return WriteState::NeedsRead;
        }

        let written = self.block.read(buf);

        WriteState::Written(written)
    }

    /// Called only once when user input stream has ended.
    /// Ideally after this method call the [`Compressor`] object should be dropped.
    pub fn write_eof(&mut self) {
        todo!();
    }

    /// Read compressed data from this [`Compressor`]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<ReadState, EncoderError> {
        let mut read_count: usize = 0;

        match &mut self.state {
            State::FirstHeader => {
                self.header
                    .write(<&mut [u8; 4]>::try_from(&mut buf[..4]).unwrap());
                read_count += 4;
                self.state = State::Block;
            }
            State::Block => {
                todo!("check block state: writing or waiting for input");
                read_count += self.block.write(buf);
            }
            State::FinalHeader => {}
            State::Finished => {}
        }

        Ok(ReadState::Read(read_count))
    }
}

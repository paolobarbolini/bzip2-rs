use std::convert::TryInto;
use std::mem;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

pub use self::reader::ParallelDecoderReader;
use self::scanner::threaded::find_signatures_parallel;
use self::util::ReadableVec;
use crate::bitreader::BitReader;
use crate::decoder::block::{Block, BlockError, FINAL_MAGIC};
use crate::decoder::{DecoderError, ReadState};
use crate::header::Header;
use crate::ThreadPool;

mod reader;
mod scanner;
mod util;

/// (block index, Result<(PreRead Block, Block)>)
type ChannelledBlock = (usize, Result<(ReadableVec, Block), BlockError>);

/// A low-level **multi-threaded** decoder implementation
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
/// # // A fake threadpool just to make the docs build when the `rayon` feature isn't enabled
/// # #[cfg(not(feature = "rayon"))]
/// # struct RayonThreadPool;
/// #
/// # #[cfg(not(feature = "rayon"))]
/// # impl bzip2_rs::ThreadPool for RayonThreadPool {
/// #     fn spawn<F>(&self, func: F)
/// #     where
/// #         F: FnOnce() + Send + 'static,
/// #     {
/// #         std::thread::spawn(func);
/// #     }
/// #
/// #     fn max_threads(&self) -> std::num::NonZeroUsize {
/// #         std::num::NonZeroUsize::new(1).unwrap()
/// #     }
/// # }
/// #
/// use bzip2_rs::decoder::{ParallelDecoder, ReadState, WriteState};
/// // use the rayon global threadpool as the threadpool for decoding this file.
/// // requires the `rayon` feature to be enabled
/// # #[cfg(feature = "rayon")]
/// use bzip2_rs::RayonThreadPool;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut compressed_file: &[u8] =
///     include_bytes!("../../../tests/samplefiles/sample1.bz2").as_ref();
/// let mut output = Vec::new();
///
/// let mut decoder = ParallelDecoder::new(RayonThreadPool, 1024 * 1024);
///
/// let mut buf = [0; 8192];
/// loop {
///     match decoder.read(&mut buf)? {
///         ReadState::NeedsWrite(space) => {
///             // `ParallelDecoder` needs more data to be written to it before it
///             // can decode the next block.
///             // If we reached the end of the file `compressed_file.len()` will be 0,
///             // signaling to the `Decoder` that the last block is smaller and it can
///             // proceed with reading.
///             decoder.write(&compressed_file)?;
///             compressed_file = &[];
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
/// let decompressed_file: &[u8] =
///     include_bytes!("../../../tests/samplefiles/sample1.ref").as_ref();
/// assert_eq!(output, decompressed_file);
/// #
/// # Ok(())
/// # }
/// ```
pub struct ParallelDecoder<P> {
    header: Option<Header>,
    in_buf: Vec<u8>,
    skip_bits: usize,

    pool: P,
    sender: Sender<ChannelledBlock>,
    receiver: Receiver<ChannelledBlock>,
    receive_index: usize,
    receive_pool: Vec<Option<(ReadableVec, Block)>>,

    max_preread_len: usize,

    eof: bool,
}

impl<P> ParallelDecoder<P> {
    /// Construct a new [`ParallelDecoder`], ready to decompress a new bzip2 file
    pub fn new(pool: P, max_preread_len: usize) -> Self {
        let (sender, receiver) = channel::<ChannelledBlock>();

        Self {
            header: None,
            in_buf: Vec::new(),
            skip_bits: 0,

            pool,
            sender,
            receiver,
            receive_index: 0,
            receive_pool: Vec::new(),

            max_preread_len,

            eof: false,
        }
    }
}

impl<P: ThreadPool> ParallelDecoder<P> {
    /// Read decompressed data into `buf`.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<ReadState, DecoderError> {
        match self.receive_pool.get_mut(self.receive_index) {
            Some(Some((pre_read, block))) => {
                // there's a block here

                let original_len = buf.len();
                let buf = pre_read.read(buf);
                let read1 = original_len - buf.len();

                if !buf.is_empty() && original_len > 0 {
                    // the pre_read has been exhausted
                    let read2 = block.read_from_block(buf)?;

                    if read2 == 0 {
                        self.go_to_next_block();

                        let r = self.read(buf)?;
                        match r {
                            ReadState::NeedsWrite(n) if read1 == 0 => Ok(ReadState::NeedsWrite(n)),
                            ReadState::NeedsWrite(_) => Ok(ReadState::Read(read1)),
                            ReadState::Read(n) => Ok(ReadState::Read(read1 + n)),
                            ReadState::Eof if read1 == 0 => Ok(ReadState::Eof),
                            ReadState::Eof => Ok(ReadState::Read(read1)),
                        }
                    } else {
                        Ok(ReadState::Read(read1 + read2))
                    }
                } else {
                    Ok(ReadState::Read(read1))
                }
            }
            Some(None) => {
                // this block is already scheduled for decoding

                loop {
                    let (receive_index, block) = self.receiver.recv().unwrap();
                    let block = block?;

                    self.receive_pool[receive_index] = Some(block);

                    // we finally got the block we were waiting for
                    if self.receive_index == receive_index {
                        return self.read(buf);
                    }
                }
            }
            None => {
                // this block hasn't yet been scheduled for decoding

                if self.eof && self.receive_pool.len() == self.receive_index {
                    // the eof flag has been set, and no more blocks are in the queue.
                    // we reached the eof
                    Ok(ReadState::Eof)
                } else {
                    // more blocks are available for decoding
                    Ok(ReadState::NeedsWrite(usize::max_value()))
                }
            }
        }
    }

    /// Write `buf` compressed bytes into this decoder
    pub fn write(&mut self, buf: &[u8]) -> Result<(), DecoderError> {
        if self.eof {
            return if buf.is_empty() {
                Ok(())
            } else {
                Err(BlockError::new("eof").into())
            };
        }

        match self.header.clone() {
            Some(header) => {
                if buf.is_empty() {
                    self.eof = true;
                } else {
                    self.in_buf.extend_from_slice(&buf);
                }

                let skip_bytes = self.skip_bits / 8;
                let filled_portion = self.in_buf.len() - skip_bytes;
                let min_blocks = filled_portion / (header.max_blocksize() as usize);

                if buf.is_empty() || min_blocks >= self.pool.max_threads().get() {
                    // let's decode the blocks in `self.in_buf`

                    let in_buf = mem::replace(&mut self.in_buf, Vec::new());
                    let in_buf = Arc::<[u8]>::from(in_buf);

                    let in_buf_ = Arc::clone(&in_buf);
                    let mut signatures = find_signatures_parallel(in_buf_, &self.pool);
                    match signatures.pop() {
                        Some(last_signature) => {
                            if !buf.is_empty() {
                                // this isn't the last write, so we have to put the last block back into `self.in_buf`
                                // because it's still truncated at this stage
                                self.in_buf
                                    .extend_from_slice(&in_buf[(last_signature / 8) as usize..]);
                            } else {
                                // this is the last write, put the last block back
                                signatures.push(last_signature);
                            }
                            self.skip_bits = (last_signature % 8) as usize;

                            let num_signatures = signatures.len();
                            for signature_index in signatures {
                                let block_index = self.receive_pool.len();
                                let max_preread_len = self.max_preread_len / num_signatures;
                                let sender = self.sender.clone();
                                let header = header.clone();
                                let in_buf = Arc::clone(&in_buf);

                                // get a space for writing the decoded block into
                                self.receive_pool.push(None);

                                // spawn the block decoder
                                self.pool.spawn(move || {
                                    let mut reader =
                                        BitReader::new(&in_buf[(signature_index / 8) as usize..]);
                                    assert!(reader.advance_by((signature_index % 8) as usize));

                                    let mut block = Block::new(header);
                                    match block.read_block(&mut reader) {
                                        Ok(b) => {
                                            if b.is_none() {
                                                // we reached the EOF
                                                return;
                                            }

                                            let mut pre_read = Vec::new();

                                            loop {
                                                let remaining = max_preread_len - pre_read.len();
                                                if remaining == 0 {
                                                    // we reached the maximum pre read len
                                                    break;
                                                }

                                                let mut filled = pre_read.len();
                                                pre_read
                                                    .resize(filled + remaining.min(32 * 1024), 0);
                                                match block.read_from_block(&mut pre_read[filled..])
                                                {
                                                    Ok(read) => {
                                                        filled += read;

                                                        // will the next read succeed?
                                                        let end = filled < pre_read.len();

                                                        // remove the extra zeros
                                                        pre_read.truncate(filled);

                                                        if end {
                                                            // end of block
                                                            break;
                                                        }
                                                    }
                                                    Err(err) => {
                                                        let _ =
                                                            sender.send((block_index, Err(err)));
                                                        break;
                                                    }
                                                }
                                            }

                                            let pre_read = ReadableVec::from(pre_read);
                                            let _ =
                                                sender.send((block_index, Ok((pre_read, block))));
                                        }
                                        Err(err) => {
                                            let _ = sender.send((block_index, Err(err)));
                                        }
                                    }
                                });
                            }
                        }
                        None => {
                            // no signatures where found???

                            let mut reader = BitReader::new(&in_buf);
                            if !reader.advance_by(self.skip_bits) {
                                return Err(
                                    BlockError::new("no blocks have been found - eof").into()
                                );
                            }

                            let magic = reader.read_u64(48).ok_or_else(|| {
                                BlockError::new("no blocks have been found - eof")
                            })?;
                            if magic != FINAL_MAGIC {
                                return Err(BlockError::new("no blocks have been found").into());
                            }

                            self.eof = true;
                        }
                    }
                }
            }
            None => {
                self.in_buf.extend_from_slice(&buf);

                if self.in_buf.len() >= 4 {
                    let header = Header::parse(self.in_buf[..4].try_into().unwrap())?;
                    self.header = Some(header);

                    self.skip_bits = 4 * 8;
                }
            }
        }

        Ok(())
    }

    fn go_to_next_block(&mut self) {
        // deallocate Block and Vec<u8>
        self.receive_pool[self.receive_index] = None;
        // go to the next block
        self.receive_index += 1;
    }
}

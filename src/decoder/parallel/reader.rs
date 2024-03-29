#[cfg(feature = "nightly")]
use std::io::BorrowedBuf;
use std::io::{self, Read, Result};
#[cfg(feature = "nightly")]
use std::mem::MaybeUninit;

use super::{ParallelDecoder, ReadState, ThreadPool};

/// A high-level **multi-threaded** decoder that wraps a [`Read`] and implements [`Read`], yielding decompressed bytes
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
/// use std::fs::File;
/// use std::io;
///
/// use bzip2_rs::ParallelDecoderReader;
/// // use the rayon global threadpool as the threadpool for decoding this file.
/// // requires the `rayon` feature to be enabled
/// # #[cfg(feature = "rayon")]
/// use bzip2_rs::RayonThreadPool;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut compressed_file = File::open("tests/samplefiles/sample1.bz2")?;
/// # fn no_run() -> Result<(), Box<dyn std::error::Error>> {
/// let mut output = File::create("output.ref")?;
/// # Ok(())
/// # }
/// # let mut output = Vec::new();
///
/// let mut reader = ParallelDecoderReader::new(compressed_file, RayonThreadPool, 1024 * 1024);
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
    ///
    /// Compared to [`DecoderReader`], this decoder implements multi-threaded decoding.
    /// This is done by scanning the bitstream for bzip2 block signatures to find
    /// the next block, and then decoding each of them in a separate thread.
    /// Because of the overhead of having to scan the bitstream, this decoder really
    /// shines on systems with more than two threads.
    ///
    /// [`ParallelDecoderReader`] takes `P` as a generic argument, which is the [`ThreadPool`]
    /// implementation used for spawning tasks. If the `rayon` feature is enabled,
    /// [`RayonThreadPool`] can be used, with internally uses the `rayon` global
    /// threadpool for spawning tasks.
    ///
    /// `max_preread_len` defines how many bytes can be pre-read from the block. This
    /// significantly speeds up the reading process, which would otherwise limit the decoder
    /// to using at most two threads, independently of how many more are available.
    /// Setting a value close to zero is then highly discouraged, at the same time
    /// using a value higher than the amount of available memory could lead to OOM
    /// for files with a high compression ratio.
    ///
    /// [`DecoderReader`]: crate::DecoderReader
    /// [`RayonThreadPool`]: crate::RayonThreadPool
    pub fn new(reader: R, pool: P, max_preread_len: usize) -> Self {
        Self {
            decoder: ParallelDecoder::new(pool, max_preread_len),

            reader,
        }
    }
}

impl<R: Read, P: ThreadPool> Read for ParallelDecoderReader<R, P> {
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

                    if read_zero && self.decoder.header.is_none() {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "The reader is empty?",
                        ));
                    }
                    read_zero = read.is_empty();

                    self.decoder.write(read)?;
                }
                ReadState::Read(n) => return Ok(n),
                ReadState::Eof => return Ok(0),
            }
        }
    }
}

//! `bzip2_rs` is a pure Rust bzip2 decoder.
//!
//! ## Main APIs
//!
//! ### Single-threaded decoder
//!
//! * [`Decoder`]: low-level, Sans I/O, bzip2 decoder
//! * [`DecoderReader`]: high-level synchronous bzip2 decoder
//!
//! ### Multi-threaded decoder
//!
//! * [`ParallelDecoder`]: low-level, Sans I/O, bzip2 decoder
//! * [`ParallelDecoderReader`]: high-level synchronous bzip2 decoder
//!
//! ## Features
//!
//! * `rayon`: enable using the [rayon] global threadpool for parallel decoding.
//!            NOTE: this feature is not subject to the normal MSRV. At the time
//!            of writing the MSRV for rayon is 1.36.0
//!
//! * Default features: Rust >= 1.34.2 is supported
//! * `rustc_1_37`: bump MSRV to 1.37, enable more optimizations
//! * `nightly`: require Rust Nightly, enable more optimizations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::io;
//! use bzip2_rs::DecoderReader;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut compressed_file = File::open("input.bz2")?;
//! let mut decompressed_output = File::create("output")?;
//!
//! let mut reader = DecoderReader::new(compressed_file);
//! io::copy(&mut reader, &mut decompressed_output)?;
//! # Ok(())
//! # }
//! ```
//!
//! [`Decoder`]: crate::decoder::Decoder
//! [`ParallelDecoder`]: crate::decoder::ParallelDecoder
//! [rayon]: https://crates.io/crates/rayon

#![deny(
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    clippy::cast_lossless,
    clippy::doc_markdown,
    missing_docs,
    broken_intra_doc_links
)]
#![forbid(unsafe_code)]
// TODO: remove once rustc 1.35 is our MSRV
#![allow(clippy::manual_range_contains)]
// TODO: remove once rustc 1.40 is our msrv
#![allow(clippy::mem_replace_with_default)]
#![cfg_attr(feature = "nightly", feature(maybe_uninit_write_slice))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(no_inline)]
pub use self::decoder::{DecoderReader, ParallelDecoderReader};
#[cfg(feature = "rayon")]
pub use self::threadpool::RayonThreadPool;
pub use self::threadpool::ThreadPool;

mod bitreader;
mod crc;
pub mod decoder;
pub mod header;
mod huffman;
mod move_to_front;
mod threadpool;

#[doc(hidden)]
#[deprecated(note = "moved to bzip2_rs::decoder::block", since = "0.1.3")]
pub mod block {
    #[deprecated(
        note = "moved to bzip2_rs::decoder::block::BlockError",
        since = "0.1.3"
    )]
    pub type BlockError = crate::decoder::block::BlockError;
}

#[cfg(feature = "nightly")]
const LEN_258: usize = 258;
#[cfg(not(feature = "nightly"))]
const LEN_258: usize = 512;

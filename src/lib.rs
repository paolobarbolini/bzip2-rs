//! `bzip2_rs` is a pure Rust bzip2 decoder.
//!
//! ## Main APIs
//!
//! * [`Decoder`]: low-level, no IO, bzip2 decoder
//! * [`DecoderReader`]: high-level synchronous bzip2 decoder
//!
//! ## Features
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
#![cfg_attr(feature = "nightly", feature(maybe_uninit_write_slice))]

#[doc(no_inline)]
pub use self::decoder::DecoderReader;

mod bitreader;
mod block_common;
mod crc;
pub mod decoder;
pub mod encblock;
pub mod encoder;
pub mod header;
mod huffman;
mod move_to_front;

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

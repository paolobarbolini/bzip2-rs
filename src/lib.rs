//! bzip2_rs is a pure Rust bzip2 decoder.
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
//! * `rustc_1_40`: bump MSRV to 1.40, enable more optimizations
//! * `rustc_1_51`: bump MSRV to 1.51, enable more optimizations
//!
//! [`Decoder`]: crate::decoder::DecoderReader

#![deny(trivial_casts, trivial_numeric_casts, rust_2018_idioms)]
#![forbid(unsafe_code)]
// TODO: remove once rustc 1.35 is our MSRV
#![allow(clippy::manual_range_contains)]

#[doc(no_inline)]
pub use self::decoder::DecoderReader;

mod bitreader;
pub mod block;
mod crc;
pub mod decoder;
pub mod header;
mod huffman;
mod move_to_front;

#[cfg(feature = "rustc_1_51")]
const LEN_258: usize = 258;
#[cfg(not(feature = "rustc_1_51"))]
const LEN_258: usize = 512;

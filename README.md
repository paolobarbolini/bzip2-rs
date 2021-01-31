# bzip2-rs

[![crates.io](https://img.shields.io/crates/v/bzip2-rs.svg)](https://crates.io/crates/bzip2-rs)
[![Documentation](https://docs.rs/bzip2-rs/badge.svg)](https://docs.rs/bzip2-rs)
[![dependency status](https://deps.rs/crate/bzip2-rs/0.1.0/status.svg)](https://deps.rs/crate/bzip2-rs/0.1.0)
[![Rustc Version 1.34.2+](https://img.shields.io/badge/rustc-1.34.2+-lightgray.svg)](https://blog.rust-lang.org/2019/04/11/Rust-1.34.0.html)
[![CI](https://github.com/paolobarbolini/bzip2-rs/workflows/CI/badge.svg)](https://github.com/paolobarbolini/bzip2-rs/actions?query=workflow%3ACI)

Pure Rust 100% safe bzip2 decompressor.

## Features

* No features are enabled by default, Rust >= 1.34.2 is supported
* `rustc_1_37`: enables Rust >= 1.37 optimizations
* `rustc_1_40`: enables Rust >= 1.40 optimizations
* `rustc_1_51`: enables Rust >= 1.51 optimizations

## Usage

```rust
use std::fs::File;
use std::io;
use bzip2_rs::DecoderReader;

let mut compressed_file = File::open("input.bz2")?;
let mut output = File::create("output")?;

let mut reader = DecoderReader::new(compressed_file);
io::copy(&mut reader, &mut output)?;
```

## Upcoming features

* parallel decoding support (similar to [pbzip2](https://github.com/cosnicolaou/pbzip2))
* bzip2 encoding support
* no_std support (is anybody interested with this?)

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

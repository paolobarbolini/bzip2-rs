# bzip2-rs

[![crates.io](https://img.shields.io/crates/v/bzip2-rs.svg)](https://crates.io/crates/bzip2-rs)
[![Documentation](https://docs.rs/bzip2-rs/badge.svg)](https://docs.rs/bzip2-rs)
[![dependency status](https://deps.rs/crate/bzip2-rs/0.1.2/status.svg)](https://deps.rs/crate/bzip2-rs/0.1.2)
[![Rustc Version 1.34.2+](https://img.shields.io/badge/rustc-1.34.2+-lightgray.svg)](https://blog.rust-lang.org/2019/04/11/Rust-1.34.0.html)
[![CI](https://github.com/paolobarbolini/bzip2-rs/workflows/CI/badge.svg)](https://github.com/paolobarbolini/bzip2-rs/actions?query=workflow%3ACI)

Pure Rust 100% safe bzip2 decompressor.

## Features

* `rayon`: enable using the [rayon] global threadpool for parallel decoding.
           NOTE: this feature is not subject to a MSRV. At the time of writing the MSRV for rayon is 1.56.0

* Default features: Rust >= 1.34.2 is supported
* `rustc_1_37`: bump MSRV to 1.37, enable more optimizations
* `rustc_1_55`: bump MSRV to 1.55, enable more optimizations
* `rustc_1_63`: bump MSRV to 1.63, enable more optimizations
* `nightly`: require Rust Nightly, enable more optimizations

## Usage

```rust
use std::fs::File;
use std::io;
use bzip2_rs::DecoderReader;

let mut compressed_file = File::open("input.bz2")?;
let mut decompressed_output = File::create("output")?;

let mut reader = DecoderReader::new(compressed_file);
io::copy(&mut reader, &mut decompressed_output)?;
```

## Upcoming features

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

[rayon]: https://crates.io/crates/rayon

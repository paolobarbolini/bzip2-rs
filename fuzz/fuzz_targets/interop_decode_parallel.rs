#![no_main]
use libfuzzer_sys::fuzz_target;

use std::io::{self,Read};

use bzip2::bufread::BzEncoder;
use bzip2::Compression;
use bzip2_rs::{ParallelDecoderReader,RayonThreadPool};

fuzz_target!(|data: &[u8]| {
    let mut encoder = BzEncoder::new(data, Compression::new(3));
    let mut compressed = Vec::new();
    encoder.read_to_end(&mut compressed).expect("reference encoder worked");

    let mut decoder = ParallelDecoderReader::new(compressed.as_slice(),RayonThreadPool,usize::MAX);
    let mut decompressed = Vec::new();
    io::copy(&mut decoder, &mut decompressed).expect("failed decompressing what the reference implementation compressed");

    assert_eq!(data, decompressed.as_slice());
});

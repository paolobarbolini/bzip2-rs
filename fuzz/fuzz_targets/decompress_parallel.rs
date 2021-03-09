#![no_main]
use libfuzzer_sys::fuzz_target;

use std::io;

use bzip2_rs::{ParallelDecoderReader,RayonThreadPool};

fuzz_target!(|data: &[u8]| {
    let mut decoder = ParallelDecoderReader::new(data,RayonThreadPool,usize::MAX);
    let _ = io::copy(&mut decoder, &mut io::sink());
});

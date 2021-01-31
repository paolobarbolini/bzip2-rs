#![no_main]
use libfuzzer_sys::fuzz_target;

use std::io;

use bzip2_rs::DecoderReader;

fuzz_target!(|data: &[u8]| {
    let mut decoder = DecoderReader::new(data);
    let _ = io::copy(&mut decoder, &mut io::sink());
});

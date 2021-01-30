use std::io::Read;

use pretty_assertions::assert_eq;

use bzip2_rs::decoder::DecoderReader;

#[test]
fn sample1() {
    let compressed = include_bytes!("samplefiles/sample1.bz2");
    let decompressed = include_bytes!("samplefiles/sample1.ref");

    let mut reader = DecoderReader::new(compressed.as_ref());

    let mut out = Vec::new();
    reader.read_to_end(&mut out).unwrap();

    assert_eq!(decompressed.len(), out.len());
    assert_eq!(decompressed.as_ref(), out.as_slice());
}

#[test]
fn sample2() {
    let compressed = include_bytes!("samplefiles/sample2.bz2");
    let decompressed = include_bytes!("samplefiles/sample2.ref");

    let mut reader = DecoderReader::new(compressed.as_ref());

    let mut out = Vec::new();
    reader.read_to_end(&mut out).unwrap();

    assert_eq!(decompressed.len(), out.len());
    assert_eq!(decompressed.as_ref(), out.as_slice());
}

#[test]
fn sample3() {
    let compressed = include_bytes!("samplefiles/sample3.bz2");
    let decompressed = include_bytes!("samplefiles/sample3.ref");

    let mut reader = DecoderReader::new(compressed.as_ref());

    let mut out = Vec::new();
    reader.read_to_end(&mut out).unwrap();

    assert_eq!(decompressed.len(), out.len());
    assert_eq!(decompressed.as_ref(), out.as_slice());
}

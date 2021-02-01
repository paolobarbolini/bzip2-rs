use std::io::{self, Read};

use pretty_assertions::assert_eq;

use bzip2_rs::decoder::DecoderReader;

#[test]
fn empty() {
    let compressed: &[u8] = &[];
    let mut reader = DecoderReader::new(compressed);

    let mut buf = [0; 1024];
    let err = reader.read(&mut buf).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
}

#[test]
fn trees_overflow() {
    // Reported in #1
    let compressed: &[u8] = &[
        66, 90, 104, 52, 49, 65, 89, 38, 83, 89, 1, 0, 0, 0, 91, 90, 66, 104, 0, 56, 50, 65, 175,
        229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229,
        229, 229, 229, 229, 229, 229, 229, 229, 27, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 229, 229,
        229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229, 229,
        229, 229, 229, 229, 229, 229, 229, 229, 118, 10, 233, 0, 122, 36,
    ];

    let mut reader = DecoderReader::new(compressed);

    let mut buf = [0; 1024];
    let err = reader.read(&mut buf).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::Other);
    assert_eq!(err.to_string(), "tree index too large");
}

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

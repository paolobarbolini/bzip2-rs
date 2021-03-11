use std::io::{self, Read};
use std::num::NonZeroUsize;
use std::thread;

#[cfg(feature = "rayon")]
use bzip2_rs::RayonThreadPool;
use bzip2_rs::{ParallelDecoderReader, ThreadPool};
use pretty_assertions::assert_eq;

#[cfg(not(feature = "rayon"))]
struct NaiveThreadPool;

#[cfg(not(feature = "rayon"))]
impl ThreadPool for NaiveThreadPool {
    fn spawn<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(func);
    }

    fn max_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new(4).unwrap()
    }
}

#[cfg(feature = "rayon")]
fn new_pool() -> RayonThreadPool {
    RayonThreadPool
}

#[cfg(not(feature = "rayon"))]
fn new_pool() -> NaiveThreadPool {
    NaiveThreadPool
}

#[test]
fn empty() {
    let compressed: &[u8] = &[];
    let mut reader = ParallelDecoderReader::new(compressed, new_pool(), usize::max_value());

    let mut buf = [0; 1024];
    let err = reader.read(&mut buf).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
}

#[test]
fn empty_stream() {
    let compressed: &[u8] = &[66, 90, 104, 51, 23, 114, 69, 56, 80, 144, 0, 0, 0, 0];
    let mut reader = ParallelDecoderReader::new(compressed, new_pool(), usize::max_value());

    let mut buf = [0; 1024];
    let read = reader.read(&mut buf).unwrap();
    assert_eq!(read, 0);
}

#[test]
fn sample1() {
    let compressed = include_bytes!("samplefiles/sample1.bz2");
    let decompressed = include_bytes!("samplefiles/sample1.ref");

    let mut reader =
        ParallelDecoderReader::new(compressed.as_ref(), new_pool(), usize::max_value());

    let mut out = Vec::new();
    reader.read_to_end(&mut out).unwrap();

    assert_eq!(decompressed.len(), out.len());
    assert_eq!(decompressed.as_ref(), out.as_slice());
}

#[test]
fn sample2() {
    let compressed = include_bytes!("samplefiles/sample2.bz2");
    let decompressed = include_bytes!("samplefiles/sample2.ref");

    let mut reader =
        ParallelDecoderReader::new(compressed.as_ref(), new_pool(), usize::max_value());

    let mut out = Vec::new();
    reader.read_to_end(&mut out).unwrap();

    assert_eq!(decompressed.len(), out.len());
    assert_eq!(decompressed.as_ref(), out.as_slice());
}

#[test]
fn sample3() {
    let compressed = include_bytes!("samplefiles/sample3.bz2");
    let decompressed = include_bytes!("samplefiles/sample3.ref");

    let mut reader =
        ParallelDecoderReader::new(compressed.as_ref(), new_pool(), usize::max_value());

    let mut out = Vec::new();
    reader.read_to_end(&mut out).unwrap();

    assert_eq!(decompressed.len(), out.len());
    assert_eq!(decompressed.as_ref(), out.as_slice());
}

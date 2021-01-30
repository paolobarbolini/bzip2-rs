use std::fs;
use std::io::Read;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bzip2::read::BzDecoder;
use bzip2_rs::decoder::DecoderReader;

fn bench_decode(c: &mut Criterion) {
    let compressed = fs::read("tests/samplefiles/sample2.bz2").unwrap();
    let decompressed = fs::read("tests/samplefiles/sample2.ref").unwrap();

    let compressed: &[u8] = compressed.as_ref();
    let decompressed: &[u8] = decompressed.as_ref();

    c.bench_function("decode rust", move |b| {
        b.iter(|| {
            let compressed = black_box(compressed);

            let mut decoder = DecoderReader::new(compressed);

            let mut out = Vec::with_capacity(decompressed.len());
            decoder.read_to_end(&mut out).unwrap();

            let decompressed = black_box(decompressed);
            assert_eq!(decompressed, out.as_slice());
        })
    });

    c.bench_function("decode c", move |b| {
        b.iter(|| {
            let compressed = black_box(compressed);

            let mut decoder = BzDecoder::new(compressed);

            let mut out = Vec::with_capacity(decompressed.len());
            decoder.read_to_end(&mut out).unwrap();

            let decompressed = black_box(decompressed);
            assert_eq!(decompressed, out.as_slice());
        })
    });
}

criterion_group!(benches, bench_decode);
criterion_main!(benches);

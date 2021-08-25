//! bzip2 low-level block APIs

pub use self::decoder::Decoder;
pub use self::error::BlockError;
pub use self::reader::Reader;

pub(crate) const BLOCK_MAGIC: u64 = 0x314159265359;
pub(crate) const FINAL_MAGIC: u64 = 0x177245385090;

mod bwt;
mod decoder;
mod error;
mod reader;

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::Decoder as BlockDecoder;
    use crate::bitreader::BitReader;
    use crate::header::Header;

    #[test]
    fn decode1() {
        let compressed = include_bytes!("../../../tests/samplefiles/sample1.bz2");
        let decompressed = include_bytes!("../../../tests/samplefiles/sample1.ref");

        let header = Header::parse(compressed[..4].try_into().unwrap()).unwrap();
        println!("block_size: {}", header.raw_blocksize());

        let compressed = &compressed[4..];

        let mut bits = BitReader::new(compressed, 0);
        let mut decoder = BlockDecoder::new(header);
        let mut reader = decoder.decode(&mut bits).unwrap().unwrap();

        let mut out = vec![0u8; decompressed.len()];

        let read = reader.read(&mut out);
        assert_eq!(&out[..read], decompressed.as_ref());
    }

    #[test]
    fn decode2() {
        let compressed = include_bytes!("../../../tests/samplefiles/sample2.bz2");
        let decompressed = include_bytes!("../../../tests/samplefiles/sample2.ref");

        let header = Header::parse(compressed[..4].try_into().unwrap()).unwrap();
        println!("block_size: {}", header.raw_blocksize());

        let compressed = &compressed[4..];

        let mut bits = BitReader::new(compressed, 0);
        let  decoder = BlockDecoder::new(header);
        let mut reader = decoder.decode(&mut bits).unwrap().unwrap();

        let mut out = vec![0u8; decompressed.len()];

        let read1 = reader.read(&mut out);
        let  decoder = reader.recycle();
        let mut reader = decoder.decode(&mut bits).unwrap().unwrap();
        let read2 = reader.read(&mut out[read1..]);
        assert_eq!(&out[..read1 + read2], decompressed.as_ref());
    }

    #[test]
    fn decode3() {
        let compressed = include_bytes!("../../../tests/samplefiles/sample3.bz2");
        let decompressed = include_bytes!("../../../tests/samplefiles/sample3.ref");

        let header = Header::parse(compressed[..4].try_into().unwrap()).unwrap();
        println!("block_size: {}", header.raw_blocksize());

        let compressed = &compressed[4..];

        let mut bits = BitReader::new(compressed, 0);
        let mut decoder = BlockDecoder::new(header);
        let mut reader = decoder.decode(&mut bits).unwrap().unwrap();

        let mut out = vec![0u8; decompressed.len()];

        let read = reader.read(&mut out);
        assert_eq!(&out[..read], decompressed.as_ref());
    }
}

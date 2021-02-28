//! bzip2 low-level block APIs

use std::mem;

use tinyvec::ArrayVec;

pub use self::error::BlockError;
use crate::bitreader::BitReader;
#[cfg(target_pointer_width = "64")]
use crate::bitreader::CachedBitReader;
use crate::crc::Hasher;
use crate::header::Header;
use crate::huffman::HuffmanTree;
use crate::move_to_front::MoveToFrontDecoder;

mod bwt;
mod error;

const BLOCK_MAGIC: u64 = 0x314159265359;
const FINAL_MAGIC: u64 = 0x177245385090;

pub(crate) struct Block {
    header: Header,

    tt: Vec<u32>,
    pre_rle_used: u32,
    t_pos: u32,
    last_byte: i16,
    byte_repeats: u8,
    repeats: u8,

    hasher: Hasher,
    expected_crc: u32,
}

impl Block {
    pub fn new(header: Header) -> Self {
        let max_blocksize = header.max_blocksize();

        Self {
            header,

            tt: Vec::with_capacity(max_blocksize as usize),
            pre_rle_used: 0,
            t_pos: 0,
            last_byte: -1,
            byte_repeats: 0,
            repeats: 0,

            hasher: Hasher::new(),
            expected_crc: 0,
        }
    }

    pub fn read_block(&mut self, reader: &mut BitReader<'_>) -> Result<Option<()>, BlockError> {
        let magic = reader
            .read_u64(48)
            .ok_or_else(|| BlockError::new("next magic truncated"))?;
        match magic {
            BLOCK_MAGIC => {
                self.do_read_block(reader)?;
                Ok(Some(()))
            }
            FINAL_MAGIC => {
                let _crc = reader
                    .read_u32(32)
                    .ok_or_else(|| BlockError::new("whole stream crc truncated"))?;

                // TODO: check whole stream crc

                Ok(None)
            }
            _ => Err(BlockError::new("bad magic value found")),
        }
    }

    pub fn read_from_block(&mut self, out: &mut [u8]) -> usize {
        let mut read = 0;

        while (self.repeats > 0 || self.pre_rle_used < (self.tt.len() as u32)) && read < out.len() {
            if self.repeats > 0 {
                out[read] = self.last_byte as u8;
                read += 1;

                self.repeats -= 1;
                if self.repeats == 0 {
                    self.last_byte = -1;
                }
                continue;
            }

            self.t_pos = self.tt[self.t_pos as usize];
            let b = self.t_pos as u8;
            self.t_pos >>= 8;
            self.pre_rle_used += 1;

            if self.byte_repeats == 3 {
                self.repeats = b;
                self.byte_repeats = 0;
                continue;
            }

            if self.last_byte == i16::from(b) {
                self.byte_repeats += 1;
            } else {
                self.byte_repeats = 0;
            }
            self.last_byte = i16::from(b);

            out[read] = b;
            read += 1;
        }

        self.hasher.update(&out[..read]);
        read
    }

    pub fn check_crc(&self) -> Result<(), BlockError> {
        let crc = self.hasher.finalize();
        if self.expected_crc == crc {
            Ok(())
        } else {
            Err(BlockError::new("bad crc"))
        }
    }

    fn do_read_block(&mut self, reader: &mut BitReader<'_>) -> Result<(), BlockError> {
        self.hasher = Hasher::new();
        self.tt.clear();

        self.expected_crc = reader
            .read_u32(32)
            .ok_or_else(|| BlockError::new("crc truncated"))?;

        let randomised = reader
            .read_bool()
            .ok_or_else(|| BlockError::new("randomised truncated"))?;
        if randomised {
            return Err(BlockError::new("randomised expected to be 'normal'"));
        }

        let orig_ptr = reader
            .read_u32(24)
            .ok_or_else(|| BlockError::new("orig ptr truncated"))?;

        let mut huffman_used_symbols = ArrayVec::<[u8; 16]>::new();
        let mut huffman_used_bitmaps = ArrayVec::<[u8; 256]>::new();

        for i in 0..16 {
            if reader
                .read_bool()
                .ok_or_else(|| BlockError::new("symbol range truncated"))?
            {
                huffman_used_symbols.push(i);
            }
        }

        for symbol_range in huffman_used_symbols {
            for symbol in 0..16 {
                if reader
                    .read_bool()
                    .ok_or_else(|| BlockError::new("symbol range truncated"))?
                {
                    huffman_used_bitmaps.push(symbol_range * 16 + symbol);
                }
            }
        }

        if huffman_used_bitmaps.is_empty() {
            return Err(BlockError::new("huffman no symbols in input"));
        }

        let huffman_groups = reader
            .read_u8(3)
            .ok_or_else(|| BlockError::new("huffmann groups truncated"))?;
        if huffman_groups < 2 || huffman_groups > 6 {
            return Err(BlockError::new("invalid number of huffman trees"));
        }

        let selectors_used = reader
            .read_u16(15)
            .ok_or_else(|| BlockError::new("selectors used truncated"))?;

        #[cfg(feature = "nightly")]
        let mut selectors_list = ArrayVec::<[u8; 18001]>::new();
        #[cfg(feature = "nightly")]
        selectors_list.set_len(usize::from(selectors_used));
        #[cfg(not(feature = "nightly"))]
        let mut selectors_list = vec![0u8; usize::from(selectors_used)];

        let mut move_to_front_decoder = MoveToFrontDecoder::new();
        for selector_item in selectors_list.iter_mut().rev() {
            let mut trees = 0;

            while reader
                .read_bool()
                .ok_or_else(|| BlockError::new("selector truncated"))?
            {
                trees += 1;

                if trees >= huffman_groups {
                    return Err(BlockError::new("tree index too large"));
                }
            }

            *selector_item = move_to_front_decoder.decode(trees);
        }

        // limited in lenght of huffman_used_symbols
        let mut symbols = [0u8; 256];
        symbols[..huffman_used_bitmaps.len()].copy_from_slice(&huffman_used_bitmaps);
        let mut move_to_front_decoder_2 = MoveToFrontDecoder::new_from_symbols(symbols);

        let mut huffman_trees = ArrayVec::<[HuffmanTree; 6]>::new();

        let mut lengths = ArrayVec::<[u8; crate::LEN_258]>::new();
        lengths.set_len(huffman_used_bitmaps.len() + 2);

        for _ in 0..huffman_groups {
            let mut length = reader
                .read_u8(5)
                .ok_or_else(|| BlockError::new("huffman group length truncated"))?;

            for length_item in &mut *lengths {
                loop {
                    if length < 1 || length > 20 {
                        return Err(BlockError::new("huffman length out of range"));
                    }

                    if !reader
                        .read_bool()
                        .ok_or_else(|| BlockError::new("length bit1 truncated"))?
                    {
                        break;
                    }

                    if reader
                        .read_bool()
                        .ok_or_else(|| BlockError::new("length bit2 truncated"))?
                    {
                        length -= 1;
                    } else {
                        length += 1;
                    }
                }

                *length_item = length;
            }

            let tree = HuffmanTree::new(&lengths).map_err(BlockError::new)?;
            huffman_trees.push(tree);
        }

        let selector = selectors_list
            .pop()
            .ok_or_else(|| BlockError::new("no tree selectors given"))?;
        let mut current_huffman_tree = huffman_trees
            .get(usize::from(selector))
            .ok_or_else(|| BlockError::new("tree selector out of range"))?;

        let mut repeat = 0u32;
        let mut repeat_power = 0u32;
        let mut c = [0u32; 256];

        let mut decoded = 0;
        #[cfg(target_pointer_width = "64")]
        let mut r = CachedBitReader::new(reader)
            .ok_or_else(|| BlockError::new("huffman bitstream truncated"))?;
        loop {
            if decoded == 50 {
                let selector = selectors_list.pop().ok_or_else(|| {
                    BlockError::new("insufficient selector indices for number of symbols")
                })?;

                current_huffman_tree = huffman_trees
                    .get(usize::from(selector))
                    .ok_or_else(|| BlockError::new("tree selector out of range"))?;
                decoded = 0;
            }

            #[cfg(target_pointer_width = "64")]
            let v = {
                let read = r.read();
                let v = current_huffman_tree.decode(&mut r);

                if r.overflowed() {
                    r.restore(reader, read);
                    r.refresh(reader)
                        .ok_or_else(|| BlockError::new("huffman bitstream truncated"))?;

                    current_huffman_tree.decode(&mut r)
                } else {
                    v
                }
            };
            #[cfg(not(target_pointer_width = "64"))]
            let v = current_huffman_tree.decode(reader);

            let v = v.ok_or_else(|| BlockError::new("huffman bitstream truncated"))?;
            decoded += 1;

            if v < 2 {
                if repeat == 0 {
                    repeat_power = 1;
                }
                repeat += repeat_power << v;
                repeat_power <<= 1;

                if repeat > 2 * 1024 * 1024 {
                    return Err(BlockError::new("repeat count too large"));
                }
                continue;
            }

            let old_repeat = mem::replace(&mut repeat, 0);
            if old_repeat > 0 {
                if old_repeat > self.header.max_blocksize() - (self.tt.len() as u32) {
                    return Err(BlockError::new("repeats past end of block"));
                }

                let b = move_to_front_decoder_2.first();
                // extend self.tt with `b` repeated `old_repeat` times
                let new_len = self.tt.len() + old_repeat as usize;
                self.tt.resize(new_len, u32::from(b));
                c[usize::from(b)] += old_repeat;
            }

            if usize::from(v) == (huffman_used_bitmaps.len() + 2) - 1 {
                break;
            }

            let b = move_to_front_decoder_2.decode((v - 1) as u8);
            if self.tt.len() >= self.header.max_blocksize() as usize {
                return Err(BlockError::new("data exceeds block size"));
            }

            self.tt.push(u32::from(b));
            c[usize::from(b)] += 1;
        }
        #[cfg(target_pointer_width = "64")]
        r.restore(reader, r.read());

        if (orig_ptr as usize) >= self.tt.len() {
            return Err(BlockError::new("orig_ptr out of bounds"));
        }

        self.pre_rle_used = 0;
        self.t_pos = bwt::inverse_bwt(&mut self.tt, orig_ptr as usize, c);
        self.last_byte = -1;
        self.byte_repeats = 0;
        self.repeats = 0;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn decode1() {
        let compressed = include_bytes!("../../../tests/samplefiles/sample1.bz2");
        let decompressed = include_bytes!("../../../tests/samplefiles/sample1.ref");

        let header = Header::parse(compressed[..4].try_into().unwrap()).unwrap();
        println!("block_size: {}", header.raw_blocksize());

        let compressed = &compressed[4..];

        let mut bits = BitReader::new(&compressed);
        let mut reader = Block::new(header);

        reader.read_block(&mut bits).unwrap();

        let mut out = vec![0u8; decompressed.len() * 2];
        let read = reader.read_from_block(&mut out);
        assert!(reader.check_crc().is_ok());

        assert_eq!(reader.read_block(&mut bits), Ok(None));

        assert_eq!(&out[..read], decompressed.as_ref());
    }

    #[test]
    fn decode2() {
        let compressed = include_bytes!("../../../tests/samplefiles/sample2.bz2");
        let decompressed = include_bytes!("../../../tests/samplefiles/sample2.ref");

        let header = Header::parse(compressed[..4].try_into().unwrap()).unwrap();
        println!("block_size: {}", header.raw_blocksize());

        let compressed = &compressed[4..];

        let mut bits = BitReader::new(&compressed);
        let mut reader = Block::new(header);

        reader.read_block(&mut bits).unwrap();

        let mut out = vec![0u8; decompressed.len() * 2];
        let read1 = reader.read_from_block(&mut out);
        assert!(reader.check_crc().is_ok());

        reader.read_block(&mut bits).unwrap();

        let read2 = reader.read_from_block(&mut out[read1..]);
        assert!(reader.check_crc().is_ok());

        assert_eq!(reader.read_block(&mut bits), Ok(None));

        assert_eq!(&out[..read1 + read2], decompressed.as_ref());
    }

    #[test]
    fn decode3() {
        let compressed = include_bytes!("../../../tests/samplefiles/sample3.bz2");
        let decompressed = include_bytes!("../../../tests/samplefiles/sample3.ref");

        let header = Header::parse(compressed[..4].try_into().unwrap()).unwrap();
        println!("block_size: {}", header.raw_blocksize());

        let compressed = &compressed[4..];

        let mut bits = BitReader::new(&compressed);
        let mut reader = Block::new(header);

        reader.read_block(&mut bits).unwrap();

        let mut out = vec![0u8; decompressed.len() * 2];
        let read = reader.read_from_block(&mut out);
        assert!(reader.check_crc().is_ok());

        assert_eq!(reader.read_block(&mut bits), Ok(None));

        assert_eq!(&out[..read], decompressed.as_ref());
    }
}
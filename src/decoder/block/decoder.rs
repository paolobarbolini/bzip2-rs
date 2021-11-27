//! bzip2 low-level block APIs

use std::mem;

use tinyvec::ArrayVec;

use super::{bwt, BlockError, Reader, BLOCK_MAGIC, FINAL_MAGIC};
use crate::bitreader::BitReader;
#[cfg(target_pointer_width = "64")]
use crate::bitreader::CachedBitReader;
use crate::header::Header;
use crate::huffman::HuffmanTree;
use crate::move_to_front::MoveToFrontDecoder;

pub struct Decoder {
    header: Header,

    tt: Vec<u32>,
}

impl Decoder {
    pub fn new(header: Header) -> Self {
        let max_blocksize = header.max_blocksize();

        Self {
            header,

            tt: Vec::with_capacity(max_blocksize as usize),
        }
    }

    pub(crate) fn header(&self) -> &Header {
        &self.header
    }

    pub(super) fn recycle_from(header: Header, mut tt: Vec<u32>) -> Self {
        tt.clear();
        Self { header, tt }
    }

    pub fn decode(self, reader: &mut BitReader<'_>) -> Result<Option<Reader>, BlockError> {
        let magic = reader
            .read_u64(48)
            .ok_or_else(|| BlockError::new("next magic truncated"))?;
        match magic {
            BLOCK_MAGIC => {
                let reader = self.do_decode(reader)?;
                Ok(Some(reader))
            }
            FINAL_MAGIC => {
                let _crc = reader
                    .read_u32(32)
                    .ok_or_else(|| BlockError::new("whole stream crc truncated"))?;

                // TODO: check whole stream crc

                while reader.bit_position() % 8 != 0 {
                    let _ = reader
                        .next()
                        .expect("reader should always be able to read bits");
                }

                Ok(None)
            }
            _ => Err(BlockError::new("bad magic value found")),
        }
    }

    fn do_decode(mut self, reader: &mut BitReader<'_>) -> Result<Reader, BlockError> {
        let expected_crc = reader
            .read_u32(32)
            .ok_or_else(|| BlockError::new("crc truncated"))?;

        let randomised = reader
            .next()
            .ok_or_else(|| BlockError::new("randomised truncated"))?;
        if randomised {
            return Err(BlockError::new("randomised expected to be 'normal'"));
        }

        let orig_ptr = reader
            .read_u32(24)
            .ok_or_else(|| BlockError::new("orig ptr truncated"))?;

        let (mut huffman_decoder, alpha_size) =
            MoveToFrontDecoder::read_from_block(reader).map_err(BlockError::new)?;

        let huffman_groups = reader
            .read_u8(3)
            .ok_or_else(|| BlockError::new("huffmann groups truncated"))?;
        if huffman_groups < 2 || huffman_groups > 6 {
            return Err(BlockError::new("invalid number of huffman trees"));
        }

        let num_selectors = reader
            .read_u16(15)
            .ok_or_else(|| BlockError::new("selectors used truncated"))?;

        #[cfg(feature = "nightly")]
        let mut reverse_selectors = ArrayVec::<[u8; 18001]>::new();
        #[cfg(feature = "nightly")]
        {
            if num_selectors > 18001 {
                return Err(BlockError::new("too high value for num_selectors"));
            }
            reverse_selectors.set_len(usize::from(num_selectors));
        }
        #[cfg(not(feature = "nightly"))]
        let mut reverse_selectors = vec![0u8; usize::from(num_selectors)];

        let mut selectors_decoder = MoveToFrontDecoder::new();
        for selector in reverse_selectors.iter_mut().rev() {
            let mut trees = 0;

            while reader
                .next()
                .ok_or_else(|| BlockError::new("selector truncated"))?
            {
                trees += 1;

                if trees >= huffman_groups {
                    return Err(BlockError::new("tree index too large"));
                }
            }

            *selector = selectors_decoder.decode(trees);
        }

        let mut huffman_trees = ArrayVec::<[HuffmanTree; 6]>::new();

        let mut lengths = ArrayVec::<[u8; crate::LEN_258]>::new();
        lengths.set_len(alpha_size);

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
                        .next()
                        .ok_or_else(|| BlockError::new("length bit1 truncated"))?
                    {
                        break;
                    }

                    if reader
                        .next()
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

        let selector = reverse_selectors
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
                let selector = reverse_selectors.pop().ok_or_else(|| {
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

                let b = huffman_decoder.first();
                // extend self.tt with `b` repeated `old_repeat` times
                let new_len = self.tt.len() + old_repeat as usize;
                self.tt.resize(new_len, u32::from(b));
                c[usize::from(b)] += old_repeat;
            }

            if usize::from(v) == (alpha_size) - 1 {
                break;
            }

            let b = huffman_decoder.decode((v - 1) as u8);
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

        let t_pos = bwt::inverse_bwt(&mut self.tt, orig_ptr as usize, c);
        Ok(Reader::from_decoder(
            self.tt,
            t_pos,
            expected_crc,
            self.header,
        ))
    }
}

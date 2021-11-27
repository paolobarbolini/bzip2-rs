//! bzip2 low-level block APIs

use super::{BlockError, Decoder};
use crate::crc::Hasher;
use crate::header::Header;

pub struct Reader {
    tt: Vec<u32>,
    pre_rle_used: u32,
    t_pos: u32,
    last_byte: i16,
    byte_repeats: u8,
    repeats: u8,

    hasher: Hasher,
    expected_crc: u32,

    // needed for recycling
    header: Header,
}

impl Reader {
    pub(super) fn from_decoder(
        tt: Vec<u32>,
        t_pos: u32,
        expected_crc: u32,
        header: Header,
    ) -> Self {
        Self {
            tt,
            pre_rle_used: 0,
            t_pos,
            last_byte: -1,
            byte_repeats: 0,
            repeats: 0,

            hasher: Hasher::new(),
            expected_crc,

            header,
        }
    }

    pub fn read(&mut self, out: &mut [u8]) -> usize {
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

    pub fn recycle(self) -> Decoder {
        Decoder::recycle_from(self.header, self.tt)
    }
}

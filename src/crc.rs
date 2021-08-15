pub struct Hasher {
    // CRC32B hasher
    val: crc32fast::Hasher,
    // reversed bits
    bytes: [u8; 512],
}

impl Hasher {
    pub fn new() -> Self {
        Self {
            val: crc32fast::Hasher::new(),
            bytes: [0; 512],
        }
    }

    pub fn update(&mut self, buf: &[u8]) {
        for chunk in buf.chunks(self.bytes.len()) {
            #[allow(clippy::needless_range_loop)]
            for i in 0..chunk.len() {
                let mut byte = chunk[i];

                #[cfg(feature = "rustc_1_37")]
                {
                    byte = byte.reverse_bits();
                }

                #[cfg(not(feature = "rustc_1_37"))]
                {
                    byte = (byte & 0xF0) >> 4 | (byte & 0x0F) << 4;
                    byte = (byte & 0xCC) >> 2 | (byte & 0x33) << 2;
                    byte = (byte & 0xAA) >> 1 | (byte & 0x55) << 1;
                }

                self.bytes[i] = byte;
            }

            self.val.update(&self.bytes[..chunk.len()]);
        }
    }

    pub fn finalize(&self) -> u32 {
        let reversed = self.val.clone().finalize();
        // CRC32B to CRC32
        #[cfg(feature = "rustc_1_37")]
        {
            reversed.reverse_bits()
        }
        #[cfg(not(feature = "rustc_1_37"))]
        {
            let mut reversed = reversed;
            reversed = (reversed >> 1) & 0x55555555 | (reversed << 1) & 0xaaaaaaaa;
            reversed = (reversed >> 2) & 0x33333333 | (reversed << 2) & 0xcccccccc;
            reversed = (reversed >> 4) & 0x0f0f0f0f | (reversed << 4) & 0xf0f0f0f0;
            reversed = (reversed >> 8) & 0x00ff00ff | (reversed << 8) & 0xff00ff00;
            (reversed >> 16) & 0x0000ffff | (reversed << 16) & 0xffff0000
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc() {
        let mut hasher = Hasher::new();
        hasher.update(b"123456789");
        assert_eq!(hasher.finalize(), 0xFC891918);
    }
}

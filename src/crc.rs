#[cfg(feature = "nightly")]
use std::mem::MaybeUninit;

pub struct Hasher {
    // CRC32B hasher
    val: crc32fast::Hasher,
    // reversed bits
    #[cfg(not(feature = "nightly"))]
    bytes: [u8; 512],
}

impl Hasher {
    pub fn new() -> Self {
        Self {
            val: crc32fast::Hasher::new(),
            #[cfg(not(feature = "nightly"))]
            bytes: [0; 512],
        }
    }

    pub fn update(&mut self, mut buf: &[u8]) {
        #[cfg(feature = "nightly")]
        let mut bytes = [MaybeUninit::<u8>::uninit(); 512];
        #[cfg(not(feature = "nightly"))]
        let bytes = &mut self.bytes;

        while !buf.is_empty() {
            let len = buf.len().min(bytes.len());
            #[cfg(feature = "nightly")]
            let bytes = MaybeUninit::write_slice(&mut bytes[..len], &buf[..len]);
            #[cfg(not(feature = "nightly"))]
            bytes[..len].copy_from_slice(&buf[..len]);
            buf = &buf[len..];

            for byte in bytes.iter_mut() {
                #[cfg(feature = "rustc_1_37")]
                {
                    *byte = byte.reverse_bits();
                }

                #[cfg(not(feature = "rustc_1_37"))]
                {
                    *byte = (*byte & 0xF0) >> 4 | (*byte & 0x0F) << 4;
                    *byte = (*byte & 0xCC) >> 2 | (*byte & 0x33) << 2;
                    *byte = (*byte & 0xAA) >> 1 | (*byte & 0x55) << 1;
                }
            }
            self.val.update(&bytes[..len]);
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

    pub fn reset(&mut self) {
        self.val.reset();
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

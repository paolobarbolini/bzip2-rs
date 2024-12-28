pub struct Hasher {
    // CRC32B hasher
    val: crc32fast::Hasher,
}

impl Hasher {
    pub fn new() -> Self {
        Self {
            val: crc32fast::Hasher::new(),
        }
    }

    pub fn update(&mut self, buf: &[u8]) {
        let mut bytes = [0u8; 512];
        let mut chunks = buf.chunks_exact(bytes.len());

        for full_chunk in &mut chunks {
            #[allow(clippy::needless_range_loop)]
            for i in 0..full_chunk.len() {
                bytes[i] = full_chunk[i].reverse_bits();
            }

            self.val.update(&bytes);
        }

        let chunk = chunks.remainder();
        #[allow(clippy::needless_range_loop)]
        for i in 0..chunk.len() {
            bytes[i] = chunk[i].reverse_bits();
        }

        self.val.update(&bytes[..chunk.len()]);
    }

    pub fn finalize(&self) -> u32 {
        let reversed = self.val.clone().finalize();
        // CRC32B to CRC32
        reversed.reverse_bits()
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

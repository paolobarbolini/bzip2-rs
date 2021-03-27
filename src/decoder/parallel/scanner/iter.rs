use super::linear::find_next_signature;

/// An iterator returning the bit offsets into `buf` to `BLOCK_MAGIC`
pub struct SignatureFinder<'a> {
    buf: &'a [u8],
    skip: u64,
}

impl<'a> SignatureFinder<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, skip: 0 }
    }
}

impl<'a> Iterator for SignatureFinder<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let skip = self.skip / 8;
        let buf = self.buf.get((skip as usize)..)?;
        let raw_signature = find_next_signature(buf)?;

        // take into account the skipped bits
        let signature = skip * 8 + raw_signature;

        // skip this signature in the next iteration
        self.skip = signature + 48;

        Some(signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitreader::BitReader;
    use crate::decoder::block::BLOCK_MAGIC;

    #[test]
    fn find_at_any_offsets() {
        for shift in 0..=80 {
            let mut haystack = vec![0u8; 1024];
            let shifted = u128::from(BLOCK_MAGIC) << shift;
            let shifted = u128::to_be_bytes(shifted);
            haystack.extend_from_slice(&shifted);
            haystack.resize(haystack.len() + 1024, 0);

            let mut repeated_haystack = Vec::new();
            for _ in 0..16 {
                repeated_haystack.extend_from_slice(&haystack);
            }

            let mut finder = SignatureFinder::new(&repeated_haystack);

            for i in 0..16 {
                let pos = finder.next().unwrap();

                let mut reader = BitReader::new(&repeated_haystack);
                assert!(reader.advance_by(pos as usize));

                let magic = reader.read_u64(48).unwrap();
                assert_eq!(BLOCK_MAGIC, magic);
            }

            assert!(finder.next().is_none());
        }
    }
}

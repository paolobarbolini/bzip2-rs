pub struct BitReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> BitReader<'a> {
        BitReader { bytes, position: 0 }
    }

    pub fn read_u8(&mut self, bit_count: u8) -> Option<u8> {
        debug_assert!(bit_count <= 8);

        if (self.position + usize::from(bit_count)) > self.bytes.len() * 8 {
            return None;
        }

        let mut value = 0;
        for bit in self.take(bit_count as usize) {
            value = (value << 1) | u8::from(bit);
        }
        Some(value)
    }

    pub fn read_u16(&mut self, bit_count: u8) -> Option<u16> {
        debug_assert!(bit_count <= 16);

        if (self.position + usize::from(bit_count)) > self.bytes.len() * 8 {
            return None;
        }

        let mut value = 0;
        for bit in self.take(bit_count as usize) {
            value = (value << 1) | u16::from(bit);
        }
        Some(value)
    }

    pub fn read_u32(&mut self, bit_count: u8) -> Option<u32> {
        debug_assert!(bit_count <= 32);

        if (self.position + usize::from(bit_count)) > self.bytes.len() * 8 {
            return None;
        }

        let mut value = 0;
        for bit in self.take(bit_count as usize) {
            value = (value << 1) | u32::from(bit);
        }
        Some(value)
    }

    pub fn read_u64(&mut self, bit_count: u8) -> Option<u64> {
        debug_assert!(bit_count <= 64);

        if (self.position + usize::from(bit_count)) > self.bytes.len() * 8 {
            return None;
        }

        let mut value = 0;
        for bit in self.take(bit_count as usize) {
            value = (value << 1) | u64::from(bit);
        }
        Some(value)
    }

    pub fn read_bool(&mut self) -> Option<bool> {
        self.next()
    }

    /// Skip arbitrary number of bits. However, you can skip at most to the end of the byte slice.
    pub fn advance_by(&mut self, bit_count: usize) -> bool {
        let end_position = self.position + bit_count;
        if end_position > self.bytes.len() * 8 {
            return false;
        }
        self.position = end_position;
        true
    }

    /// Returns the position of the cursor, or how many bits have been read so far.
    pub fn position(&self) -> usize {
        self.position
    }
}

impl<'a> Iterator for BitReader<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.position;
        let byte = *self.bytes.get(position / 8)?;

        self.position += 1;

        let bit = byte << (position % 8);
        Some(bit & 0b1000_0000 != 0)
    }
}

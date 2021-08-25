use std::convert::TryInto;

pub struct BitReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8], position: usize) -> BitReader<'a> {
        BitReader { bytes, position }
    }

    pub fn read_u8(&mut self, bit_count: u8) -> Option<u8> {
        debug_assert!(bit_count <= 8);

        if (self.position + usize::from(bit_count)) > self.bytes.len() * 8 {
            return None;
        }

        let mut value = 0;
        for bit in self.take(usize::from(bit_count)) {
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
        for bit in self.take(usize::from(bit_count)) {
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
        for bit in self.take(usize::from(bit_count)) {
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
        for bit in self.take(usize::from(bit_count)) {
            value = (value << 1) | u64::from(bit);
        }
        Some(value)
    }

    /// Returns the position of the cursor, or how many bits have been read so far.
    pub fn bit_position(&self) -> usize {
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

/// A bitreader which can read at most 64 bits, but is faster on systems
/// with 64 bit registers.
/// NOTE: a few points are assumed by this bitreader
///
/// * `<CachedBitReader as Iterator>::next` will never yield `None`, even after
///   the guaranteed to be valid 64 bits have been exhausted.
/// * The soft limit on 64 bits can be reset by calling `CachedBitReader::refresh`
/// * At least 9 bytes must be available in the `BitReader` passed to `CachedBitReader::refresh`
/// * If `<CachedBitReader as Iterator>::next` may have been called at least one time,
///   `CachedBitReader::restore` must be called in order to update the `BitReader::position`
/// * `CachedBitReader::restore` must be called before calling `CachedBitReader::refresh`,
///   unless `CachedBitReader::read() == 0`
#[cfg(target_pointer_width = "64")]
pub struct CachedBitReader {
    cache: u64,
    read: usize,
}

#[cfg(target_pointer_width = "64")]
impl CachedBitReader {
    pub fn new(reader: &BitReader<'_>) -> Option<Self> {
        let mut this = Self { cache: 0, read: 0 };
        this.refresh(reader)?;
        Some(this)
    }

    pub fn refresh(&mut self, reader: &BitReader<'_>) -> Option<()> {
        let pos = reader.position / 8;
        let data = reader.bytes.get(pos..pos + 9)?;

        let mut b1 = u64::from_be_bytes(data[..8].try_into().unwrap());
        b1 <<= reader.position % 8;

        let mut b2 = u16::from(data[8]);
        b2 <<= reader.position % 8;
        b2 >>= 8;

        self.cache = b1 | u64::from(b2);
        self.read = 0;
        Some(())
    }

    pub fn read(&self) -> usize {
        self.read
    }

    pub fn overflowed(&self) -> bool {
        self.read() > 64
    }

    pub fn restore(&mut self, reader: &mut BitReader<'_>, read: usize) {
        reader.position += read;
    }
}

#[cfg(target_pointer_width = "64")]
impl Iterator for CachedBitReader {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        // read the left most bit
        let bit = self.cache & !(u64::max_value() >> 1);

        self.read += 1;
        self.cache <<= 1;

        Some(bit != 0)
    }
}

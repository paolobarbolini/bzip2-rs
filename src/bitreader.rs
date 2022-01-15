use std::convert::TryInto;
use std::mem;

const USIZE_BYTES: usize = mem::size_of::<usize>();
const USIZE_BITS: usize = USIZE_BYTES * 8;

pub struct BitReader<'a> {
    bytes: &'a [u8],

    bits: usize,
    remaining_bits: u8,

    read_bits: u32,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> BitReader<'a> {
        BitReader {
            bytes,
            bits: 0,
            remaining_bits: 0,
            read_bits: 0,
        }
    }

    pub fn read_u8(&mut self, bit_count: u8) -> Option<u8> {
        debug_assert!(bit_count <= 8);

        let mut read_bits = 0u8;
        let mut value = 0;
        for bit in self.take(usize::from(bit_count)) {
            value = (value << 1) | u8::from(bit);
            read_bits += 1;
        }

        if read_bits < bit_count {
            None
        } else {
            Some(value)
        }
    }

    pub fn read_u16(&mut self, bit_count: u8) -> Option<u16> {
        debug_assert!(bit_count <= 16);

        let mut read_bits = 0u8;
        let mut value = 0;
        for bit in self.take(usize::from(bit_count)) {
            value = (value << 1) | u16::from(bit);
            read_bits += 1;
        }

        if read_bits < bit_count {
            None
        } else {
            Some(value)
        }
    }

    pub fn read_u32(&mut self, bit_count: u8) -> Option<u32> {
        debug_assert!(bit_count <= 32);

        let mut read_bits = 0u8;
        let mut value = 0;
        for bit in self.take(usize::from(bit_count)) {
            value = (value << 1) | u32::from(bit);
            read_bits += 1;
        }

        if read_bits < bit_count {
            None
        } else {
            Some(value)
        }
    }

    pub fn read_u64(&mut self, bit_count: u8) -> Option<u64> {
        debug_assert!(bit_count <= 64);

        let mut read_bits = 0u8;
        let mut value = 0;
        for bit in self.take(usize::from(bit_count)) {
            value = (value << 1) | u64::from(bit);
            read_bits += 1;
        }

        if read_bits < bit_count {
            None
        } else {
            Some(value)
        }
    }

    /// Returns the position of the cursor, or how many bits have been read so far.
    pub fn position(&self) -> u32 {
        self.read_bits - u32::from(self.remaining_bits)
    }

    #[inline(never)]
    #[cold]
    fn refill_bits(&mut self) {
        debug_assert_eq!(self.remaining_bits, 0);

        if self.bytes.len() >= USIZE_BYTES {
            self.refill_usize_bits();
        } else {
            self.refill_available_bits();
        }
    }

    fn refill_usize_bits(&mut self) {
        self.bits = usize::from_be_bytes(self.bytes[..USIZE_BYTES].try_into().unwrap());
        self.bytes = &self.bytes[USIZE_BYTES..];
        self.remaining_bits = USIZE_BITS as u8;

        self.read_bits += USIZE_BITS as u32;
    }

    #[inline(never)]
    #[cold]
    fn refill_available_bits(&mut self) {
        debug_assert!(self.bytes.len() < USIZE_BYTES);

        let mut bytes = [0u8; USIZE_BYTES];
        bytes[..self.bytes.len()].copy_from_slice(self.bytes);
        self.bits = usize::from_be_bytes(bytes);
        let bytes_slice = mem::replace(&mut self.bytes, &[]);

        self.remaining_bits = (bytes_slice.len() * 8) as u8;

        self.read_bits += self.remaining_bits as u32;
    }
}

impl<'a> Iterator for BitReader<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_bits == 0 {
            self.refill_bits();

            if self.remaining_bits == 0 {
                return None;
            }
        }

        // read the left most bit
        let bit = self.bits & !(usize::max_value() >> 1);

        self.remaining_bits -= 1;
        self.bits <<= 1;

        Some(bit != 0)
    }
}

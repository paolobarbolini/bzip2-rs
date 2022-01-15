use tinyvec::{ArrayVec, SliceVec};

use crate::bitreader::BitReader;
use crate::decoder::block::BlockError;

pub struct MoveToFrontDecoder {
    symbols: [u8; 256],
}

impl MoveToFrontDecoder {
    pub fn new_zeroed() -> Self {
        Self {
            symbols: [0u8; 256],
        }
    }

    pub fn new() -> Self {
        let mut this = Self::new_zeroed();

        for (i, symbol) in this.symbols.iter_mut().enumerate() {
            *symbol = i as u8;
        }

        this
    }

    pub fn read_from_block(reader: &mut BitReader<'_>) -> Result<(Self, usize), BlockError> {
        let mut this = Self::new_zeroed();

        let mut bitmaps = ArrayVec::<[u8; 16]>::new();

        for i in 0..16 {
            if reader
                .next()
                .ok_or_else(|| BlockError::new("symbol range truncated"))?
            {
                bitmaps.push(i);
            }
        }

        let mut symbols = SliceVec::from_slice_len(&mut this.symbols, 0);

        for symbol_range in bitmaps {
            for symbol in 0..16 {
                if reader
                    .next()
                    .ok_or_else(|| BlockError::new("symbol range truncated"))?
                {
                    symbols.push(symbol_range * 16 + symbol);
                }
            }
        }

        if symbols.is_empty() {
            return Err(BlockError::new("no symbols in input"));
        }

        let alpha_size = symbols.len() + 2;
        Ok((this, alpha_size))
    }

    pub fn decode(&mut self, n: u8) -> u8 {
        // Optimized from
        //
        // ```rust
        // let b = self.symbols[usize::from(n)];
        // self.symbols[..=usize::from(n)].rotate_right(1);
        // b
        // ```

        let b = self.symbols[usize::from(n)];
        #[cfg(feature = "rustc_1_37")]
        self.symbols.copy_within(..usize::from(n), 1);
        #[cfg(not(feature = "rustc_1_37"))]
        {
            let symbols = self.symbols;
            self.symbols[1..=usize::from(n)].copy_from_slice(&symbols[..usize::from(n)]);
        }
        self.symbols[0] = b;

        b
    }

    // Like `decode` but only works with `n <= 6`
    // About 25% faster because it doesn't emit a `memmove`
    pub fn decode_small(&mut self, n: u8) -> u8 {
        debug_assert!(n <= 6);

        let b = self.symbols[usize::from(n)];
        self.symbols[6 - usize::from(n < 6)] = self.symbols[5];
        self.symbols[5 - usize::from(n < 5)] = self.symbols[4];
        self.symbols[4 - usize::from(n < 4)] = self.symbols[3];
        self.symbols[3 - usize::from(n < 3)] = self.symbols[2];
        self.symbols[2 - usize::from(n < 2)] = self.symbols[1];
        self.symbols[1 - usize::from(n < 1)] = self.symbols[0];
        self.symbols[0] = b;

        b
    }

    pub fn first(&self) -> u8 {
        self.symbols[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_stuff() {
        let mut decoder = MoveToFrontDecoder::new();

        for i in 0..=255 {
            assert_eq!(i, usize::from(decoder.symbols[i]));
        }

        let decode = decoder.decode(0);
        assert_eq!(decode, 0);

        for i in 0..=255 {
            assert_eq!(i, usize::from(decoder.symbols[i]));
        }

        let decode = decoder.decode(5);
        assert_eq!(decode, 5);

        assert_eq!(decoder.symbols[0], 5);
        assert_eq!(decoder.symbols[1], 0);
        assert_eq!(decoder.symbols[2], 1);
        assert_eq!(decoder.symbols[3], 2);
        assert_eq!(decoder.symbols[4], 3);
        assert_eq!(decoder.symbols[5], 4);
        for i in 6..=255 {
            assert_eq!(i, usize::from(decoder.symbols[i]));
        }
    }
}

pub struct MoveToFrontDecoder {
    symbols: [u8; 256],
}

impl MoveToFrontDecoder {
    pub fn new() -> Self {
        let mut this = Self::new_from_symbols([0u8; 256]);

        for (i, symbol) in this.symbols.iter_mut().enumerate() {
            *symbol = i as u8;
        }

        this
    }

    pub fn new_from_symbols(symbols: [u8; 256]) -> Self {
        Self { symbols }
    }

    pub fn decode(&mut self, n: u8) -> u8 {
        let b = self.symbols[usize::from(n)];
        #[cfg(feature = "rustc_1_37")]
        self.symbols.copy_within(..usize::from(n), 1);
        #[cfg(not(feature = "rustc_1_37"))]
        {
            assert!((usize::from(n) + 1) <= self.symbols.len());
            unsafe {
                std::ptr::copy(
                    self.symbols.as_ptr(),
                    self.symbols.as_mut_ptr().add(1),
                    usize::from(n),
                );
            }
        }
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
            assert_eq!(i, decoder.symbols[i] as usize);
        }

        let decode = decoder.decode(0);
        assert_eq!(decode, 0);

        for i in 0..=255 {
            assert_eq!(i, decoder.symbols[i] as usize);
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
            assert_eq!(i, decoder.symbols[i] as usize);
        }
    }
}

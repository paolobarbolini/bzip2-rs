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
            let symbols = self.symbols;
            self.symbols[1..=usize::from(n)].copy_from_slice(&symbols[..usize::from(n)]);
        }
        self.symbols[0] = b;

        b
    }

    pub fn first(&self) -> u8 {
        self.symbols[0]
    }
}

pub struct PositionalMTFEncoder {
    selectors: Vec<u16>,
    mtf_freq: Vec<usize>,
}

/// Given a block of bytes B, this encoder generates
/// a list of selectors in the RUNA/RUNB format that may be
/// used to reconstruct B. These selectors can be bit-level
/// and thus achieve compression
impl PositionalMTFEncoder {
    const BYTE_RANGE: usize = 256;
    const ALPHA_RANGE: usize = 258;
    const RUNB: u16 = 1;
    const RUNA: u16 = 0;

    pub fn new() -> Self {
        PositionalMTFEncoder {
            selectors: Vec::new(),
            mtf_freq: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.selectors.clear();
        self.mtf_freq.clear();
    }

    // Tighly coupled with the overall compression routine
    // as it requires the ptr array
    // However, I find it difficult to decouple this without sacrificing performance
    pub fn encode(
        &mut self,
        nblock: usize,
        buf: &[u8],
        ptr: &[usize],
        in_use: &[bool; Self::BYTE_RANGE],
    ) {
        self.reset();
        // TODO: reserve space for at least nblock elements
        // not "additional" elements as the docs say
        // self.selectors.reserve_exact();

        let mut count_used_bytes = 0;
        let mut index_selector = [0; Self::BYTE_RANGE];
        for byte in 0..Self::BYTE_RANGE {
            if in_use[byte] {
                index_selector[byte] = count_used_bytes;
                count_used_bytes += 1;
            }
        }

        self.mtf_freq = vec![0; count_used_bytes + 2]; // +1 for EOB, +1 for 1-offset

        let mut live_mtf: Vec<usize> = (0..count_used_bytes).collect();
        let mut index_zero_freq = 0;

        macro_rules! insert_selector {
            ($selector: expr) => {
                self.selectors.push($selector);
                self.mtf_freq[$selector as usize] += 1;
            };
        }

        macro_rules! insert_pending_zeros {
            () => {
                if index_zero_freq > 0 {
                    index_zero_freq -= 1;

                    loop {
                        if (index_zero_freq & 1) == 1 {
                            insert_selector!(Self::RUNB);
                        } else {
                            insert_selector!(Self::RUNA);
                        }
                        if index_zero_freq < 2 {
                            break;
                        }
                        index_zero_freq -= 2;
                        index_zero_freq /= 2;
                    }

                    index_zero_freq = 0;
                }
            };
        }

        for i in 0..nblock {
            let j = if ptr[i] < 1 {
                ptr[i] + nblock - 1
            } else {
                ptr[i] - 1
            };

            let byte = buf[j];
            let index = index_selector[byte as usize];

            if live_mtf[0] == index {
                index_zero_freq += 1;
                continue;
            }

            insert_pending_zeros!();

            println!("{:?}", live_mtf);

            // TODO: nightly has rotate_right?
            // [0..loc] transformed to [loc; 0..loc-1]
            let mut loc = 1;
            let mut assignee = live_mtf[loc - 1];
            while live_mtf[loc] != index {
                let prev_assignee = assignee;
                assignee = live_mtf[loc];
                live_mtf[loc] = prev_assignee;
                loc += 1;
            }
            live_mtf[loc] = assignee;
            live_mtf[0] = index;

            println!("{:?} {}", live_mtf, loc);
            insert_selector!(loc as u16 + 1);
        }

        insert_pending_zeros!();

        let eob = count_used_bytes + 1;
        insert_selector!(eob as u16);
    }
}

#[cfg(test)]
mod decoder_tests {
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

#[cfg(test)]
mod encoder_tests {
    use super::*;

    fn vector_equals<T: Copy + Eq>(a: &[T], b: &[T]) -> bool {
        let count = a.iter().zip(b).filter(|&(x, y)| x == y).count();
        count == a.len() && count == b.len()
    }

    fn check_mtfe(buf: &[u8], answer: &[u16], mtf_freq: &[usize]) {
        let nblock = buf.len();
        let ptr: Vec<usize> = (1..=nblock).collect();
        let mut in_use = [false; PositionalMTFEncoder::BYTE_RANGE];

        for ch in buf.iter() {
            in_use[(*ch) as usize] = true;
        }

        let mut mtfe = PositionalMTFEncoder::new();
        mtfe.encode(nblock, buf, &ptr, &in_use);

        assert!(vector_equals(&answer, &mtfe.selectors));
        assert!(vector_equals(&mtf_freq, &mtfe.mtf_freq));
    }

    #[test]
    fn encode_distinct_bytes() {
        let buf = b"abcd1234";
        let answer = [5, 6, 7, 8, 5, 6, 7, 8, 9];
        let mtf_freq = [0, 0, 0, 0, 0, 2, 2, 2, 2, 1];
        check_mtfe(buf, &answer, &mtf_freq);
    }

    #[test]
    fn encode_repeating_small() {
        let buf = b"ab1ccb1bcddd2aa34";
        let answer = [5, 6, 3, 7, 0, 3, 3, 2, 3, 8, 1, 6, 6, 0, 7, 8, 9];
        let mtf_freq = [2, 1, 1, 4, 0, 1, 3, 2, 2, 1];

        check_mtfe(buf, &answer, &mtf_freq);
    }

    #[test]
    fn encode_repeating_large() {
        let buf = b"972938M0o1Dwy5T4afiCDM1sJ227jot92F35cJwpivOOK13yvPOEdI177zxQVk82N1SWK1e962xPngky02445blwn9mEz3T3LCwW";
        let answer = [
            10, 9, 5, 3, 6, 10, 19, 7, 41, 9, 14, 46, 48, 14, 28, 15, 31, 36, 38, 20, 10, 14, 12,
            45, 25, 20, 0, 21, 40, 18, 46, 23, 6, 26, 24, 20, 38, 11, 23, 46, 19, 47, 34, 0, 32,
            19, 11, 27, 6, 35, 7, 32, 41, 34, 9, 23, 0, 49, 49, 39, 41, 46, 38, 25, 41, 10, 42, 43,
            21, 4, 45, 30, 43, 9, 14, 21, 49, 47, 15, 25, 45, 8, 44, 0, 34, 47, 48, 34, 11, 15, 49,
            30, 27, 33, 48, 2, 49, 47, 10, 26, 50,
        ];
        let mtf_freq = [
            4, 0, 1, 1, 1, 1, 3, 2, 1, 4, 5, 3, 1, 0, 4, 3, 0, 0, 1, 3, 3, 3, 0, 3, 1, 3, 2, 2, 1,
            0, 2, 1, 2, 1, 4, 1, 1, 0, 3, 1, 1, 4, 1, 2, 1, 3, 4, 4, 3, 5, 1,
        ];
        check_mtfe(buf, &answer, &mtf_freq);
    }
}

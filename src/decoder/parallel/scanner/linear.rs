use std::convert::TryInto;

use crate::decoder::block::BLOCK_MAGIC;

/// Returns the bit offset into `buf` to `BLOCK_MAGIC`, or `None` if it's not found
pub fn find_next_signature(buf: &[u8]) -> Option<u64> {
    for (byte_index, window) in buf.windows(8).enumerate().step_by(2) {
        let window = u64::from_be_bytes(window.try_into().expect("unreachable"));

        macro_rules! unrolled_check {
            ($shift: expr) => {
                let window = window >> $shift;
                let signature = window & !(u64::max_value() << 48);
                if signature == BLOCK_MAGIC {
                    return Some((byte_index as u64) * 8 + (16 - $shift));
                }
            };
        }

        unrolled_check!(0);
        unrolled_check!(1);
        unrolled_check!(2);
        unrolled_check!(3);
        unrolled_check!(4);
        unrolled_check!(5);
        unrolled_check!(6);
        unrolled_check!(7);
        unrolled_check!(8);
        unrolled_check!(9);
        unrolled_check!(10);
        unrolled_check!(11);
        unrolled_check!(12);
        unrolled_check!(13);
        unrolled_check!(14);
        unrolled_check!(15);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitreader::BitReader;

    #[test]
    fn find_at_any_offset_0() {
        for shift in 0..=80 {
            println!("shift: {}", shift);

            let mut haystack = vec![0u8; 1024];
            let shifted = u128::from(BLOCK_MAGIC) << shift;
            let shifted = u128::to_be_bytes(shifted);
            println!("shifted: {:?}", shifted);
            haystack.extend_from_slice(&shifted);
            haystack.resize(haystack.len() + 1024, 0);

            let pos = find_next_signature(&haystack).unwrap();
            assert_eq!(pos, (1024 * 8) + (128 - 48 - shift));

            let mut reader = BitReader::new(&haystack);
            reader.advance_by(pos as usize);

            let magic = reader.read_u64(48).unwrap();
            assert_eq!(BLOCK_MAGIC, magic);
        }
    }

    #[test]
    fn find_at_any_offset_255() {
        for shift in 0..=80 {
            println!("shift: {}", shift);

            let mut haystack = vec![255u8; 1024];
            let shifted = u128::from(BLOCK_MAGIC) << shift;
            let shifted = u128::to_be_bytes(shifted);
            println!("shifted: {:?}", shifted);
            haystack.extend_from_slice(&shifted);
            haystack.resize(haystack.len() + 1024, 255);

            let pos = find_next_signature(&haystack).unwrap();
            assert_eq!(pos, (1024 * 8) + (128 - 48 - shift));

            let mut reader = BitReader::new(&haystack);
            reader.advance_by(pos as usize);

            let magic = reader.read_u64(48).unwrap();
            assert_eq!(BLOCK_MAGIC, magic);
        }
    }
}

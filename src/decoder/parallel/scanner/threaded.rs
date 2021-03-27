use std::sync::mpsc::channel;
use std::sync::Arc;

use super::iter::SignatureFinder;
use crate::ThreadPool;

/// An iterator returning the bit offsets into `buf` to `BLOCK_MAGIC`
pub fn find_signatures_parallel<P>(memory: Arc<[u8]>, pool: &P) -> Vec<u64>
where
    P: ThreadPool,
{
    let threads = pool.max_threads();
    let chunk_size = memory.len() / threads.get();

    let (sender, receiver) = channel::<u64>();

    for i in 0..threads.get() {
        let start = chunk_size * i;
        let end = start + chunk_size + 8;

        let sender = sender.clone();
        let memory = Arc::clone(&memory);
        pool.spawn(move || {
            let finder = SignatureFinder::new(&memory[start..end.min(memory.len())]);
            for signature_index in finder {
                let _ = sender.send(((start as u64) * 8) + signature_index);
            }
        });
    }

    // drop sender, so that when all threads finish no copy of `Sender` will be left
    // and `collect`ing `Receiver` will finish.
    drop(sender);

    let mut indexes = receiver.into_iter().collect::<Vec<u64>>();
    indexes.sort_unstable();

    indexes
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::thread;

    use super::*;
    use crate::bitreader::BitReader;
    use crate::decoder::block::BLOCK_MAGIC;

    struct NaiveThreadPool;

    impl ThreadPool for NaiveThreadPool {
        fn spawn<F>(&self, func: F)
        where
            F: FnOnce() + Send + 'static,
        {
            thread::spawn(func);
        }

        fn max_threads(&self) -> NonZeroUsize {
            NonZeroUsize::new(4).unwrap()
        }
    }

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

            let repeated_haystack = Arc::<[u8]>::from(repeated_haystack);

            let finder = find_signatures_parallel(repeated_haystack.clone(), &NaiveThreadPool);
            let mut finder = finder.into_iter();

            for _ in 0..16 {
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

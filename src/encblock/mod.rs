use crate::bitreader::BitWriter;
use crate::block_common::*;
use crate::crc::Hasher;
use crate::header::Header;

mod blocksort;

#[derive(Clone, Copy, Debug)]
enum State {
    /// Writing compressed data
    Writing,
    /// Not writing compressed data
    NotWriting,
    /// Awaiting compression
    ReadyToCompress,
}

pub(crate) struct Block {
    header: Header,
    state: State,
    hasher: Hasher,
    in_buf: Vec<u8>,
    block_crc: u32,
    combined_crc: u32,
}

impl Block {
    pub fn new(header: Header) -> Self {
        let in_buf = Vec::with_capacity(header.max_blocksize() as usize);

        Self {
            header,
            state: State::NotWriting,
            in_buf,
            hasher: Hasher::new(),
            block_crc: 0,
            combined_crc: 0,
        }
    }

    pub fn reset(&mut self) {
        self.in_buf.clear();
        self.hasher.reset();
    }

    pub fn ready_to_write(&mut self) {
        self.state = State::Writing;
        self.hasher.finalyze();
        // TOOD: this should also handle whole stream CRC
    }

    pub fn read(&mut self, buf: &[u8]) -> usize {
        let space = self.space();
        let written = space.min(buf.len());
        let slice_taken = &buf[..written];
        self.in_buf.extend_from_slice(slice_taken);
        self.hasher.update(slice_taken);

        if self.space() == 0 {
            self.ready_to_write();
        }

        return written;
    }

    pub fn space(&self) -> usize {
        if self.is_writing() {
            return 0;
        }

        // As per bz2 C code (nblockMAX)
        let max_length = (self.header.max_blocksize() - 19) as usize;
        max_length - self.in_buf.len()
    }

    pub fn is_writing(&self) -> bool {
        match self.state {
            State::Writing => true,
            State::NotWriting | State::ReadyToCompress => false,
        }
    }

    pub fn write(&mut self, writer: &mut BitWriter<'_>) -> usize {
        match self.state {
            State::ReadyToCompress => {
                blocksort::block_sort();
            }
            State::Writing => {
                // TODO: how to check buf size
                writer.write_u32(self.block_crc);
                writer.
            }
            State::NotWriting => {}
        }
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_test(filename: &str) {
        let compressed = include_bytes!(format!("../../tests/samplefiles/{}.bz2", filename));
        let decompressed = include_bytes!(format!("../../tests/samplefiles/{}.ref", filename));

        todo!();
    }

    #[test]
    fn encode_small_test() {
        encode_test("sample1");
    }
}

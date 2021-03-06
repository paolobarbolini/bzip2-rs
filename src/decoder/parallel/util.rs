pub struct ReadableVec {
    vec: Vec<u8>,
    skip: usize,
}

impl ReadableVec {
    pub fn read<'a>(&mut self, buf: &'a mut [u8]) -> &'a mut [u8] {
        let available = &self.vec[self.skip..];
        let read = available.len().min(buf.len());

        buf[..read].copy_from_slice(&available[..read]);
        self.skip += read;
        &mut buf[read..]
    }
}

impl From<Vec<u8>> for ReadableVec {
    fn from(vec: Vec<u8>) -> Self {
        Self { vec, skip: 0 }
    }
}

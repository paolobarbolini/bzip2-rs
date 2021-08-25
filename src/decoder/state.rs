/// State returned by [`Decoder::read`]
pub enum ReadState {
    /// Not enough data has been written to the underlying [`Decoder`]
    /// in order to allow the next block to be decoded. Call
    /// [`Decoder::write`] to write more data. If the end of the file
    /// has been reached, call [`Decoder::write`] with an empty buffer.
    NeedsWrite,
    /// N. number of data has been read
    Read(usize),
    /// The end of the compressed file has been reached and
    /// there is no more data to read
    Eof,
}

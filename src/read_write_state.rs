/// State returned by [`Decoder::write`]
pub enum WriteState {
    /// Enough data has already been written to [`Decoder`]
    /// in order for it to be able to decode the next block.
    /// Now call [`Decoder::read`] to read the decompressed data.
    NeedsRead,
    /// N. number of bytes have been written.
    Written(usize),
}

/// State returned by [`Decoder::read`]
pub enum ReadState {
    /// Not enough data has been written to the underlying [`Decoder`]
    /// in order to allow the next block to be decoded. Call
    /// [`Decoder::write`] to write more data. If the end of the file
    /// has been reached, call [`Decoder::write`] with an empty buffer.
    NeedsWrite(usize),
    /// N. number of data has been read
    Read(usize),
    /// The end of the compressed file has been reached and
    /// there is no more data to read
    Eof,
}

/// State returned by [`Decoder::read`][super::Decoder::read]
pub enum ReadState {
    /// Not enough data has been written to the underlying
    /// [`Decoder`][super::Decoder] in order to allow the next block
    /// to be decoded. Call [`Decoder::write`][super::Decoder::write]
    /// to write more data.
    /// If the end of the file has been reached, call
    /// [`Decoder::write_eof`][super::Decoder::write_eof] instead.
    NeedsWrite,
    /// N. bytes have been decoded into the provided buffer
    Read(usize),
    /// The end of the compressed file has been reached and
    /// there is no more data to read
    Eof,
}

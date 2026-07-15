use crate::error::ProtocolError;
use flatbuffers::FlatBufferBuilder;

/// Trait implemented by every owned PrivChat protocol message.
///
/// Owned structs (`SendMessageRequest`, `PushMessageRequest`, etc.) live in
/// `crate::protocol`. Generated FlatBuffers view types live in `crate::fb`
/// — application code should not see them.
///
/// Hot-path callers can reuse a single `FlatBufferBuilder` across messages
/// via `encode_fb_into`, avoiding per-call allocation.
pub trait FlatBufferMessage: Sized {
    /// Build the message into the supplied builder. Caller is responsible
    /// for `builder.reset()` between uses; this method does NOT call reset.
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError>;

    /// Allocate a builder, encode, and return the bytes.
    fn encode_fb(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut builder = FlatBufferBuilder::with_capacity(1024);
        self.encode_fb_into(&mut builder)?;
        Ok(builder.finished_data().to_vec())
    }

    /// Decode a verified FlatBuffer into the owned struct.
    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError>;
}

/// Top-level encode entry point. Equivalent to `T::encode_fb()`.
pub fn encode_message<T: FlatBufferMessage>(message: &T) -> Result<Vec<u8>, ProtocolError> {
    message.encode_fb()
}

/// Top-level decode entry point. Equivalent to `T::decode_fb(bytes)`.
pub fn decode_message<T: FlatBufferMessage>(bytes: &[u8]) -> Result<T, ProtocolError> {
    T::decode_fb(bytes)
}

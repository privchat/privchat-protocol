use thiserror::Error;

/// Errors raised by the FlatBuffers codec layer.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("flatbuffers verification failed: {0}")]
    Verification(String),

    #[error("flatbuffers decode failed: {0}")]
    Decode(String),

    #[error("required field missing on the wire: {0}")]
    MissingField(&'static str),

    #[error("invalid field value: {0}")]
    InvalidValue(String),

    #[error("unsupported message type: {0}")]
    UnsupportedMessageType(u8),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<flatbuffers::InvalidFlatbuffer> for ProtocolError {
    fn from(e: flatbuffers::InvalidFlatbuffer) -> Self {
        ProtocolError::Verification(e.to_string())
    }
}

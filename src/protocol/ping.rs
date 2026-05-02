//! Application-layer heartbeat messages.

use super::{Message, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PingRequest {
    pub timestamp: i64,
}

impl PingRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PingRequest, self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PongResponse {
    pub timestamp: i64,
}

impl PongResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PongResponse, self)
    }
}

impl Message for PingRequest {
    fn message_type(&self) -> MessageType {
        MessageType::PingRequest
    }
}
impl Message for PongResponse {
    fn message_type(&self) -> MessageType {
        MessageType::PongResponse
    }
}

impl FlatBufferMessage for PingRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let args = fb::PingRequestArgs {
            timestamp: self.timestamp,
        };
        let offset = fb::PingRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::PingRequest>(bytes)?;
        Ok(Self {
            timestamp: view.timestamp(),
        })
    }
}

impl FlatBufferMessage for PongResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let args = fb::PongResponseArgs {
            timestamp: self.timestamp,
        };
        let offset = fb::PongResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::PongResponse>(bytes)?;
        Ok(Self {
            timestamp: view.timestamp(),
        })
    }
}

//! Disconnect handshake messages.

use super::{Message, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectRequest {
    pub reason: DisconnectReason,
    pub message: Option<String>,
}

impl DisconnectRequest {
    pub fn new() -> Self {
        Self {
            reason: DisconnectReason::Unknown,
            message: None,
        }
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::DisconnectRequest, self)
    }
}

impl Default for DisconnectRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisconnectResponse {
    pub acknowledged: bool,
}

impl DisconnectResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::DisconnectResponse, self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DisconnectReason {
    Unknown = 0,
    UserInitiated = 1,
    ServerShutdown = 2,
    AuthenticationFailed = 3,
    ProtocolError = 4,
    Timeout = 5,
    DuplicateConnection = 6,
    ServerMaintenance = 7,
}

impl Default for DisconnectReason {
    fn default() -> Self {
        DisconnectReason::Unknown
    }
}

impl Message for DisconnectRequest {
    fn message_type(&self) -> MessageType {
        MessageType::DisconnectRequest
    }
}
impl Message for DisconnectResponse {
    fn message_type(&self) -> MessageType {
        MessageType::DisconnectResponse
    }
}

fn reason_to_fb(r: DisconnectReason) -> fb::DisconnectReason {
    fb::DisconnectReason(r as u8)
}
fn reason_from_fb(r: fb::DisconnectReason) -> DisconnectReason {
    match r.0 {
        1 => DisconnectReason::UserInitiated,
        2 => DisconnectReason::ServerShutdown,
        3 => DisconnectReason::AuthenticationFailed,
        4 => DisconnectReason::ProtocolError,
        5 => DisconnectReason::Timeout,
        6 => DisconnectReason::DuplicateConnection,
        7 => DisconnectReason::ServerMaintenance,
        _ => DisconnectReason::Unknown,
    }
}

impl FlatBufferMessage for DisconnectRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let message = self.message.as_ref().map(|s| builder.create_string(s));
        let args = fb::DisconnectRequestArgs {
            reason: reason_to_fb(self.reason),
            message,
        };
        let offset = fb::DisconnectRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::DisconnectRequest>(bytes)?;
        Ok(Self {
            reason: reason_from_fb(view.reason()),
            message: view.message().map(|s| s.to_string()),
        })
    }
}

impl FlatBufferMessage for DisconnectResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let args = fb::DisconnectResponseArgs {
            acknowledged: self.acknowledged,
        };
        let offset = fb::DisconnectResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::DisconnectResponse>(bytes)?;
        Ok(Self {
            acknowledged: view.acknowledged(),
        })
    }
}

//! Channel subscribe / unsubscribe messages.

use super::{Message, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscribeRequest {
    /// Bit flags (raw u8 — NOT the MessageSetting struct).
    pub setting: u8,
    pub local_message_id: u64,
    pub channel_id: u64,
    pub channel_type: u8,
    pub action: u8,
    pub param: String,
}

impl SubscribeRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SubscribeRequest, self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscribeResponse {
    pub local_message_id: u64,
    pub channel_id: u64,
    pub channel_type: u8,
    pub action: u8,
    pub reason_code: u8,
}

impl SubscribeResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SubscribeResponse, self)
    }
}

impl Message for SubscribeRequest {
    fn message_type(&self) -> MessageType {
        MessageType::SubscribeRequest
    }
}
impl Message for SubscribeResponse {
    fn message_type(&self) -> MessageType {
        MessageType::SubscribeResponse
    }
}

impl FlatBufferMessage for SubscribeRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let param = builder.create_string(&self.param);
        let args = fb::SubscribeRequestArgs {
            setting: self.setting,
            local_message_id: self.local_message_id,
            channel_id: self.channel_id,
            channel_type: self.channel_type,
            action: self.action,
            param: Some(param),
        };
        let offset = fb::SubscribeRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::SubscribeRequest>(bytes)?;
        Ok(Self {
            setting: view.setting(),
            local_message_id: view.local_message_id(),
            channel_id: view.channel_id(),
            channel_type: view.channel_type(),
            action: view.action(),
            param: view.param().unwrap_or("").to_string(),
        })
    }
}

impl FlatBufferMessage for SubscribeResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let args = fb::SubscribeResponseArgs {
            local_message_id: self.local_message_id,
            channel_id: self.channel_id,
            channel_type: self.channel_type,
            action: self.action,
            reason_code: self.reason_code,
        };
        let offset = fb::SubscribeResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::SubscribeResponse>(bytes)?;
        Ok(Self {
            local_message_id: view.local_message_id(),
            channel_id: view.channel_id(),
            channel_type: view.channel_type(),
            action: view.action(),
            reason_code: view.reason_code(),
        })
    }
}

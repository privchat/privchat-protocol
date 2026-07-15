//! Channel publish (server-side broadcast input) messages.

use super::{Message, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishRequest {
    pub channel_id: u64,
    pub topic: Option<String>,
    pub timestamp: u64,
    pub payload: Vec<u8>,
    pub publisher: Option<String>,
    pub server_message_id: Option<u64>,
}

impl PublishRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PublishRequest, self)
    }

    pub fn system_push(channel_id: u64, payload: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            channel_id,
            topic: None,
            timestamp,
            payload,
            publisher: Some("system".to_string()),
            server_message_id: Some(timestamp),
        }
    }

    pub fn topic_push(channel_id: u64, topic: &str, payload: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            channel_id,
            topic: Some(topic.to_string()),
            timestamp,
            payload,
            publisher: None,
            server_message_id: Some(timestamp),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishResponse {
    pub succeed: bool,
    pub message: Option<String>,
}

impl PublishResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PublishResponse, self)
    }

    pub fn success() -> Self {
        Self {
            succeed: true,
            message: Some("推送消息接收成功".to_string()),
        }
    }

    pub fn failure(error_msg: &str) -> Self {
        Self {
            succeed: false,
            message: Some(error_msg.to_string()),
        }
    }
}

impl Message for PublishRequest {
    fn message_type(&self) -> MessageType {
        MessageType::PublishRequest
    }
}
impl Message for PublishResponse {
    fn message_type(&self) -> MessageType {
        MessageType::PublishResponse
    }
}

impl FlatBufferMessage for PublishRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let topic = self.topic.as_ref().map(|s| builder.create_string(s));
        let publisher = self.publisher.as_ref().map(|s| builder.create_string(s));
        let payload = builder.create_vector(&self.payload);

        let args = fb::PublishRequestArgs {
            channel_id: self.channel_id,
            topic,
            timestamp: self.timestamp,
            payload: Some(payload),
            publisher,
            server_message_id: self.server_message_id.unwrap_or(0),
        };
        let offset = fb::PublishRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::PublishRequest>(bytes)?;
        let server_message_id = match view.server_message_id() {
            0 => None,
            n => Some(n),
        };
        Ok(Self {
            channel_id: view.channel_id(),
            topic: view.topic().map(|s| s.to_string()),
            timestamp: view.timestamp(),
            payload: view
                .payload()
                .map(|v| v.bytes().to_vec())
                .unwrap_or_default(),
            publisher: view.publisher().map(|s| s.to_string()),
            server_message_id,
        })
    }
}

impl FlatBufferMessage for PublishResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let message = self.message.as_ref().map(|s| builder.create_string(s));
        let args = fb::PublishResponseArgs {
            succeed: self.succeed,
            message,
        };
        let offset = fb::PublishResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::PublishResponse>(bytes)?;
        Ok(Self {
            succeed: view.succeed(),
            message: view.message().map(|s| s.to_string()),
        })
    }
}

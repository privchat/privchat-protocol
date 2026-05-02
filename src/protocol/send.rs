//! Client-to-server send-message messages.

use super::{decode_setting, encode_setting, Message, MessageSetting, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub setting: MessageSetting,
    pub client_seq: u32,
    pub local_message_id: u64,
    pub stream_no: String,
    pub channel_id: u64,
    /// Application content type (NOT the wire `MessageType`).
    pub message_type: u32,
    pub expire: u32,
    pub from_uid: u64,
    pub topic: String,
    pub payload: Vec<u8>,
}

impl SendMessageRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SendMessageRequest, self)
    }

    pub fn verify_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.local_message_id, self.channel_id, self.from_uid
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub client_seq: u32,
    pub server_message_id: u64,
    pub message_seq: u32,
    pub reason_code: u32,
}

impl SendMessageResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SendMessageResponse, self)
    }
}

impl Message for SendMessageRequest {
    fn message_type(&self) -> MessageType {
        MessageType::SendMessageRequest
    }
}
impl Message for SendMessageResponse {
    fn message_type(&self) -> MessageType {
        MessageType::SendMessageResponse
    }
}

impl FlatBufferMessage for SendMessageRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let setting = encode_setting(builder, &self.setting);
        let stream_no = builder.create_string(&self.stream_no);
        let topic = builder.create_string(&self.topic);
        let payload = builder.create_vector(&self.payload);

        let args = fb::SendMessageRequestArgs {
            setting: Some(setting),
            client_seq: self.client_seq,
            local_message_id: self.local_message_id,
            stream_no: Some(stream_no),
            channel_id: self.channel_id,
            message_type: self.message_type,
            expire: self.expire,
            from_uid: self.from_uid,
            topic: Some(topic),
            payload: Some(payload),
        };
        let offset = fb::SendMessageRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::SendMessageRequest>(bytes)?;
        Ok(Self {
            setting: decode_setting(view.setting()),
            client_seq: view.client_seq(),
            local_message_id: view.local_message_id(),
            stream_no: view.stream_no().unwrap_or("").to_string(),
            channel_id: view.channel_id(),
            message_type: view.message_type(),
            expire: view.expire(),
            from_uid: view.from_uid(),
            topic: view.topic().unwrap_or("").to_string(),
            payload: view.payload().map(|v| v.bytes().to_vec()).unwrap_or_default(),
        })
    }
}

impl FlatBufferMessage for SendMessageResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let args = fb::SendMessageResponseArgs {
            client_seq: self.client_seq,
            server_message_id: self.server_message_id,
            message_seq: self.message_seq,
            reason_code: self.reason_code,
        };
        let offset = fb::SendMessageResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::SendMessageResponse>(bytes)?;
        Ok(Self {
            client_seq: view.client_seq(),
            server_message_id: view.server_message_id(),
            message_seq: view.message_seq(),
            reason_code: view.reason_code(),
        })
    }
}

//! Server-to-client message push (single + batch).

use super::{decode_setting, encode_setting, Message, MessageSetting, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushMessageRequest {
    pub setting: MessageSetting,
    pub msg_key: String,
    pub server_message_id: u64,
    pub message_seq: u32,
    pub local_message_id: u64,
    pub stream_no: String,
    pub stream_seq: u32,
    pub stream_flag: u8,
    pub timestamp: u32,
    pub channel_id: u64,
    pub channel_type: u8,
    pub message_type: u32,
    pub expire: u32,
    pub topic: String,
    pub from_uid: u64,
    pub payload: Vec<u8>,
    /// True when this push notifies a recall of an existing server_message_id.
    /// SDK marks the matching local message as revoked.
    pub deleted: bool,
}

impl PushMessageRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushMessageRequest, self)
    }

    pub fn verify_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.server_message_id, self.channel_id, self.from_uid
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushMessageResponse {
    pub succeed: bool,
    pub message: Option<String>,
}

impl PushMessageResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushMessageResponse, self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushBatchRequest {
    pub messages: Vec<PushMessageRequest>,
}

impl PushBatchRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushBatchRequest, self)
    }

    pub fn single_batch(messages: Vec<PushMessageRequest>) -> Self {
        Self { messages }
    }

    pub fn multi_batch(messages: Vec<PushMessageRequest>) -> Self {
        Self { messages }
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushBatchResponse {
    pub succeed: bool,
    pub message: Option<String>,
}

impl PushBatchResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushBatchResponse, self)
    }

    pub fn success() -> Self {
        Self {
            succeed: true,
            message: Some("批量消息接收成功".to_string()),
        }
    }

    pub fn failure(error_msg: &str) -> Self {
        Self {
            succeed: false,
            message: Some(error_msg.to_string()),
        }
    }
}

impl Message for PushMessageRequest {
    fn message_type(&self) -> MessageType {
        MessageType::PushMessageRequest
    }
}
impl Message for PushMessageResponse {
    fn message_type(&self) -> MessageType {
        MessageType::PushMessageResponse
    }
}
impl Message for PushBatchRequest {
    fn message_type(&self) -> MessageType {
        MessageType::PushBatchRequest
    }
}
impl Message for PushBatchResponse {
    fn message_type(&self) -> MessageType {
        MessageType::PushBatchResponse
    }
}

fn encode_push<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    msg: &PushMessageRequest,
) -> flatbuffers::WIPOffset<fb::PushMessageRequest<'a>> {
    let setting = encode_setting(builder, &msg.setting);
    let msg_key = builder.create_string(&msg.msg_key);
    let stream_no = builder.create_string(&msg.stream_no);
    let topic = builder.create_string(&msg.topic);
    let payload = builder.create_vector(&msg.payload);

    fb::PushMessageRequest::create(
        builder,
        &fb::PushMessageRequestArgs {
            setting: Some(setting),
            msg_key: Some(msg_key),
            server_message_id: msg.server_message_id,
            message_seq: msg.message_seq,
            local_message_id: msg.local_message_id,
            stream_no: Some(stream_no),
            stream_seq: msg.stream_seq,
            stream_flag: msg.stream_flag,
            timestamp: msg.timestamp,
            channel_id: msg.channel_id,
            channel_type: msg.channel_type,
            message_type: msg.message_type,
            expire: msg.expire,
            topic: Some(topic),
            from_uid: msg.from_uid,
            payload: Some(payload),
            deleted: msg.deleted,
        },
    )
}

fn decode_push(view: fb::PushMessageRequest<'_>) -> PushMessageRequest {
    PushMessageRequest {
        setting: decode_setting(view.setting()),
        msg_key: view.msg_key().unwrap_or("").to_string(),
        server_message_id: view.server_message_id(),
        message_seq: view.message_seq(),
        local_message_id: view.local_message_id(),
        stream_no: view.stream_no().unwrap_or("").to_string(),
        stream_seq: view.stream_seq(),
        stream_flag: view.stream_flag(),
        timestamp: view.timestamp(),
        channel_id: view.channel_id(),
        channel_type: view.channel_type(),
        message_type: view.message_type(),
        expire: view.expire(),
        topic: view.topic().unwrap_or("").to_string(),
        from_uid: view.from_uid(),
        payload: view
            .payload()
            .map(|v| v.bytes().to_vec())
            .unwrap_or_default(),
        deleted: view.deleted(),
    }
}

impl FlatBufferMessage for PushMessageRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let offset = encode_push(builder, self);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::PushMessageRequest>(bytes)?;
        Ok(decode_push(view))
    }
}

impl FlatBufferMessage for PushMessageResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let message = self.message.as_ref().map(|s| builder.create_string(s));
        let args = fb::PushMessageResponseArgs {
            succeed: self.succeed,
            message,
        };
        let offset = fb::PushMessageResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::PushMessageResponse>(bytes)?;
        Ok(Self {
            succeed: view.succeed(),
            message: view.message().map(|s| s.to_string()),
        })
    }
}

impl FlatBufferMessage for PushBatchRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let msg_offsets: Vec<_> = self
            .messages
            .iter()
            .map(|m| encode_push(builder, m))
            .collect();
        let messages_vec = builder.create_vector(&msg_offsets);
        let args = fb::PushBatchRequestArgs {
            messages: Some(messages_vec),
        };
        let offset = fb::PushBatchRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::PushBatchRequest>(bytes)?;
        let messages = view
            .messages()
            .map(|vec| vec.iter().map(decode_push).collect())
            .unwrap_or_default();
        Ok(Self { messages })
    }
}

impl FlatBufferMessage for PushBatchResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let message = self.message.as_ref().map(|s| builder.create_string(s));
        let args = fb::PushBatchResponseArgs {
            succeed: self.succeed,
            message,
        };
        let offset = fb::PushBatchResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::PushBatchResponse>(bytes)?;
        Ok(Self {
            succeed: view.succeed(),
            message: view.message().map(|s| s.to_string()),
        })
    }
}

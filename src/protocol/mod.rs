//! Owned PrivChat protocol message types and their FlatBuffers codec impls.
//!
//! Each submodule mirrors a `protocol/*.fbs` file. Within each submodule, the
//! owned Rust struct lives next to its `FlatBufferMessage` implementation —
//! schema, owned type, and codec stay together.

use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

mod auth;
mod content;
mod disconnect;
mod entity_sync;
mod ping;
mod publish;
mod push;
mod rpc;
mod send;
mod subscribe;
mod timeline;
mod transfer;

pub use auth::*;
pub use content::*;
pub use disconnect::*;
pub use entity_sync::*;
pub use ping::*;
pub use publish::*;
pub use push::*;
pub use rpc::*;
pub use send::*;
pub use subscribe::*;
pub use timeline::*;
pub use transfer::*;

// ------------------------------------------------------------------
// Cross-message shared types
// ------------------------------------------------------------------

/// Trait shared by every protocol message — identifies its wire type.
pub trait Message: Debug + Send + Sync {
    fn message_type(&self) -> MessageType;
}

/// Wire-level message type, carried in `msgtrans` `Packet.biz_type`.
///
/// Numeric values are PERMANENT once assigned; only add new variants at the
/// end. `Unknown = 0` is reserved as the FlatBuffers default value (a packet
/// arriving with `biz_type == 0` is ill-formed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    Unknown = 0,
    AuthorizationRequest = 1,
    AuthorizationResponse = 2,
    DisconnectRequest = 3,
    DisconnectResponse = 4,
    SendMessageRequest = 5,
    SendMessageResponse = 6,
    PushMessageRequest = 7,
    PushMessageResponse = 8,
    PushBatchRequest = 9,
    PushBatchResponse = 10,
    PingRequest = 11,
    PongResponse = 12,
    SubscribeRequest = 13,
    SubscribeResponse = 14,
    PublishRequest = 15,
    PublishResponse = 16,
    RpcRequest = 17,
    RpcResponse = 18,
    TransferRequest = 19,
    TransferResponse = 20,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => MessageType::AuthorizationRequest,
            2 => MessageType::AuthorizationResponse,
            3 => MessageType::DisconnectRequest,
            4 => MessageType::DisconnectResponse,
            5 => MessageType::SendMessageRequest,
            6 => MessageType::SendMessageResponse,
            7 => MessageType::PushMessageRequest,
            8 => MessageType::PushMessageResponse,
            9 => MessageType::PushBatchRequest,
            10 => MessageType::PushBatchResponse,
            11 => MessageType::PingRequest,
            12 => MessageType::PongResponse,
            13 => MessageType::SubscribeRequest,
            14 => MessageType::SubscribeResponse,
            15 => MessageType::PublishRequest,
            16 => MessageType::PublishResponse,
            17 => MessageType::RpcRequest,
            18 => MessageType::RpcResponse,
            19 => MessageType::TransferRequest,
            20 => MessageType::TransferResponse,
            _ => MessageType::Unknown,
        }
    }
}

impl From<MessageType> for u8 {
    fn from(msg_type: MessageType) -> Self {
        msg_type as u8
    }
}

/// Per-message delivery flags carried alongside Send / Push.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageSetting {
    pub need_receipt: bool,
    pub signal: u8,
}

impl MessageSetting {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Programming-side wrapper grouping a message body with its type tag.
/// Not on the wire — `msgtrans.biz_type` is the canonical type carrier.
#[derive(Debug, Clone)]
pub struct Packet<T: Message> {
    pub message_type: MessageType,
    pub body: T,
}

impl<T: Message> Packet<T> {
    pub fn new(message_type: MessageType, body: T) -> Self {
        Self { message_type, body }
    }
}

// ------------------------------------------------------------------
// Cross-module codec helpers (used by send + push)
// ------------------------------------------------------------------

pub(crate) fn encode_setting<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    setting: &MessageSetting,
) -> flatbuffers::WIPOffset<fb::MessageSetting<'a>> {
    fb::MessageSetting::create(
        builder,
        &fb::MessageSettingArgs {
            need_receipt: setting.need_receipt,
            signal: setting.signal,
        },
    )
}

pub(crate) fn decode_setting(view: Option<fb::MessageSetting<'_>>) -> MessageSetting {
    match view {
        Some(s) => MessageSetting {
            need_receipt: s.need_receipt(),
            signal: s.signal(),
        },
        None => MessageSetting::default(),
    }
}

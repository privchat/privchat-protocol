// Copyright 2024 Shanghai Boyu Information Technology Co., Ltd.
// https://privchat.dev
//
// Author: zoujiaqing <zoujiaqing@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0

//! Application-level content type discriminator.
//!
//! `ContentMessageType` enumerates the meaning of a message's content
//! (text / voice / image / ...). It corresponds to the `message_type: u32`
//! field on `SendMessageRequest` / `PushMessageRequest` and is intentionally
//! NOT a FlatBuffers enum — its `Text = 0` wire convention predates the FB
//! migration and must remain stable across stored messages.
//!
//! The accompanying payload structures (`MessagePayloadEnvelope`,
//! `*Metadata`, `MessageSource`) now live in `crate::protocol::content` and
//! are FlatBuffers-encoded. They are re-exported from the crate root via
//! `pub use protocol::*;` so existing `privchat_protocol::ImageMetadata`
//! imports keep resolving.

use crate::protocol::MessageSource;
use serde::{Deserialize, Serialize};

/// Legacy JSON envelope used by local SDK persistence, FFI inputs, and any
/// caller that receives or builds a JSON-encoded envelope blob (e.g. an old
/// row stored in the SDK's local SQLite content column, an API call where
/// the user provides a JSON envelope as content text, etc.).
///
/// Compared to the FlatBuffers-canonical [`crate::MessagePayloadEnvelope`]:
///   - `metadata` is an opaque `serde_json::Value` (no typed dispatch)
///   - `reply_to_message_id` is `Option<String>` (matches the legacy JSON
///     contract that stores u64 IDs as strings to stay JS-safe)
///   - `mentioned_user_ids` is `Option<Vec<u64>>` (preserves None vs Some([]))
///
/// Use `LocalMessagePayloadEnvelope` for local-only / JSON paths; convert
/// via [`crate::MessagePayloadEnvelope::from_legacy`] /
/// [`crate::MessagePayloadEnvelope::to_legacy`] at the wire boundary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalMessagePayloadEnvelope {
    #[serde(default)]
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub reply_to_message_id: Option<String>,
    pub mentioned_user_ids: Option<Vec<u64>>,
    pub message_source: Option<MessageSource>,
}

/// Typed metadata used while a local attachment is being prepared.
///
/// This is a local persistence contract, not a wire payload. Platform wrappers
/// pass these fields through FFI and the Rust SDK serializes the structure for
/// its legacy SQLite `extra` column. Keeping the structure here prevents each
/// platform from inventing a JSON shape independently.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalAttachmentMetadata {
    pub file_name: String,
    pub mime_type: String,
    /// 随附件一起发出的说明文字（「图片配一句话」）。空 = 没有。
    ///
    /// 它是消息内容的一部分，不是本地渲染细节：发送时它就是 wire envelope 的
    /// `content`，接收端据此显示。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default)]
    pub duration: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub thumbnail_width: Option<u32>,
    #[serde(default)]
    pub thumbnail_height: Option<u32>,
}

/// Content message type (u32 wire). Numeric values are PERMANENT; only
/// add new variants at the end. Unknown values are surfaced as `None`
/// from `from_u32` and rendered as a fallback bubble by clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ContentMessageType {
    Text = 0,
    Voice = 1,
    Image = 2,
    Video = 3,
    File = 4,
    System = 5,
    Sticker = 6,
    ContactCard = 7,
    Location = 8,
    Link = 9,
    Forward = 10,
    /// 红包（PrivChat Money Message，PLATFORM-only）。payload 只带 redPacketId + 展示快照；
    /// 资金真相在 application/payment。SDK 只搬运消息不碰资金。见 RED_PACKET_AND_TRANSFER_DESIGN_SPEC。
    RedPacket = 11,
    /// 转账（PrivChat Money Message，PLATFORM-only）。payload 只带 transferId + 展示快照。
    MoneyTransfer = 12,
}

impl ContentMessageType {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "text" => Some(ContentMessageType::Text),
            "voice" => Some(ContentMessageType::Voice),
            "image" => Some(ContentMessageType::Image),
            "video" => Some(ContentMessageType::Video),
            "file" => Some(ContentMessageType::File),
            "system" => Some(ContentMessageType::System),
            "sticker" => Some(ContentMessageType::Sticker),
            "contact_card" | "contact" => Some(ContentMessageType::ContactCard),
            "location" => Some(ContentMessageType::Location),
            "link" => Some(ContentMessageType::Link),
            "forward" => Some(ContentMessageType::Forward),
            "red_packet" => Some(ContentMessageType::RedPacket),
            "money_transfer" => Some(ContentMessageType::MoneyTransfer),
            _ => None,
        }
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(ContentMessageType::Text),
            1 => Some(ContentMessageType::Voice),
            2 => Some(ContentMessageType::Image),
            3 => Some(ContentMessageType::Video),
            4 => Some(ContentMessageType::File),
            5 => Some(ContentMessageType::System),
            6 => Some(ContentMessageType::Sticker),
            7 => Some(ContentMessageType::ContactCard),
            8 => Some(ContentMessageType::Location),
            9 => Some(ContentMessageType::Link),
            10 => Some(ContentMessageType::Forward),
            11 => Some(ContentMessageType::RedPacket),
            12 => Some(ContentMessageType::MoneyTransfer),
            _ => None,
        }
    }

    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ContentMessageType::Text => "text",
            ContentMessageType::Voice => "voice",
            ContentMessageType::Image => "image",
            ContentMessageType::Video => "video",
            ContentMessageType::File => "file",
            ContentMessageType::System => "system",
            ContentMessageType::Sticker => "sticker",
            ContentMessageType::ContactCard => "contact_card",
            ContentMessageType::Location => "location",
            ContentMessageType::Link => "link",
            ContentMessageType::Forward => "forward",
            ContentMessageType::RedPacket => "red_packet",
            ContentMessageType::MoneyTransfer => "money_transfer",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LocalAttachmentMetadata;

    #[test]
    fn local_attachment_metadata_has_protocol_owned_field_names() {
        let metadata = LocalAttachmentMetadata {
            file_name: "clip.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
            caption: Some("周末爬山".to_string()),
            duration: Some(7),
            width: Some(1920),
            height: Some(1080),
            thumbnail_width: None,
            thumbnail_height: None,
        };
        let encoded = serde_json::to_value(metadata).expect("serialize metadata");
        assert_eq!(encoded["file_name"], "clip.mp4");
        assert_eq!(encoded["mime_type"], "video/mp4");
        assert_eq!(encoded["duration"], 7);
        assert!(encoded["thumbnail_width"].is_null());
        assert_eq!(encoded["caption"], "周末爬山");
    }

    /// 没有说明文字的附件不能在 JSON 里留一个 `caption: null`：
    /// 接收端按「有没有这个键」判断要不要用它顶掉 `[图片]` 占位文案。
    #[test]
    fn a_missing_caption_is_absent_not_null() {
        let metadata = LocalAttachmentMetadata {
            file_name: "a.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            caption: None,
            duration: None,
            width: None,
            height: None,
            thumbnail_width: None,
            thumbnail_height: None,
        };
        let encoded = serde_json::to_value(metadata).expect("serialize metadata");
        assert!(encoded.get("caption").is_none());
    }
}

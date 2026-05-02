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
}

impl ContentMessageType {
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
        }
    }
}

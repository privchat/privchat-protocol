//! Message payload envelope — strongly typed, FlatBuffers-encoded.
//!
//! Replaces the legacy `serde_json::Value`-based `MessagePayloadEnvelope`
//! that used to live in `crate::message`. The wire format is FlatBuffers;
//! the union dispatches per-type metadata so receivers no longer need to
//! re-decode JSON inside the payload.

use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------
// Per-type metadata structs
// ------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub file_id: u64,
    pub url: Option<String>,
    // width/height 由发送端可选填（SDK 的 attachment_content 不一定带）。必须 #[serde(default)]，
    // 否则 from_json_value(Image, attachment_json) 因缺字段失败 → metadata=None → 服务端
    // 校验报 20006 MessageContentInvalid。file_id 仍必填。
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    /// 缩略图独立 file_id（0/None=无）。接收端走 `thumbnail_file_id -> file/get_url -> cek`
    /// 下载解密，与主文件同一流程。CEK 不进 metadata。
    #[serde(default)]
    pub thumbnail_file_id: Option<u64>,
    /// legacy 明文缩略图 url；v1 加密附件无此字段，仅历史明文 fallback。
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    /// 原文件名（图片一般不展示，保留以备转发/下载）。
    #[serde(default, alias = "filename")]
    pub file_name: Option<String>,
}

/// 文件展示语义随协议传输：file_id + file_name + size + mime 构成完整文件引用，
/// 离线/历史/搜索/转发/通知都直接用；下载仍走 file_id -> file/get_url -> cek。CEK 不进 metadata。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: u64,
    #[serde(default, alias = "filename")]
    pub file_name: Option<String>,
    #[serde(default)]
    pub file_size: u64,
    #[serde(default)]
    pub mime_type: Option<String>,
}

/// Voice/audio bubble metadata. `duration` drives UI bubble width.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VoiceMetadata {
    pub file_id: u64,
    #[serde(default)]
    pub duration: u32,
    #[serde(default, alias = "filename")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub file_id: u64,
    #[serde(default)]
    pub duration: u32,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    pub thumbnail_file_id: Option<u64>,
    pub thumbnail_width: Option<u32>,
    pub thumbnail_height: Option<u32>,
    /// legacy 明文缩略图 url；v1 加密走 thumbnail_file_id -> file/get_url -> cek。
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    /// 原文件名（作为文件发送时展示用）。
    #[serde(default, alias = "filename")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LocationMetadata {
    pub latitude: f64,
    pub longitude: f64,
    pub coordinate_system: Option<String>,
    pub name: Option<String>,
    pub address: Option<String>,
    pub poi_id: Option<String>,
    pub poi_source: Option<String>,
    pub thumbnail_file_id: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContactCardMetadata {
    pub user_id: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StickerMetadata {
    pub sticker_id: String,
    pub image_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ForwardMessageRef {
    pub message_id: Option<u64>,
    pub content: Option<String>,
    /// Opaque JSON bytes for forward extension data. Empty == none.
    pub extra: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ForwardMetadata {
    pub messages: Vec<ForwardMessageRef>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LinkMetadata {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail_file_id: Option<u64>,
}

// ------------------------------------------------------------------
// Metadata enum (mirrors fbs union — NONE = no metadata, i.e. text/system)
// ------------------------------------------------------------------

/// Strongly-typed message metadata. The `tag` discriminator emitted by the
/// serde JSON shape (`{"type":"image", ...}`) is for debug / logging only;
/// the canonical wire format is FlatBuffers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageMetadata {
    Image(ImageMetadata),
    File(FileMetadata),
    Voice(VoiceMetadata),
    Video(VideoMetadata),
    Location(LocationMetadata),
    ContactCard(ContactCardMetadata),
    Sticker(StickerMetadata),
    Forward(ForwardMetadata),
    Link(LinkMetadata),
}

// ------------------------------------------------------------------
// Source descriptor for stranger messages
// ------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MessageSource {
    /// "search" | "group" | "card_share" | "qrcode" | "phone"
    #[serde(rename = "type")]
    pub source_type: String,
    pub source_id: String,
}

// ------------------------------------------------------------------
// Top-level envelope
// ------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MessagePayloadEnvelope {
    /// Display content (text body for text; caption for media).
    #[serde(default)]
    pub content: String,
    /// Type-specific metadata. `None` for text / system messages.
    pub metadata: Option<MessageMetadata>,
    /// Replied-to server_message_id; `None` if not a reply.
    pub reply_to_message_id: Option<u64>,
    /// Mentioned user IDs; empty list = no mentions.
    #[serde(default)]
    pub mentioned_user_ids: Vec<u64>,
    /// Stranger-message source descriptor; absent for friend messages.
    pub message_source: Option<MessageSource>,
}

// ------------------------------------------------------------------
// Bridge to/from the legacy Value-based JSON envelope
// ------------------------------------------------------------------

impl MessageMetadata {
    /// Return every file reference that must be attached to the message row.
    ///
    /// This is shared by all server-side message entry points so that a
    /// typed message cannot silently skip ownership binding when it is
    /// created outside `sync/submit`.
    pub fn attachment_file_ids(&self) -> Vec<u64> {
        let mut ids = Vec::new();
        match self {
            MessageMetadata::Image(value) => {
                ids.push(value.file_id);
                if let Some(id) = value.thumbnail_file_id {
                    ids.push(id);
                }
            }
            MessageMetadata::File(value) => ids.push(value.file_id),
            MessageMetadata::Voice(value) => ids.push(value.file_id),
            MessageMetadata::Video(value) => {
                ids.push(value.file_id);
                if let Some(id) = value.thumbnail_file_id {
                    ids.push(id);
                }
            }
            MessageMetadata::Location(value) => {
                if let Some(id) = value.thumbnail_file_id {
                    ids.push(id);
                }
            }
            MessageMetadata::Link(value) => {
                if let Some(id) = value.thumbnail_file_id {
                    ids.push(id);
                }
            }
            MessageMetadata::ContactCard(_)
            | MessageMetadata::Sticker(_)
            | MessageMetadata::Forward(_) => {}
        }
        ids.retain(|id| *id != 0);
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Build a typed metadata variant from a JSON `Value` plus the
    /// content-type discriminator. Used when bridging legacy data
    /// (SDK local DB rows, FFI inputs) into the wire-canonical struct.
    ///
    /// Returns `None` when:
    ///   - `content_type` carries no metadata (Text / System), or
    ///   - the JSON value cannot be decoded as the expected variant.
    pub fn from_json_value(
        content_type: crate::message::ContentMessageType,
        value: &serde_json::Value,
    ) -> Option<MessageMetadata> {
        use crate::message::ContentMessageType::*;
        match content_type {
            Text | System => None,
            // Money Message：payload 是引用 + 展示快照（不透明 JSON），SDK 不解析成 typed metadata
            // （只搬运，资金真相在 application/payment）。渲染由产品端按快照做。
            RedPacket | MoneyTransfer => None,
            Image => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::Image),
            File => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::File),
            Voice => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::Voice),
            Video => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::Video),
            Location => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::Location),
            ContactCard => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::ContactCard),
            Sticker => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::Sticker),
            Forward => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::Forward),
            Link => serde_json::from_value(value.clone())
                .ok()
                .map(MessageMetadata::Link),
        }
    }

    /// Inverse of [`from_json_value`]: serialize the inner metadata struct
    /// to JSON **without** the `"type"` discriminator tag, matching the
    /// legacy local-DB shape (`{"file_id":...}` not
    /// `{"type":"image","file_id":...}`).
    pub fn to_inner_json_value(&self) -> serde_json::Value {
        match self {
            MessageMetadata::Image(m) => serde_json::to_value(m).unwrap_or(serde_json::Value::Null),
            MessageMetadata::File(m) => serde_json::to_value(m).unwrap_or(serde_json::Value::Null),
            MessageMetadata::Voice(m) => serde_json::to_value(m).unwrap_or(serde_json::Value::Null),
            MessageMetadata::Video(m) => serde_json::to_value(m).unwrap_or(serde_json::Value::Null),
            MessageMetadata::Location(m) => {
                serde_json::to_value(m).unwrap_or(serde_json::Value::Null)
            }
            MessageMetadata::ContactCard(m) => {
                serde_json::to_value(m).unwrap_or(serde_json::Value::Null)
            }
            MessageMetadata::Sticker(m) => {
                serde_json::to_value(m).unwrap_or(serde_json::Value::Null)
            }
            MessageMetadata::Forward(m) => {
                serde_json::to_value(m).unwrap_or(serde_json::Value::Null)
            }
            MessageMetadata::Link(m) => serde_json::to_value(m).unwrap_or(serde_json::Value::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_file_ids_are_complete_deduplicated_and_zero_free() {
        let metadata = MessageMetadata::Image(ImageMetadata {
            file_id: 42,
            thumbnail_file_id: Some(42),
            ..Default::default()
        });
        assert_eq!(metadata.attachment_file_ids(), vec![42]);

        let metadata = MessageMetadata::Video(VideoMetadata {
            file_id: 0,
            thumbnail_file_id: Some(99),
            ..Default::default()
        });
        assert_eq!(metadata.attachment_file_ids(), vec![99]);

        assert!(
            MessageMetadata::ContactCard(ContactCardMetadata { user_id: 7 })
                .attachment_file_ids()
                .is_empty()
        );
    }
}

impl MessagePayloadEnvelope {
    /// Convert from the legacy JSON envelope used by SDK local persistence
    /// and FFI inputs. The `content_type` discriminator tells us which
    /// metadata variant to decode the legacy `Option<Value>` into.
    pub fn from_legacy(
        legacy: &crate::message::LocalMessagePayloadEnvelope,
        content_type: crate::message::ContentMessageType,
    ) -> Self {
        let metadata = legacy
            .metadata
            .as_ref()
            .and_then(|v| MessageMetadata::from_json_value(content_type, v));
        let reply_to_message_id = legacy
            .reply_to_message_id
            .as_ref()
            .and_then(|s| s.parse::<u64>().ok());
        let mentioned_user_ids = legacy.mentioned_user_ids.clone().unwrap_or_default();
        Self {
            content: legacy.content.clone(),
            metadata,
            reply_to_message_id,
            mentioned_user_ids,
            message_source: legacy.message_source.clone(),
        }
    }

    /// Inverse of [`from_legacy`]. Used by the server when downstream code
    /// still expects the legacy `Option<String>` JSON metadata shape.
    pub fn to_legacy(&self) -> crate::message::LocalMessagePayloadEnvelope {
        crate::message::LocalMessagePayloadEnvelope {
            content: self.content.clone(),
            metadata: self.metadata.as_ref().map(|m| m.to_inner_json_value()),
            reply_to_message_id: self.reply_to_message_id.map(|n| n.to_string()),
            mentioned_user_ids: if self.mentioned_user_ids.is_empty() {
                None
            } else {
                Some(self.mentioned_user_ids.clone())
            },
            message_source: self.message_source.clone(),
        }
    }
}

// ------------------------------------------------------------------
// FlatBuffers codec — metadata variants
// ------------------------------------------------------------------

fn encode_image<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &ImageMetadata,
) -> flatbuffers::WIPOffset<fb::ImageMetadata<'a>> {
    let url = m.url.as_ref().map(|s| builder.create_string(s));
    let thumbnail_url = m.thumbnail_url.as_ref().map(|s| builder.create_string(s));
    let file_name = m.file_name.as_ref().map(|s| builder.create_string(s));
    fb::ImageMetadata::create(
        builder,
        &fb::ImageMetadataArgs {
            file_id: m.file_id,
            url,
            width: m.width,
            height: m.height,
            thumbnail_file_id: m.thumbnail_file_id.unwrap_or(0),
            thumbnail_url,
            file_name,
        },
    )
}
fn decode_image(v: fb::ImageMetadata<'_>) -> ImageMetadata {
    let thumb_id = v.thumbnail_file_id();
    ImageMetadata {
        file_id: v.file_id(),
        url: v.url().map(|s| s.to_string()),
        width: v.width(),
        height: v.height(),
        thumbnail_file_id: if thumb_id == 0 { None } else { Some(thumb_id) },
        thumbnail_url: v.thumbnail_url().map(|s| s.to_string()),
        file_name: v.file_name().map(|s| s.to_string()),
    }
}

fn encode_file<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &FileMetadata,
) -> flatbuffers::WIPOffset<fb::FileMetadata<'a>> {
    let file_name = m.file_name.as_ref().map(|s| builder.create_string(s));
    let mime_type = m.mime_type.as_ref().map(|s| builder.create_string(s));
    fb::FileMetadata::create(
        builder,
        &fb::FileMetadataArgs {
            file_id: m.file_id,
            file_name,
            file_size: m.file_size,
            mime_type,
        },
    )
}
fn decode_file(v: fb::FileMetadata<'_>) -> FileMetadata {
    FileMetadata {
        file_id: v.file_id(),
        file_name: v.file_name().map(|s| s.to_string()),
        file_size: v.file_size(),
        mime_type: v.mime_type().map(|s| s.to_string()),
    }
}

fn encode_voice<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &VoiceMetadata,
) -> flatbuffers::WIPOffset<fb::VoiceMetadata<'a>> {
    let file_name = m.file_name.as_ref().map(|s| builder.create_string(s));
    fb::VoiceMetadata::create(
        builder,
        &fb::VoiceMetadataArgs {
            file_id: m.file_id,
            duration: m.duration,
            file_name,
        },
    )
}
fn decode_voice(v: fb::VoiceMetadata<'_>) -> VoiceMetadata {
    VoiceMetadata {
        file_id: v.file_id(),
        duration: v.duration(),
        file_name: v.file_name().map(|s| s.to_string()),
    }
}

fn encode_video<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &VideoMetadata,
) -> flatbuffers::WIPOffset<fb::VideoMetadata<'a>> {
    let thumbnail_url = m.thumbnail_url.as_ref().map(|s| builder.create_string(s));
    let file_name = m.file_name.as_ref().map(|s| builder.create_string(s));
    fb::VideoMetadata::create(
        builder,
        &fb::VideoMetadataArgs {
            file_id: m.file_id,
            duration: m.duration,
            width: m.width,
            height: m.height,
            thumbnail_file_id: m.thumbnail_file_id.unwrap_or(0),
            thumbnail_width: m.thumbnail_width.unwrap_or(0),
            thumbnail_height: m.thumbnail_height.unwrap_or(0),
            thumbnail_url,
            file_name,
        },
    )
}
fn decode_video(v: fb::VideoMetadata<'_>) -> VideoMetadata {
    let to_opt_u64 = |n: u64| if n == 0 { None } else { Some(n) };
    let to_opt_u32 = |n: u32| if n == 0 { None } else { Some(n) };
    VideoMetadata {
        file_id: v.file_id(),
        duration: v.duration(),
        width: v.width(),
        height: v.height(),
        thumbnail_file_id: to_opt_u64(v.thumbnail_file_id()),
        thumbnail_width: to_opt_u32(v.thumbnail_width()),
        thumbnail_height: to_opt_u32(v.thumbnail_height()),
        thumbnail_url: v.thumbnail_url().map(|s| s.to_string()),
        file_name: v.file_name().map(|s| s.to_string()),
    }
}

fn encode_location<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &LocationMetadata,
) -> flatbuffers::WIPOffset<fb::LocationMetadata<'a>> {
    let coordinate_system = m
        .coordinate_system
        .as_ref()
        .map(|s| builder.create_string(s));
    let name = m.name.as_ref().map(|s| builder.create_string(s));
    let address = m.address.as_ref().map(|s| builder.create_string(s));
    let poi_id = m.poi_id.as_ref().map(|s| builder.create_string(s));
    let poi_source = m.poi_source.as_ref().map(|s| builder.create_string(s));
    fb::LocationMetadata::create(
        builder,
        &fb::LocationMetadataArgs {
            latitude: m.latitude,
            longitude: m.longitude,
            coordinate_system,
            name,
            address,
            poi_id,
            poi_source,
            thumbnail_file_id: m.thumbnail_file_id.unwrap_or(0),
        },
    )
}
fn decode_location(v: fb::LocationMetadata<'_>) -> LocationMetadata {
    let thumbnail_file_id = match v.thumbnail_file_id() {
        0 => None,
        n => Some(n),
    };
    LocationMetadata {
        latitude: v.latitude(),
        longitude: v.longitude(),
        coordinate_system: v.coordinate_system().map(|s| s.to_string()),
        name: v.name().map(|s| s.to_string()),
        address: v.address().map(|s| s.to_string()),
        poi_id: v.poi_id().map(|s| s.to_string()),
        poi_source: v.poi_source().map(|s| s.to_string()),
        thumbnail_file_id,
    }
}

fn encode_contact<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &ContactCardMetadata,
) -> flatbuffers::WIPOffset<fb::ContactCardMetadata<'a>> {
    fb::ContactCardMetadata::create(builder, &fb::ContactCardMetadataArgs { user_id: m.user_id })
}
fn decode_contact(v: fb::ContactCardMetadata<'_>) -> ContactCardMetadata {
    ContactCardMetadata {
        user_id: v.user_id(),
    }
}

fn encode_sticker<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &StickerMetadata,
) -> flatbuffers::WIPOffset<fb::StickerMetadata<'a>> {
    let sticker_id = builder.create_string(&m.sticker_id);
    let image_url = builder.create_string(&m.image_url);
    fb::StickerMetadata::create(
        builder,
        &fb::StickerMetadataArgs {
            sticker_id: Some(sticker_id),
            image_url: Some(image_url),
        },
    )
}
fn decode_sticker(v: fb::StickerMetadata<'_>) -> StickerMetadata {
    StickerMetadata {
        sticker_id: v.sticker_id().unwrap_or("").to_string(),
        image_url: v.image_url().unwrap_or("").to_string(),
    }
}

fn encode_forward_ref<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    r: &ForwardMessageRef,
) -> flatbuffers::WIPOffset<fb::ForwardMessageRef<'a>> {
    let content = r.content.as_ref().map(|s| builder.create_string(s));
    let extra = builder.create_vector(&r.extra);
    fb::ForwardMessageRef::create(
        builder,
        &fb::ForwardMessageRefArgs {
            message_id: r.message_id.unwrap_or(0),
            content,
            extra: Some(extra),
        },
    )
}
fn decode_forward_ref(v: fb::ForwardMessageRef<'_>) -> ForwardMessageRef {
    let message_id = match v.message_id() {
        0 => None,
        n => Some(n),
    };
    ForwardMessageRef {
        message_id,
        content: v.content().map(|s| s.to_string()),
        extra: v.extra().map(|x| x.bytes().to_vec()).unwrap_or_default(),
    }
}

fn encode_forward<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &ForwardMetadata,
) -> flatbuffers::WIPOffset<fb::ForwardMetadata<'a>> {
    let refs: Vec<_> = m
        .messages
        .iter()
        .map(|r| encode_forward_ref(builder, r))
        .collect();
    let messages = builder.create_vector(&refs);
    fb::ForwardMetadata::create(
        builder,
        &fb::ForwardMetadataArgs {
            messages: Some(messages),
        },
    )
}
fn decode_forward(v: fb::ForwardMetadata<'_>) -> ForwardMetadata {
    let messages = v
        .messages()
        .map(|vec| vec.iter().map(decode_forward_ref).collect())
        .unwrap_or_default();
    ForwardMetadata { messages }
}

fn encode_link<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    m: &LinkMetadata,
) -> flatbuffers::WIPOffset<fb::LinkMetadata<'a>> {
    let url = builder.create_string(&m.url);
    let title = m.title.as_ref().map(|s| builder.create_string(s));
    let description = m.description.as_ref().map(|s| builder.create_string(s));
    fb::LinkMetadata::create(
        builder,
        &fb::LinkMetadataArgs {
            url: Some(url),
            title,
            description,
            thumbnail_file_id: m.thumbnail_file_id.unwrap_or(0),
        },
    )
}
fn decode_link(v: fb::LinkMetadata<'_>) -> LinkMetadata {
    let thumbnail_file_id = match v.thumbnail_file_id() {
        0 => None,
        n => Some(n),
    };
    LinkMetadata {
        url: v.url().unwrap_or("").to_string(),
        title: v.title().map(|s| s.to_string()),
        description: v.description().map(|s| s.to_string()),
        thumbnail_file_id,
    }
}

// ------------------------------------------------------------------
// FlatBuffers codec — message_source + envelope
// ------------------------------------------------------------------

fn encode_source<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    s: &MessageSource,
) -> flatbuffers::WIPOffset<fb::MessageSource<'a>> {
    let source_type = builder.create_string(&s.source_type);
    let source_id = builder.create_string(&s.source_id);
    fb::MessageSource::create(
        builder,
        &fb::MessageSourceArgs {
            source_type: Some(source_type),
            source_id: Some(source_id),
        },
    )
}
fn decode_source(v: fb::MessageSource<'_>) -> MessageSource {
    MessageSource {
        source_type: v.source_type().unwrap_or("").to_string(),
        source_id: v.source_id().unwrap_or("").to_string(),
    }
}

impl FlatBufferMessage for MessagePayloadEnvelope {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let offset = encode_payload_envelope(builder, self);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::MessagePayloadEnvelope>(bytes)?;
        Ok(decode_payload_envelope(view))
    }
}

pub(crate) fn encode_payload_envelope<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    envelope: &MessagePayloadEnvelope,
) -> flatbuffers::WIPOffset<fb::MessagePayloadEnvelope<'a>> {
    let content = builder.create_string(&envelope.content);

    // Build metadata union (discriminator + offset).
    let (metadata_type, metadata_offset) = match &envelope.metadata {
        None => (fb::MessageMetadata::NONE, None),
        Some(MessageMetadata::Image(m)) => (
            fb::MessageMetadata::ImageMetadata,
            Some(encode_image(builder, m).as_union_value()),
        ),
        Some(MessageMetadata::File(m)) => (
            fb::MessageMetadata::FileMetadata,
            Some(encode_file(builder, m).as_union_value()),
        ),
        Some(MessageMetadata::Voice(m)) => (
            fb::MessageMetadata::VoiceMetadata,
            Some(encode_voice(builder, m).as_union_value()),
        ),
        Some(MessageMetadata::Video(m)) => (
            fb::MessageMetadata::VideoMetadata,
            Some(encode_video(builder, m).as_union_value()),
        ),
        Some(MessageMetadata::Location(m)) => (
            fb::MessageMetadata::LocationMetadata,
            Some(encode_location(builder, m).as_union_value()),
        ),
        Some(MessageMetadata::ContactCard(m)) => (
            fb::MessageMetadata::ContactCardMetadata,
            Some(encode_contact(builder, m).as_union_value()),
        ),
        Some(MessageMetadata::Sticker(m)) => (
            fb::MessageMetadata::StickerMetadata,
            Some(encode_sticker(builder, m).as_union_value()),
        ),
        Some(MessageMetadata::Forward(m)) => (
            fb::MessageMetadata::ForwardMetadata,
            Some(encode_forward(builder, m).as_union_value()),
        ),
        Some(MessageMetadata::Link(m)) => (
            fb::MessageMetadata::LinkMetadata,
            Some(encode_link(builder, m).as_union_value()),
        ),
    };

    let mentioned_user_ids = builder.create_vector(&envelope.mentioned_user_ids);
    let message_source = envelope
        .message_source
        .as_ref()
        .map(|s| encode_source(builder, s));

    let args = fb::MessagePayloadEnvelopeArgs {
        content: Some(content),
        metadata_type,
        metadata: metadata_offset,
        reply_to_message_id: envelope.reply_to_message_id.unwrap_or(0),
        mentioned_user_ids: Some(mentioned_user_ids),
        message_source,
    };
    fb::MessagePayloadEnvelope::create(builder, &args)
}

pub(crate) fn decode_payload_envelope(
    view: fb::MessagePayloadEnvelope<'_>,
) -> MessagePayloadEnvelope {
    let metadata = match view.metadata_type() {
        fb::MessageMetadata::ImageMetadata => view
            .metadata_as_image_metadata()
            .map(|m| MessageMetadata::Image(decode_image(m))),
        fb::MessageMetadata::FileMetadata => view
            .metadata_as_file_metadata()
            .map(|m| MessageMetadata::File(decode_file(m))),
        fb::MessageMetadata::VoiceMetadata => view
            .metadata_as_voice_metadata()
            .map(|m| MessageMetadata::Voice(decode_voice(m))),
        fb::MessageMetadata::VideoMetadata => view
            .metadata_as_video_metadata()
            .map(|m| MessageMetadata::Video(decode_video(m))),
        fb::MessageMetadata::LocationMetadata => view
            .metadata_as_location_metadata()
            .map(|m| MessageMetadata::Location(decode_location(m))),
        fb::MessageMetadata::ContactCardMetadata => view
            .metadata_as_contact_card_metadata()
            .map(|m| MessageMetadata::ContactCard(decode_contact(m))),
        fb::MessageMetadata::StickerMetadata => view
            .metadata_as_sticker_metadata()
            .map(|m| MessageMetadata::Sticker(decode_sticker(m))),
        fb::MessageMetadata::ForwardMetadata => view
            .metadata_as_forward_metadata()
            .map(|m| MessageMetadata::Forward(decode_forward(m))),
        fb::MessageMetadata::LinkMetadata => view
            .metadata_as_link_metadata()
            .map(|m| MessageMetadata::Link(decode_link(m))),
        _ => None, // NONE (or future unknown) → no metadata
    };

    let reply_to_message_id = match view.reply_to_message_id() {
        0 => None,
        n => Some(n),
    };

    let mentioned_user_ids = view
        .mentioned_user_ids()
        .map(|v| v.iter().collect())
        .unwrap_or_default();

    MessagePayloadEnvelope {
        content: view.content().unwrap_or("").to_string(),
        metadata,
        reply_to_message_id,
        mentioned_user_ids,
        message_source: view.message_source().map(decode_source),
    }
}

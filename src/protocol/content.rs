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
// file_id 的 string-or-number 兼容
// ------------------------------------------------------------------

/// 历史上 `file_id` / `thumbnail_file_id` 两种写法都发过：JSON 数字和十进制字符串。
///
/// 字符串写法不是随意为之——u64 的雪花 ID 超过 JS 的 2^53，Web/H5 侧走字符串才无损
/// （spec §14.1「JSON 网关统一十进制字符串」）。**两种都必须认**：serde 默认只认数字，
/// 一旦拒绝，整条 metadata 解析失败 → `from_json_value` 返回 None → 附件绑定被整条跳过，
/// 接收端拿不到授权。
mod file_id_compat {
    use serde::{Deserialize, Deserializer};

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        Num(u64),
        Str(String),
    }

    fn parse(v: NumOrStr) -> Result<u64, String> {
        match v {
            NumOrStr::Num(n) => Ok(n),
            NumOrStr::Str(s) => s
                .trim()
                .parse::<u64>()
                .map_err(|_| format!("file_id 既不是数字也不是十进制字符串: {s}")),
        }
    }

    pub fn required<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        parse(NumOrStr::deserialize(d)?).map_err(serde::de::Error::custom)
    }

    pub fn optional<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
        match Option::<NumOrStr>::deserialize(d)? {
            None => Ok(None),
            Some(v) => parse(v).map(Some).map_err(serde::de::Error::custom),
        }
    }
}

// ------------------------------------------------------------------
// Per-type metadata structs
// ------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ImageMetadata {
    #[serde(deserialize_with = "file_id_compat::required")]
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
    #[serde(default, deserialize_with = "file_id_compat::optional")]
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
    #[serde(deserialize_with = "file_id_compat::required")]
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
    #[serde(deserialize_with = "file_id_compat::required")]
    pub file_id: u64,
    #[serde(default)]
    pub duration: u32,
    #[serde(default, alias = "filename")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoMetadata {
    #[serde(deserialize_with = "file_id_compat::required")]
    pub file_id: u64,
    #[serde(default)]
    pub duration: u32,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default, deserialize_with = "file_id_compat::optional")]
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
    #[serde(default, deserialize_with = "file_id_compat::optional")]
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
    #[serde(default, deserialize_with = "file_id_compat::optional")]
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
    /// 转发来源（`MEDIA_REFERENCE_AND_FORWARD_SPEC` §6.2）。
    ///
    /// 🔴 副本必须**自带**来源，不能让客户端回头去查源消息：源消息可能在别的
    /// 会话里、可能已被删除，接收方甚至无权读它。只落库不进投影的话，
    /// 接收端和新登录设备都显示不出「转发自」。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_origin: Option<ForwardOriginSnapshot>,
}

/// 转发来源快照：**最初**作者，不是上一手转发人（对齐微信/Telegram）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardOriginSnapshot {
    /// 最初那条消息；源消息已被物理清除时为 `None`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_message_id: Option<u64>,
    /// 最初作者（快照，不随源消息删除而变）。
    pub root_author_id: u64,
    /// 展示用作者名快照。接收方未必有权读源会话，拿不到就只能显示 uid。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_author_name: Option<String>,
}

// ------------------------------------------------------------------
// Bridge to/from the legacy Value-based JSON envelope
// ------------------------------------------------------------------

/// 一个文件在消息里扮演的角色。数值是**线上/入库值，只追加不复用**
/// （spec `foundation/MEDIA_REFERENCE_AND_FORWARD_SPEC` §14.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(i16)]
pub enum MediaRole {
    /// 主体文件：图片原图 / 视频 / 语音 / 普通文件。
    Original = 0,
    /// 缩略图。**独立的 file_id 与 CEK**——接收端要单独走一次 `file/get_url`
    /// 才能解密，不是主体文件的附属物。
    Thumbnail = 1,
}

impl MediaRole {
    pub fn as_i16(self) -> i16 {
        self as i16
    }
}

/// 消息对一个文件的一次引用。
///
/// `(role, ordinal)` 才是主键的一部分——**同一个 `file_id` 可以同时是 Original
/// 和 Thumbnail**：图片协议要求必带缩略图引用，缩略图未产出时两端 SDK 都会把原图
/// 引用为缩略图。把它当「重复」去掉会让这类图片的缩略图引用整片丢失。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaRef {
    pub file_id: u64,
    pub role: MediaRole,
    /// 同一 role 下的序号。当前每种 role 至多一个，恒为 0；多图消息落地时才会 > 0。
    pub ordinal: i32,
}

impl MessageMetadata {
    /// 这条消息引用的全部文件，**带角色**。
    ///
    /// 按 typed variant 取字段——类型决定哪些字段算附件，而不是在任意 JSON 上
    /// 猜 `file_id` / `thumbnail_file_id`。文本、系统、资金消息即便携带这些字段
    /// 也产生不出引用（它们根本没有对应的 typed variant）。
    ///
    /// 🔴 **保留同一 file_id 的不同 role**，不去重。要去重的 id 列表见
    /// [`Self::unique_file_ids`]。
    pub fn attachment_refs(&self) -> Vec<MediaRef> {
        let mut refs = Vec::new();
        let mut push = |file_id: u64, role: MediaRole| {
            if file_id != 0 {
                refs.push(MediaRef {
                    file_id,
                    role,
                    ordinal: 0,
                });
            }
        };
        match self {
            MessageMetadata::Image(value) => {
                push(value.file_id, MediaRole::Original);
                if let Some(id) = value.thumbnail_file_id {
                    push(id, MediaRole::Thumbnail);
                }
            }
            MessageMetadata::File(value) => push(value.file_id, MediaRole::Original),
            MessageMetadata::Voice(value) => push(value.file_id, MediaRole::Original),
            MessageMetadata::Video(value) => {
                push(value.file_id, MediaRole::Original);
                if let Some(id) = value.thumbnail_file_id {
                    push(id, MediaRole::Thumbnail);
                }
            }
            // 位置/链接只有缩略图，没有主体文件。
            MessageMetadata::Location(value) => {
                if let Some(id) = value.thumbnail_file_id {
                    push(id, MediaRole::Thumbnail);
                }
            }
            MessageMetadata::Link(value) => {
                if let Some(id) = value.thumbnail_file_id {
                    push(id, MediaRole::Thumbnail);
                }
            }
            MessageMetadata::ContactCard(_)
            | MessageMetadata::Sticker(_)
            | MessageMetadata::Forward(_) => {}
        }
        refs
    }

    /// 去重后的 file_id 列表，供**按文件**做的操作使用（所有权绑定守卫、引用计数）。
    ///
    /// 与 [`Self::attachment_refs`] 的区别：那个按「引用」计数（同一文件两个角色 = 两条），
    /// 这个按「文件」计数（同一文件 = 一个）。绑定守卫要的是后者。
    pub fn unique_file_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.attachment_refs().into_iter().map(|r| r.file_id).collect();
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

    /// 同一个 file 同时是原图和缩略图，是**合法形态**：图片协议要求必带缩略图引用，
    /// 缩略图未产出时两端 SDK 都把原图引用为缩略图。两条 role 引用都必须保留——
    /// 当成「重复」丢掉会让这类图片的缩略图引用整片丢失，接收端拿不到授权。
    #[test]
    fn one_file_can_hold_both_roles_and_both_survive() {
        let meta = MessageMetadata::Image(ImageMetadata {
            file_id: 7,
            thumbnail_file_id: Some(7),
            ..Default::default()
        });
        let refs = meta.attachment_refs();
        assert_eq!(refs.len(), 2, "两个角色都要在: {:?}", refs);
        assert_eq!(refs[0].role, MediaRole::Original);
        assert_eq!(refs[1].role, MediaRole::Thumbnail);
        assert!(refs.iter().all(|r| r.file_id == 7));

        // 但按「文件」数只有一个——绑定守卫要的是这个。
        assert_eq!(meta.unique_file_ids(), vec![7]);
    }

    /// 类型决定哪些字段算附件。非媒体类型即使 JSON 里塞了 file_id，
    /// 也产生不出引用——它们根本没有对应的 typed variant。
    #[test]
    fn a_non_media_type_cannot_smuggle_a_file_id() {
        use crate::message::ContentMessageType;
        for kind in [
            ContentMessageType::Text,
            ContentMessageType::System,
            ContentMessageType::RedPacket,
            ContentMessageType::MoneyTransfer,
        ] {
            let forged = serde_json::json!({ "file_id": 999, "thumbnail_file_id": 998 });
            assert!(
                MessageMetadata::from_json_value(kind, &forged).is_none(),
                "{kind:?} 不该解析出 typed metadata"
            );
        }
    }

    /// 位置/链接只有缩略图，没有主体文件——不能凭空造一条 Original。
    #[test]
    fn a_link_only_contributes_a_thumbnail() {
        let meta = MessageMetadata::Link(LinkMetadata {
            thumbnail_file_id: Some(5),
            ..Default::default()
        });
        let refs = meta.attachment_refs();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].role, MediaRole::Thumbnail);
    }

    /// 🔴 `file_id` 的两种写法都必须认：JSON 数字，和十进制字符串。
    ///
    /// 字符串不是随意为之——u64 雪花 ID 超过 JS 的 2^53，Web/H5 走字符串才无损。
    /// serde 默认只认数字，一旦拒绝会让**整条 metadata** 解析失败、附件绑定被
    /// 整条跳过，接收端拿不到授权。收敛解析入口时差点踩进这个坑。
    #[test]
    fn a_file_id_may_arrive_as_a_number_or_as_a_decimal_string() {
        use crate::message::ContentMessageType;
        let from_num = MessageMetadata::from_json_value(
            ContentMessageType::Image,
            &serde_json::json!({ "file_id": 300, "thumbnail_file_id": 400 }),
        );
        let from_str = MessageMetadata::from_json_value(
            ContentMessageType::Image,
            &serde_json::json!({ "file_id": "300", "thumbnail_file_id": "400" }),
        );
        assert_eq!(from_num.as_ref().map(|m| m.unique_file_ids()), Some(vec![300, 400]));
        assert_eq!(from_str.as_ref().map(|m| m.unique_file_ids()), Some(vec![300, 400]));
        assert_eq!(
            from_num.map(|m| m.attachment_refs()),
            from_str.map(|m| m.attachment_refs()),
            "两种写法必须产出完全相同的引用"
        );
    }

    /// 但真的不是数字的字符串仍要拒绝——别把兼容做成「什么都收」。
    #[test]
    fn a_non_numeric_file_id_is_still_rejected() {
        use crate::message::ContentMessageType;
        assert!(MessageMetadata::from_json_value(
            ContentMessageType::Image,
            &serde_json::json!({ "file_id": "not-a-number" }),
        )
        .is_none());
    }

    /// role 数值是入库值，改动会让存量行与新行语义不一致。
    #[test]
    fn media_role_wire_values_are_frozen() {
        assert_eq!(MediaRole::Original.as_i16(), 0);
        assert_eq!(MediaRole::Thumbnail.as_i16(), 1);
    }
    use super::*;

    #[test]
    fn unique_file_ids_are_complete_deduplicated_and_zero_free() {
        let metadata = MessageMetadata::Image(ImageMetadata {
            file_id: 42,
            thumbnail_file_id: Some(42),
            ..Default::default()
        });
        assert_eq!(metadata.unique_file_ids(), vec![42]);

        let metadata = MessageMetadata::Video(VideoMetadata {
            file_id: 0,
            thumbnail_file_id: Some(99),
            ..Default::default()
        });
        assert_eq!(metadata.unique_file_ids(), vec![99]);

        assert!(
            MessageMetadata::ContactCard(ContactCardMetadata { user_id: 7 })
                .unique_file_ids()
                .is_empty()
        );
    }

    /// The legacy JSON path must agree with the FlatBuffers decoder on what
    /// "no reply" looks like. Production payloads carry the stringified
    /// `"null"` (a sender that serialized an absent optional), and `0` is the
    /// wire sentinel — either one leaking through reads downstream as a real
    /// reply anchor and renders an "original unavailable" quote strip.
    #[test]
    fn legacy_reply_anchor_rejects_sentinels_and_junk() {
        let envelope = |reply: Option<&str>| {
            MessagePayloadEnvelope::from_legacy(
                &crate::message::LocalMessagePayloadEnvelope {
                    content: "hi".to_string(),
                    metadata: None,
                    reply_to_message_id: reply.map(str::to_string),
                    mentioned_user_ids: None,
                    message_source: None,
                    forward_origin: None,
                },
                crate::message::ContentMessageType::Text,
            )
            .reply_to_message_id
        };

        for absent in [
            None,
            Some("null"),
            Some("undefined"),
            Some(""),
            Some("0"),
            Some("abc"),
        ] {
            assert_eq!(envelope(absent), None, "expected no anchor for {absent:?}");
        }
        assert_eq!(
            envelope(Some("600997771041832960")),
            Some(600997771041832960)
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
        // `0` is the "no reply" sentinel on the FlatBuffers side (see
        // `decode_payload_envelope`), so the legacy JSON path must agree —
        // otherwise `Some(0)` travels on as a reference to message 0.
        // Non-numeric junk (senders that stringified an absent optional into
        // `"null"`) already falls out via `parse`.
        let reply_to_message_id = legacy
            .reply_to_message_id
            .as_ref()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|id| *id != 0);
        let mentioned_user_ids = legacy.mentioned_user_ids.clone().unwrap_or_default();
        Self {
            content: legacy.content.clone(),
            metadata,
            reply_to_message_id,
            mentioned_user_ids,
            message_source: legacy.message_source.clone(),
            forward_origin: legacy.forward_origin.clone(),
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
            forward_origin: self.forward_origin.clone(),
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
        // FlatBuffers schema 尚未带这个字段；实际投递给客户端的是 JSON 投影，
        // 那条路径会带上来源（见 §6.2）。FB 解码路径拿不到就是 None。
        forward_origin: None,
    }
}

//! 消息类型与 Payload 解析结构体
//!
//! 仅定义已知消息类型，服务端与客户端统一使用本模块定义解析 payload。

use serde::{Deserialize, Serialize};

/// 内容消息类型（u32，仅已知类型）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ContentMessageType {
    /// 文本消息
    Text = 0,
    /// 图片消息
    Image = 1,
    /// 文件消息
    File = 2,
    /// 语音消息
    Voice = 3,
    /// 视频消息
    Video = 4,
    /// 系统消息
    System = 5,
    /// 音频消息
    Audio = 6,
    /// 位置消息
    Location = 7,
    /// 名片消息
    ContactCard = 8,
    /// 表情包消息
    Sticker = 9,
    /// 转发消息
    Forward = 10,
}

impl ContentMessageType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(ContentMessageType::Text),
            1 => Some(ContentMessageType::Image),
            2 => Some(ContentMessageType::File),
            3 => Some(ContentMessageType::Voice),
            4 => Some(ContentMessageType::Video),
            5 => Some(ContentMessageType::System),
            6 => Some(ContentMessageType::Audio),
            7 => Some(ContentMessageType::Location),
            8 => Some(ContentMessageType::ContactCard),
            9 => Some(ContentMessageType::Sticker),
            10 => Some(ContentMessageType::Forward),
            _ => None,
        }
    }

    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// 转换为字符串（用于显示、RPC 等）
    pub fn as_str(self) -> &'static str {
        match self {
            ContentMessageType::Text => "text",
            ContentMessageType::Image => "image",
            ContentMessageType::File => "file",
            ContentMessageType::Voice => "voice",
            ContentMessageType::Video => "video",
            ContentMessageType::System => "system",
            ContentMessageType::Audio => "audio",
            ContentMessageType::Location => "location",
            ContentMessageType::ContactCard => "contact_card",
            ContentMessageType::Sticker => "sticker",
            ContentMessageType::Forward => "forward",
        }
    }
}

// ---------- Payload 顶层信封（与 message_type 无关的公共字段） ----------

/// 消息来源（非好友消息时使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSource {
    pub source_type: String, // "search" | "group" | "card_share" | "qrcode" | "phone"
    pub source_id: String,
}

/// Payload 解析后的顶层结构（content + metadata + 公共扩展）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessagePayloadEnvelope {
    /// 消息显示内容
    pub content: String,
    /// 类型相关元数据，按 ContentMessageType 解析为对应 *Metadata 结构体
    pub metadata: Option<serde_json::Value>,
    /// 引用消息 ID（可选）
    pub reply_to_message_id: Option<String>,
    /// @ 提及的用户 ID 列表（可选）
    pub mentioned_user_ids: Option<Vec<u64>>,
    /// 非好友消息来源（可选）
    pub message_source: Option<MessageSource>,
}

// ---------- 各消息类型对应的 metadata 解析结构体 ----------

/// 图片消息 metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub file_id: u64,
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// 文件 / 语音 / 视频 / 音频消息 metadata（共用 file_id）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLikeMetadata {
    pub file_id: u64,
}

/// 位置消息 metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationMetadata {
    pub latitude: f64,
    pub longitude: f64,
}

/// 名片消息 metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCardMetadata {
    pub user_id: u64,
}

/// 表情包消息 metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerMetadata {
    pub sticker_id: String,
    pub image_url: String,
}

/// 转发单条引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardMessageRef {
    pub message_id: Option<u64>,
    pub content: Option<String>,
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

/// 转发消息 metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardMetadata {
    pub messages: Vec<ForwardMessageRef>,
}

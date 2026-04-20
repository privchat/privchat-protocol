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
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! 消息类型与 Payload 解析结构体
//!
//! 仅定义已知消息类型，服务端与客户端统一使用本模块定义解析 payload。

use serde::{Deserialize, Serialize};

/// 内容消息类型（u32，仅已知类型）
///
/// 编号按业务优先级顺序分配（0=文字 最常用 … 10=转发 最少用）。
/// 未知值由客户端按 Unknown 兜底渲染。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ContentMessageType {
    /// 文本消息
    Text = 0,
    /// 语音消息（IM 录音气泡，携带 duration）
    Voice = 1,
    /// 图片消息
    Image = 2,
    /// 视频消息（携带时长与尺寸、可选缩略图）
    Video = 3,
    /// 文件消息（普通文件，包括音频文件；由发送入口决定，不由 MIME 反推）
    File = 4,
    /// 系统消息
    System = 5,
    /// 表情包消息
    Sticker = 6,
    /// 名片消息
    ContactCard = 7,
    /// 位置消息
    Location = 8,
    /// 网址预览消息（URL + 标题/描述/缩略图）
    Link = 9,
    /// 转发消息
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

    /// 转换为字符串（用于显示、RPC 等）
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

// ---------- Payload 顶层信封（与 message_type 无关的公共字段） ----------

/// 消息来源（非好友消息时使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSource {
    #[serde(rename = "type")]
    pub source_type: String, // "search" | "group" | "card_share" | "qrcode" | "phone"
    pub source_id: String,
}

/// Payload 解析后的顶层结构（content + metadata + 公共扩展）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessagePayloadEnvelope {
    /// 消息显示内容
    #[serde(default)]
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

/// 文件消息 metadata（含音频文件等普通文件）
///
/// 语音、视频不复用本结构：它们有独立的 UI 呈现需求，分别见 [`VoiceMetadata`] 与 [`VideoMetadata`]。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: u64,
}

/// 语音消息 metadata
///
/// 语音消息区别于普通文件：客户端 UI 依赖 `duration` 渲染气泡宽度、时长文案与播放进度，
/// 因此协议层必须独立承载这个字段，不能回落到 [`FileMetadata`]。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMetadata {
    pub file_id: u64,
    /// 语音时长（秒）。录制不足 1 秒按客户端约定上取整为 1。
    pub duration: u32,
}

/// 视频消息 metadata
///
/// 视频气泡需要展示首帧/缩略图、宽高比、时长徽标，协议层独立承载以避免和普通文件混淆。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub file_id: u64,
    /// 视频时长（秒）
    pub duration: u32,
    /// 视频宽度（像素）
    pub width: u32,
    /// 视频高度（像素）
    pub height: u32,
    /// 缩略图文件 ID（可选，未生成时由客户端播放器自行抽帧）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file_id: Option<u64>,
    /// 缩略图宽度（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_width: Option<u32>,
    /// 缩略图高度（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_height: Option<u32>,
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

/// 网址预览消息 metadata
///
/// `thumbnail_file_id` 由 SDK 应用层通过预览回调（类似视频 `VideoProcessHook`）抓取网页
/// 并上传后填充；若宿主未注册回调，则保持 `None`，客户端 UI 显示空缩略图占位。
/// 服务端不参与抓取，只转发客户端填入的预览结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMetadata {
    /// 目标 URL（必填）
    pub url: String,
    /// 网页标题（可选，由 SDK 应用层回调填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 网页描述（可选，由 SDK 应用层回调填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 缩略图文件 ID（可选，SDK 回调上传后填充；未注册回调时为空）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file_id: Option<u64>,
}

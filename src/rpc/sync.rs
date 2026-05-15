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

/// pts-Based 同步协议
///
/// 设计原则：
/// - pts（per-channel monotonic）为权威顺序
/// - local_message_id 用于幂等和关联
/// - 服务器是唯一仲裁方
use serde::{Deserialize, Serialize};

// ============================================================
// JSON wire helpers — u64 ↔ string
// ============================================================
//
// JavaScript's `JSON.parse` decodes JSON `Number` to `f64`, which loses
// precision for snowflake u64 values above 2^53. The fix is to carry
// large u64 ids as JSON strings on the wire while keeping them as `u64`
// in Rust memory. Apply via `#[serde(with = "u64_str")]` /
// `#[serde(with = "option_u64_str")]` on the targeted fields.
//
// Currently used only by the `sync/get_difference` family
// (GetDifferenceRequest / GetDifferenceResponse / ServerCommit). Other
// JSON RPC structs that emit u64s as numbers are tracked separately —
// expand this attribute usage once each is migrated.

mod u64_str {
    use serde::{de::Error as DeError, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse::<u64>().map_err(D::Error::custom)
    }
}

mod option_u64_str {
    use serde::{de::Error as DeError, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(
        value: &Option<u64>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => serializer.serialize_str(&v.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<u64>, D::Error> {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => s.parse::<u64>().map(Some).map_err(D::Error::custom),
            None => Ok(None),
        }
    }
}

// ============================================================
// SessionReady - 客户端声明已完成 bootstrap
// ============================================================

/// 会话 ready 请求
///
/// RPC 路由: `sync/session_ready`
/// 语义：客户端 bootstrap sync 已完成，服务端可开始补差+实时推送
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionReadyRequest {}

/// 会话 ready 响应
///
/// RPC 路由: `sync/session_ready`
pub type SessionReadyResponse = bool;

// ============================================================
// ClientSubmit - 客户端提交命令
// ============================================================

/// 客户端提交请求
///
/// RPC路由: `sync/submit`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSubmitRequest {
    /// 客户端消息号（Snowflake u64，用于幂等）
    pub local_message_id: u64,

    /// 频道 ID
    pub channel_id: u64,

    /// 频道类型（1=私聊，2=群聊）
    pub channel_type: u8,

    /// 客户端已知的最后 pts（用于间隙检测）
    pub last_pts: u64,

    /// 命令类型
    pub command_type: String,

    /// 命令负载（JSON）
    pub payload: serde_json::Value,

    /// 客户端时间戳（毫秒）
    pub client_timestamp: i64,

    /// 设备 ID（可选，用于多设备去重）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

/// 服务器提交响应
///
/// RPC路由: `sync/submit`
/// 注意：如果 decision 是 Rejected，SDK 层会返回错误，不会反序列化这个结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSubmitResponse {
    /// 服务器决策
    pub decision: ServerDecision,

    /// 分配的 pts（如果 accepted/transformed）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pts: Option<u64>,

    /// 服务器消息 ID（如果 accepted/transformed）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_msg_id: Option<u64>,

    /// 服务器时间戳
    pub server_timestamp: i64,

    /// 关联的 local_message_id
    pub local_message_id: u64,

    /// 是否需要补齐（has_gap）
    pub has_gap: bool,

    /// 服务器当前最新 pts
    pub current_pts: u64,
}

/// 服务器决策类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerDecision {
    /// 接受（原样接受）
    Accepted,
    /// 转换（服务器修改了某些字段）
    Transformed {
        /// 转换说明
        reason: String,
    },
    /// 拒绝（不符合规则）
    Rejected {
        /// 拒绝原因
        reason: String,
    },
}

// ============================================================
// GetDifference - 获取差异（补齐间隙）
// ============================================================

/// 获取差异请求
///
/// RPC路由: `sync/get_difference`
///
/// JSON wire: `channel_id` and `last_pts` are carried as **strings** to
/// preserve u64 precision against JS `JSON.parse` rounding above 2^53.
/// Rust callers read/write them as `u64` as usual; the conversion is
/// driven by serde at the (de)serialize boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDifferenceRequest {
    /// 频道 ID
    #[serde(with = "u64_str")]
    pub channel_id: u64,

    /// 频道类型
    pub channel_type: u8,

    /// 客户端已知的最后 pts
    #[serde(with = "u64_str")]
    pub last_pts: u64,

    /// 限制数量（可选，默认 100）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// 获取差异响应
///
/// RPC路由: `sync/get_difference`
///
/// JSON wire: `current_pts` is a string. See `GetDifferenceRequest` for
/// rationale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDifferenceResponse {
    /// Commits 列表（pts 递增）
    pub commits: Vec<ServerCommit>,

    /// 服务器当前最新 pts
    #[serde(with = "u64_str")]
    pub current_pts: u64,

    /// 是否还有更多（需要继续拉取）
    pub has_more: bool,
}

/// 服务器 Commit（权威事实）
///
/// JSON wire: every u64 id (`pts`, `server_msg_id`, `local_message_id`,
/// `channel_id`, `sender_id`) is a string. `channel_type` (u8),
/// `message_type` (string), `content` (json), and `server_timestamp`
/// (i64 millis — well below 2^53 for any plausible date) stay native.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCommit {
    /// pts（per-channel 单调递增）
    #[serde(with = "u64_str")]
    pub pts: u64,

    /// 服务器消息 ID
    #[serde(with = "u64_str")]
    pub server_msg_id: u64,

    /// 关联的 local_message_id（如果来自客户端）
    #[serde(default, skip_serializing_if = "Option::is_none", with = "option_u64_str")]
    pub local_message_id: Option<u64>,

    /// 频道 ID
    #[serde(with = "u64_str")]
    pub channel_id: u64,

    /// 频道类型
    pub channel_type: u8,

    /// 消息类型
    pub message_type: String,

    /// 消息内容（JSON）
    pub content: serde_json::Value,

    /// 服务器时间戳（毫秒）
    pub server_timestamp: i64,

    /// 发送者 ID
    #[serde(with = "u64_str")]
    pub sender_id: u64,

    /// 发送者信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_info: Option<SenderInfo>,
}

/// 发送者信息（简化的用户信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderInfo {
    pub user_id: u64,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

// ============================================================
// GetChannelPts - 获取频道当前 pts（用于初始化）
// ============================================================

/// 获取频道 pts 请求
///
/// RPC路由: `sync/get_channel_pts`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetChannelPtsRequest {
    /// 频道 ID
    pub channel_id: u64,

    /// 频道类型
    pub channel_type: u8,
}

/// 获取频道 pts 响应
///
/// RPC路由: `sync/get_channel_pts`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetChannelPtsResponse {
    /// 当前 pts
    pub current_pts: u64,
}

// ============================================================
// BatchGetChannelPts - 批量获取多个频道的 pts
// ============================================================

/// 批量获取频道 pts 请求
///
/// RPC路由: `sync/batch_get_channel_pts`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGetChannelPtsRequest {
    /// 频道列表
    pub channels: Vec<ChannelIdentifier>,
}

/// 频道标识符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIdentifier {
    pub channel_id: u64,
    pub channel_type: u8,
}

/// 批量获取频道 pts 响应
///
/// RPC路由: `sync/batch_get_channel_pts`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGetChannelPtsResponse {
    /// 频道 pts 映射
    pub channel_pts_map: Vec<ChannelPtsInfo>,
}

/// 频道 pts 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPtsInfo {
    pub channel_id: u64,
    pub channel_type: u8,
    pub current_pts: u64,
}

// ============================================================
// Entity State Sync（实体状态同步，与 PTS 消息流正交）
// ============================================================
// 设计见 privchat-docs/design/ENTITY_SYNC_V1.md

/// 实体同步请求
///
/// RPC 路由: `entity/sync_entities`（待服务端实现）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntitiesRequest {
    /// 实体类型：friend, group, channel, group_member, user, user_block 等（受控枚举）
    pub entity_type: String,
    /// 客户端上次同步到的版本号，0 或空表示全量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_version: Option<u64>,
    /// 可选：同步范围，如 group_member 需带 group_id；默认走集合型增量同步
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// 可选：每页数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// 单条实体同步项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntityItem {
    /// 实体 ID（如 user_id, group_id, channel_id）
    pub entity_id: String,
    /// 该实体最新版本号
    pub version: u64,
    /// true 表示服务端已删除（Tombstone）
    #[serde(default)]
    pub deleted: bool,
    /// 实体数据，具体字段随 entity_type 变化
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FriendSyncFriendPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FriendSyncUserPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<i32>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FriendSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub friend: Option<FriendSyncFriendPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<FriendSyncUserPayload>,
    // -------- F-sync.1 扩展（friend request 多端同步）--------
    //
    // 既有客户端不感知这些字段（全部 Option，缺省即旧行为）；新客户端按 status
    // 区分"已成立的好友" vs "处于某种请求态的好友申请"。
    /// 数据库 status 整型值。
    /// 0=pending / 1=accepted / 2=blocked / 3=rejected / 4=recalled / 5=expired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i16>,
    /// 在当前 viewer 视角下这条记录是否是"我发出的"。
    /// `true` = viewer 是 requester（friendships.user_id == viewer）；
    /// `false` = viewer 是 receiver（friendships.friend_id == viewer）。
    /// status=accepted 时此字段无意义，server 可不填。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_outgoing: Option<bool>,
    /// 申请附言（apply 时由 requester 填，常驻原行直到清理）。pending 列表 UI 用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_message: Option<String>,
    /// 申请来源类型（"search" / "phone" / "qrcode" / "group" / "card_share" /
    /// "conversation" 等，见 model::privacy::FriendRequestSource）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_source: Option<String>,
    /// 来源 ID（如 channel_id / group_id / 搜索 session_id 等）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_source_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<i64>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_msg_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_msg_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelExtraSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<i64>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browse_to: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_pts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_offset_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_updated_at: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelUnreadSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<i64>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread_count: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupMemberSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joined_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelMemberSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<i32>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_invite_uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inviter_uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_deleted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub robot: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbidden_expiration_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_avatar_cache_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_message_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_message_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<i32>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setting: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_seq: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchable_word: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_seq: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_flag: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageStatusSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_message_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<i32>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_read: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelReadCursorSyncPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<i32>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reader_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_read_pts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

/// 实体同步响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntitiesResponse {
    pub items: Vec<SyncEntityItem>,
    /// 本次同步完成后的最新版本号，客户端下次请求 since_version
    pub next_version: u64,
    /// 是否还有更多数据
    #[serde(default)]
    pub has_more: bool,
    /// 可选：客户端过旧时服务端返回最小支持版本，触发全量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<u64>,
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_submit_request_serialization() {
        let req = ClientSubmitRequest {
            local_message_id: 123456789,
            channel_id: 1001,
            channel_type: 1,
            last_pts: 100,
            command_type: "send_message".to_string(),
            payload: serde_json::json!({"text": "Hello"}),
            client_timestamp: 1700000000000,
            device_id: Some("device_001".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("local_message_id"));
        assert!(json.contains("123456789"));
    }

    #[test]
    fn test_server_decision() {
        let decision_accepted = ServerDecision::Accepted;
        let decision_transformed = ServerDecision::Transformed {
            reason: "Content filtered".to_string(),
        };
        let decision_rejected = ServerDecision::Rejected {
            reason: "Spam detected".to_string(),
        };

        assert_eq!(decision_accepted, ServerDecision::Accepted);
        assert!(matches!(
            decision_transformed,
            ServerDecision::Transformed { .. }
        ));
        assert!(matches!(decision_rejected, ServerDecision::Rejected { .. }));
    }

}

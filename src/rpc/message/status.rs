/// 消息状态相关 RPC（已读回执等）
use serde::{Deserialize, Serialize};

/// 标记消息已读请求
///
/// RPC路由: `message/status/read`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStatusReadRequest {
    /// 频道ID
    pub channel_id: u64,
    /// 服务端消息ID（服务端字段名为 message_id）
    #[serde(alias = "server_message_id")]
    pub message_id: u64,
    /// 用户ID（部分旧服务端会忽略，保持兼容）
    #[serde(default)]
    pub user_id: u64,
}

/// 获取消息已读列表请求
///
/// RPC路由: `message/status/read_list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadListRequest {
    /// 服务端消息ID（服务端字段名为 message_id）
    #[serde(alias = "server_message_id")]
    pub message_id: u64,
    /// 频道ID
    pub channel_id: u64,
}

/// 获取消息已读统计请求
///
/// RPC路由: `message/status/read_stats`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadStatsRequest {
    /// 服务端消息ID（服务端字段名为 message_id）
    #[serde(alias = "server_message_id")]
    pub message_id: u64,
    /// 频道ID
    pub channel_id: u64,
}

/// 获取消息未读数请求
///
/// RPC路由: `message/status/count`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStatusCountRequest {
    /// 频道ID（为空表示查询总未读）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
}

/// 标记消息已读响应
///
/// RPC路由: `message/status/read`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type MessageStatusReadResponse = bool;

/// 获取消息已读列表响应
///
/// RPC路由: `message/status/read_list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadUserEntry {
    pub user_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_at: Option<String>,
}

/// 获取消息已读列表响应
///
/// RPC路由: `message/status/read_list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadListResponse {
    #[serde(default, alias = "read_list")]
    pub readers: Vec<MessageReadUserEntry>,
    #[serde(default, alias = "total_members")]
    pub total: usize,
    #[serde(default)]
    pub read_count: u32,
    #[serde(default)]
    pub unread_count: u32,
}

/// 获取消息已读统计响应
///
/// RPC路由: `message/status/read_stats`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadStatsResponse {
    pub read_count: u32,
    #[serde(default, alias = "total_members")]
    pub total_count: u32,
}

/// 获取消息未读数响应
///
/// RPC路由: `message/status/count`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStatusCountResponse {
    pub unread_count: i32,
    #[serde(default)]
    pub channel_id: Option<String>,
}

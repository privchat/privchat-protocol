/// 消息状态相关 RPC（已读回执等）

use serde::{Deserialize, Serialize};

/// 标记消息已读请求
/// 
/// RPC路由: `message/status/read`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStatusReadRequest {
    /// 用户ID
    pub user_id: u64,
    /// 频道ID
    pub channel_id: u64,
    /// 服务端消息ID
    pub server_message_id: u64,
}

/// 获取消息已读列表请求
/// 
/// RPC路由: `message/status/read_list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadListRequest {
    /// 服务端消息ID
    pub server_message_id: u64,
    /// 频道ID
    pub channel_id: u64,
}

/// 获取消息已读统计请求
/// 
/// RPC路由: `message/status/read_stats`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadStatsRequest {
    /// 服务端消息ID
    pub server_message_id: u64,
    /// 频道ID
    pub channel_id: u64,
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
pub struct MessageReadListResponse {
    pub readers: Vec<serde_json::Value>,  // 已读用户列表
    pub total: usize,
}

/// 获取消息已读统计响应
/// 
/// RPC路由: `message/status/read_stats`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadStatsResponse {
    pub read_count: u32,
    pub total_count: u32,
}

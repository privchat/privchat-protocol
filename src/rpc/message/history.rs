/// 消息历史相关 RPC

use serde::{Deserialize, Serialize};

/// 获取消息历史请求
/// 
/// RPC路由: `message/history/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryGetRequest {
    /// 用户ID
    pub user_id: u64,
    /// 频道ID
    pub channel_id: u64,
    /// 起始服务端消息ID（可选，用于分页）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_server_message_id: Option<u64>,
    /// 限制数量（可选，默认50）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// 消息历史响应
/// 
/// RPC路由: `message/history/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryResponse {
    pub messages: Vec<serde_json::Value>,
    pub has_more: bool,
}

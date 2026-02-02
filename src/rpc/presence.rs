use serde::{Deserialize, Serialize};
use crate::presence::*;

/// RPC: presence/subscribe
/// 订阅在线状态（打开私聊会话时调用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePresenceRequest {
    /// 要订阅的用户ID列表
    pub user_ids: Vec<u64>,
}

/// 订阅在线状态响应
/// 
/// RPC路由: `presence/subscribe`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePresenceResponse {
    /// 响应码（0 成功）
    pub code: i32,
    /// 响应消息
    pub message: String,
    /// 所有用户的初始在线状态
    pub initial_statuses: std::collections::HashMap<u64, OnlineStatusInfo>,
}

/// RPC: presence/unsubscribe
/// 取消订阅（关闭私聊会话时调用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribePresenceRequest {
    /// 要取消订阅的用户ID列表
    pub user_ids: Vec<u64>,
}

/// 取消订阅在线状态响应
/// 
/// RPC路由: `presence/unsubscribe`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribePresenceResponse {
    /// 响应码（0 成功）
    pub code: i32,
    /// 响应消息
    pub message: String,
}

/// RPC: presence/typing
/// 发送输入状态通知
pub use crate::presence::{TypingIndicatorRequest, TypingIndicatorResponse};

/// 黑名单相关 RPC
use serde::{Deserialize, Serialize};

/// 添加黑名单请求
///
/// RPC路由: `contact/blacklist/add`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistAddRequest {
    /// 当前用户ID
    pub user_id: u64,
    /// 被拉黑的用户ID
    pub blocked_user_id: u64,
}

/// 移除黑名单请求
///
/// RPC路由: `contact/blacklist/remove`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistRemoveRequest {
    /// 当前用户ID
    pub user_id: u64,
    /// 要移除的用户ID
    pub blocked_user_id: u64,
}

/// 检查黑名单请求
///
/// RPC路由: `contact/blacklist/check`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistCheckRequest {
    /// 当前用户ID
    pub user_id: u64,
    /// 要检查的用户ID
    pub target_user_id: u64,
}

/// 获取黑名单列表请求
///
/// RPC路由: `contact/blacklist/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistListRequest {
    /// 当前用户ID
    pub user_id: u64,
}

/// 添加黑名单响应
///
/// RPC路由: `contact/blacklist/add`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type BlacklistAddResponse = bool;

/// 移除黑名单响应
///
/// RPC路由: `contact/blacklist/remove`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type BlacklistRemoveResponse = bool;

/// 检查黑名单状态响应
///
/// RPC路由: `contact/blacklist/check`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistCheckResponse {
    pub is_blocked: bool,
}

/// 获取黑名单列表响应
///
/// RPC路由: `contact/blacklist/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistListResponse {
    pub users: Vec<BlacklistUserInfo>,
    pub total: usize,
}

/// 黑名单用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistUserInfo {
    pub user_id: u64,
    pub username: String,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub blocked_at: i64, // 毫秒时间戳
}

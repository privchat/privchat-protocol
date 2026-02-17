/// 账号搜索相关 RPC
use serde::{Deserialize, Serialize};

/// 搜索用户请求
///
/// RPC路由: `account/search/query`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSearchQueryRequest {
    /// 搜索关键词（用户名、手机号等）
    pub query: String,
    /// 页码（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// 每页数量（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,

    /// 搜索发起者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub from_user_id: u64,
}

/// 通过二维码搜索用户请求
///
/// RPC路由: `account/search/by_qrcode`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSearchByQRCodeRequest {
    /// 二维码Key
    pub qr_key: String,
    /// Token（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// 搜索发起者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub searcher_id: u64,
}

/// 搜索到的用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchedUser {
    /// 用户ID
    pub user_id: u64,
    /// 用户名
    pub username: String,
    /// 昵称
    pub nickname: String,
    /// 头像URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// 用户类型（0: 普通用户, 1: 系统用户, 2: 机器人）
    pub user_type: i16,
    /// 搜索会话ID（用于后续操作）
    pub search_session_id: u64,
    /// 是否已是好友
    pub is_friend: bool,
    /// 是否可以发送消息
    pub can_send_message: bool,
}

/// 搜索响应（返回用户信息列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSearchResponse {
    /// 搜索到的用户列表
    pub users: Vec<SearchedUser>,
    /// 总数
    pub total: usize,
    /// 搜索关键词
    pub query: String,
}

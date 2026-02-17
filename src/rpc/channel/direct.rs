/// 私聊会话相关 RPC（获取或创建）
use serde::{Deserialize, Serialize};

/// 获取或创建私聊会话请求
///
/// RPC 路由: `channel/direct/get_or_create`
/// 与添加好友的 source/source_id 规范一致，用于安全与追溯。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrCreateDirectChannelRequest {
    /// 对方用户 ID
    pub target_user_id: u64,
    /// 来源类型：search / phone / card_share(好友分享) / group(群聊) / qrcode 等
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 来源 ID：如搜索会话 id、群 id、分享 id、好友 id 等
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,

    /// 当前用户 ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取或创建私聊会话响应
///
/// RPC 路由: `channel/direct/get_or_create`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrCreateDirectChannelResponse {
    /// 会话 ID，用于发消息等
    pub channel_id: u64,
    /// 是否本次新创建的会话（false 表示已存在）
    pub created: bool,
}

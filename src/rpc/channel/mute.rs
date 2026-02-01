/// 频道静音 RPC

use serde::{Deserialize, Serialize};

/// 设置频道静音请求
/// 
/// RPC路由: `channel/mute`
/// 
/// 设置频道静音后，该频道的新消息将不会推送通知。
/// 这是用户个人的偏好设置，适用于私聊和群聊。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMuteRequest {
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
    /// 频道ID
    pub channel_id: u64,
    /// 是否静音
    pub muted: bool,
}

/// 设置频道静音响应
/// 
/// RPC路由: `channel/mute`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type ChannelMuteResponse = bool;

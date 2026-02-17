/// 频道置顶 RPC
use serde::{Deserialize, Serialize};

/// 置顶频道请求
///
/// RPC路由: `channel/pin`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPinRequest {
    /// 用户ID
    pub user_id: u64,
    /// 频道ID
    pub channel_id: u64,
    /// 是否置顶
    pub pinned: bool,
}

/// 置顶频道响应
///
/// RPC路由: `channel/pin`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type ChannelPinResponse = bool;

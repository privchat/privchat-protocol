/// 广播频道相关 RPC（用于订阅号/频道功能）

use serde::{Deserialize, Serialize};

/// 订阅广播频道请求
/// 
/// RPC路由: `channel/broadcast/subscribe`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBroadcastSubscribeRequest {
    /// 用户ID（服务器端从 ctx 填充）
    #[serde(default)]
    pub user_id: u64,
    
    /// 广播频道ID
    pub channel_id: u64,
}

/// 订阅广播频道响应
/// 
/// RPC路由: `channel/broadcast/subscribe`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type ChannelBroadcastSubscribeResponse = bool;

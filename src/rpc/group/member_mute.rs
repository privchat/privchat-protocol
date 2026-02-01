/// 禁言/解除禁言群组成员 RPC

use serde::{Deserialize, Serialize};

/// 禁言群组成员请求
/// 
/// RPC路由: `group/member/mute`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberMuteRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID
    pub operator_id: u64,
    /// 被禁言的用户ID
    pub user_id: u64,
    /// 禁言时长（秒）
    pub mute_duration: u64,
}

/// 解除禁言请求
/// 
/// RPC路由: `group/member/unmute`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberUnmuteRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID
    pub operator_id: u64,
    /// 被解除禁言的用户ID
    pub user_id: u64,
}

/// 禁言操作响应
/// 
/// RPC路由: `group/member/mute`
/// 返回禁言到期时间戳（毫秒），0 表示永久禁言
pub type GroupMemberMuteResponse = u64;

/// 解除禁言响应
/// 
/// RPC路由: `group/member/unmute`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type GroupMemberUnmuteResponse = bool;

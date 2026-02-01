/// 群组成员管理相关 RPC

use serde::{Deserialize, Serialize};

/// 获取群组成员列表请求
/// 
/// RPC路由: `group/member/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberListRequest {
    /// 群组ID
    pub group_id: u64,
    
    /// 请求者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 移除群组成员请求
/// 
/// RPC路由: `group/member/remove`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberRemoveRequest {
    /// 群组ID
    pub group_id: u64,
    /// 被移除的用户ID
    pub user_id: u64,
    
    /// 操作者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub operator_id: u64,
}

/// 离开群组请求
/// 
/// RPC路由: `group/member/leave`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberLeaveRequest {
    /// 群组ID
    pub group_id: u64,
    
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 添加群组成员请求
/// 
/// RPC路由: `group/member/add`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberAddRequest {
    /// 群组ID
    pub group_id: u64,
    /// 要添加的用户ID
    pub user_id: u64,
    /// 成员角色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    
    /// 邀请者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub inviter_id: u64,
}

/// 禁言群组成员请求
/// 
/// RPC路由: `group/member/mute`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberMuteRequest {
    /// 群组ID
    pub group_id: u64,
    /// 被禁言的用户ID
    pub user_id: u64,
    /// 禁言时长（秒），0表示永久禁言
    pub mute_duration: u64,
    
    /// 操作者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub operator_id: u64,
}

/// 取消禁言群组成员请求
/// 
/// RPC路由: `group/member/unmute`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberUnmuteRequest {
    /// 群组ID
    pub group_id: u64,
    /// 被取消禁言的用户ID
    pub user_id: u64,
    
    /// 操作者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub operator_id: u64,
}

/// 获取群组成员列表响应
/// 
/// RPC路由: `group/member/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberListResponse {
    pub members: Vec<serde_json::Value>,  // 成员信息列表
    pub total: usize,
}

/// 添加群组成员响应
/// 
/// RPC路由: `group/member/add`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type GroupMemberAddResponse = bool;

/// 移除群组成员响应
/// 
/// RPC路由: `group/member/remove`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type GroupMemberRemoveResponse = bool;

/// 离开群组响应
/// 
/// RPC路由: `group/member/leave`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type GroupMemberLeaveResponse = bool;

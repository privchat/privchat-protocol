/// 群组设置相关 RPC

use serde::{Deserialize, Serialize};

/// 更新群组设置请求
/// 
/// RPC路由: `group/settings/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsUpdateRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID
    pub operator_id: u64,
    /// 群组名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 群组描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 群组头像URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// 获取群组设置请求
/// 
/// RPC路由: `group/settings/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsGetRequest {
    /// 群组ID
    pub group_id: u64,
    /// 用户ID
    pub user_id: u64,
}

/// 全员禁言请求
/// 
/// RPC路由: `group/settings/mute_all`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMuteAllRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID
    pub operator_id: u64,
    /// 是否全员禁言
    pub mute_all: bool,
}

/// 更新群组设置响应
/// 
/// RPC路由: `group/settings/update`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type GroupSettingsUpdateResponse = bool;

/// 获取群组设置响应
/// 
/// RPC路由: `group/settings/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsGetResponse {
    pub settings: serde_json::Value,  // 或者定义具体的 Settings 结构
}

/// 设置群组成员角色 RPC
/// 
/// RPC路由: `group/role/set`

use serde::{Deserialize, Serialize};

/// 设置群组成员角色请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRoleSetRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID（必须是群主）
    pub operator_id: u64,
    /// 目标成员ID
    pub user_id: u64,
    /// 目标角色: "admin" | "member"
    pub role: String,
}

/// 设置群组成员角色响应
/// 
/// RPC路由: `group/role/set`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRoleSetResponse {
    pub group_id: u64,
    pub user_id: u64,
    pub role: String,
    pub updated_at: Option<String>,  // ISO 8601，可选
}

/// 群组基本操作 RPC
use serde::{Deserialize, Serialize};

/// 创建群组请求
///
/// RPC路由: `group/group/create`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCreateRequest {
    /// 群组名称
    pub name: String,
    /// 群组描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 初始成员ID列表（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_ids: Option<Vec<u64>>,

    /// 创建者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub creator_id: u64,
}

/// 获取群组信息请求
///
/// RPC路由: `group/group/info`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfoRequest {
    /// 群组ID
    pub group_id: u64,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 创建群组响应
///
/// RPC路由: `group/group/create`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCreateResponse {
    pub group_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub member_count: u32,
    pub created_at: String, // ISO 8601
    pub creator_id: u64,
}

/// 获取群组信息响应
///
/// RPC路由: `group/group/info`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfoResponse {
    pub group_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub owner_id: u64,
    pub created_at: String,
    pub updated_at: String,
    pub member_count: u32,
    pub message_count: Option<u32>,
    pub is_archived: Option<bool>,
    pub tags: Option<serde_json::Value>,
    pub custom_fields: Option<serde_json::Value>,
}

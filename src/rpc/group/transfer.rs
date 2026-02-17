/// 群主转让 RPC
use serde::{Deserialize, Serialize};

/// 转让群主请求
///
/// RPC路由: `group/role/transfer_owner`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTransferOwnerRequest {
    /// 群组ID
    pub group_id: u64,
    /// 当前群主ID
    pub current_owner_id: u64,
    /// 新群主ID
    pub new_owner_id: u64,
}

/// 转让群主响应
///
/// RPC路由: `group/role/transfer_owner`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTransferOwnerResponse {
    pub group_id: u64,
    pub new_owner_id: u64,
    pub transferred_at: Option<String>, // ISO 8601，可选
}

/// 群组审批相关 RPC
use serde::{Deserialize, Serialize};

/// 获取群组审批列表请求
///
/// RPC路由: `group/approval/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalListRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID
    pub operator_id: u64,
}

/// 处理群组审批请求
///
/// RPC路由: `group/approval/handle`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalHandleRequest {
    /// 请求ID
    pub request_id: String,
    /// 操作者ID
    pub operator_id: u64,
    /// 操作（approve/reject）
    pub action: String,
    /// 拒绝原因（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<String>,
}

/// 获取群组审批列表响应
///
/// RPC路由: `group/approval/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalMethodMemberInvite {
    pub inviter_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalMethodQrCode {
    pub qr_code_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalMethod {
    #[serde(rename = "MemberInvite", skip_serializing_if = "Option::is_none")]
    pub member_invite: Option<GroupApprovalMethodMemberInvite>,
    #[serde(rename = "QRCode", skip_serializing_if = "Option::is_none")]
    pub qr_code: Option<GroupApprovalMethodQrCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalItem {
    pub request_id: String,
    pub user_id: u64,
    pub method: GroupApprovalMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalListResponse {
    pub group_id: String,
    #[serde(default, alias = "approvals")]
    pub requests: Vec<GroupApprovalItem>,
    pub total: usize,
}

/// 处理群组审批响应
///
/// RPC路由: `group/approval/handle`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalHandleResponse {
    pub success: bool,
    pub request_id: String,
    pub action: String,
    pub group_id: u64,
    pub user_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<String>,
    pub message: String,
    pub handled_at: String,
}

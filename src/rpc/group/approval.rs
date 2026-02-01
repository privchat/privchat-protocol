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
    /// 审批ID
    pub approval_id: u64,
    /// 操作者ID
    pub operator_id: u64,
    /// 是否批准
    pub approved: bool,
    /// 拒绝原因（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<String>,
}

/// 获取群组审批列表响应
/// 
/// RPC路由: `group/approval/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupApprovalListResponse {
    pub approvals: Vec<serde_json::Value>,
    pub total: usize,
}

/// 处理群组审批响应
/// 
/// RPC路由: `group/approval/handle`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type GroupApprovalHandleResponse = bool;

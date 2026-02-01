use serde::{Deserialize, Serialize};

/// RPC: device/push/update
/// 更新设备推送状态（前台/后台切换时调用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePushUpdateRequest {
    /// 设备ID
    pub device_id: String,
    /// 是否需要推送（true: 需要推送, false: 不需要推送）
    pub apns_armed: bool,
    /// 可选的推送令牌（如果提供则更新）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_token: Option<String>,
}

/// 设备推送状态更新响应
/// 
/// RPC路由: `device/push/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePushUpdateResponse {
    /// 设备ID
    pub device_id: String,
    /// 推送状态
    pub apns_armed: bool,
    /// 用户级别：所有设备都需要推送
    pub user_push_enabled: bool,
}

/// RPC: device/push/status
/// 获取设备推送状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePushStatusRequest {
    /// 可选的设备ID（不提供则返回所有设备）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

/// 设备推送状态查询响应
/// 
/// RPC路由: `device/push/status`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePushStatusResponse {
    /// 设备列表
    pub devices: Vec<DevicePushInfo>,
    /// 用户级别：所有设备都需要推送
    pub user_push_enabled: bool,
}

/// 设备推送信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePushInfo {
    /// 设备ID
    pub device_id: String,
    /// 是否需要推送
    pub apns_armed: bool,
    /// 是否已连接
    pub connected: bool,
    /// 平台
    pub platform: String,
    /// 推送厂商
    pub vendor: String,
}

/// 二维码相关 RPC

use serde::{Deserialize, Serialize};

/// 生成二维码请求
/// 
/// RPC路由: `qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeGenerateRequest {
    /// 二维码类型（user, group, auth, feature）
    pub qr_type: String,
    
    /// 目标ID
    pub target_id: String,
    
    /// 过期时间（秒）（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_seconds: Option<u64>,
    
    /// 最大使用次数（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_usage: Option<u32>,
    
    /// 额外元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 解析二维码请求
/// 
/// RPC路由: `qrcode/resolve`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeResolveRequest {
    /// QR Key
    pub qr_key: String,
    
    /// Token（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    
    /// 扫描者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub scanner_id: u64,
}

/// 刷新二维码请求
/// 
/// RPC路由: `qrcode/refresh`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeRefreshRequest {
    /// 旧的 QR Key
    pub old_qr_key: String,
    
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 撤销二维码请求
/// 
/// RPC路由: `qrcode/revoke`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeRevokeRequest {
    /// QR Key
    pub qr_key: String,
    
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取二维码列表请求
/// 
/// RPC路由: `qrcode/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeListRequest {
    /// 二维码类型过滤（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr_type: Option<String>,
    
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 生成用户二维码请求
/// 
/// RPC路由: `user/qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeGenerateRequest {
    /// 过期时间（秒）（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_seconds: Option<u64>,
    
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 刷新用户二维码请求
/// 
/// RPC路由: `user/qrcode/refresh`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeRefreshRequest {
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取用户二维码请求
/// 
/// RPC路由: `user/qrcode/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeGetRequest {
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 生成二维码响应
/// 
/// RPC路由: `qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeGenerateResponse {
    pub qr_key: String,
    pub qr_code: String,  // privchat://...
    pub qr_type: String,
    pub target_id: u64,
    pub created_at: String,  // ISO 8601
    pub expire_at: Option<String>,  // ISO 8601
    pub max_usage: Option<u32>,
    pub used_count: u32,
}

/// 解析二维码响应
/// 
/// RPC路由: `qrcode/resolve`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeResolveResponse {
    pub qr_type: String,
    pub target_id: u64,
    pub action: String,
    pub data: Option<serde_json::Value>,
    pub used_count: u32,
    pub max_usage: Option<u32>,
    pub expire_at: Option<String>,  // ISO 8601
}

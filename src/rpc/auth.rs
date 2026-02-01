/// 认证相关 RPC

use serde::{Deserialize, Serialize};
use crate::message::DeviceInfo;

/// 登录请求
/// 
/// RPC路由: `account/auth/login`
/// 
/// 安全要求：
/// - device_id: **必须提供**，用于设备绑定和防止 token 复制
/// - device_info: 设备详细信息，用于设备管理和安全审计
/// 
/// JWT Token 会绑定 device_id，验证时会检查：
/// - token 中的 device_id 必须匹配请求中的 device_id
/// - 切换设备必须重新登录
/// - 防止 token 被复制到其他设备使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthLoginRequest {
    /// 用户名或手机号或邮箱
    pub username: String,
    
    /// 密码
    pub password: String,
    
    /// 设备ID（**必需**，用于设备绑定和 token 验证）
    /// 
    /// 安全说明：
    /// - 每个设备应有唯一的 device_id（建议使用 UUID）
    /// - JWT token 会绑定此 device_id
    /// - 切换设备必须重新登录
    /// - 防止账号认证信息被复刻到其他设备
    pub device_id: String,
    
    /// 设备信息（可选，用于设备管理和安全审计）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_info: Option<DeviceInfo>,
}

/// 用户注册请求
/// 
/// RPC路由: `account/user/register`
/// 
/// 安全要求：
/// - device_id: **必须提供**，注册时自动登录并绑定设备
/// - device_info: 设备详细信息，用于设备管理和安全审计
/// 
/// 注册成功后自动在当前设备登录，返回绑定此设备的 JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisterRequest {
    /// 用户名
    pub username: String,
    
    /// 密码
    pub password: String,
    
    /// 设备ID（**必需**，注册后自动登录并绑定此设备）
    /// 
    /// 安全说明：
    /// - 注册成功后自动在此设备登录
    /// - JWT token 会绑定此 device_id
    /// - 其他设备需要单独登录
    /// - 防止注册后 token 被复制到其他设备
    pub device_id: String,
    
    /// 昵称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    
    /// 手机号（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    
    /// 邮箱（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    
    /// 设备信息（可选，用于设备管理和安全审计）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_info: Option<DeviceInfo>,
}

/// 认证响应（登录和注册统一返回结构）
/// 
/// RPC路由: `account/auth/login`, `account/user/register`
/// 
/// 注册成功或登录成功后都返回此结构，包含绑定设备的 JWT token
/// 客户端可以直接使用 token 进行 AuthorizationRequest
/// 
/// JWT Token 安全说明：
/// - token 包含 user_id 和 device_id
/// - 验证时会检查请求的 device_id 是否匹配 token 中的 device_id
/// - 切换设备必须重新登录获取新 token
/// - 防止 token 被复制到其他设备使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// 用户ID
    pub user_id: u64,
    
    /// JWT Token（用于 AuthorizationRequest，绑定设备）
    /// 
    /// Token payload 包含：
    /// - user_id: 用户ID
    /// - device_id: 设备ID（用于验证）
    /// - exp: 过期时间
    /// - iat: 签发时间
    pub token: String,
    
    /// Refresh Token（用于刷新 token，同样绑定设备）
    pub refresh_token: Option<String>,
    
    /// Token 过期时间（RFC3339 格式）
    pub expires_at: String,
    
    /// 绑定的设备ID（返回给客户端确认）
    /// 
    /// 客户端应保存此 device_id，后续所有请求都需要携带
    pub device_id: String,
}

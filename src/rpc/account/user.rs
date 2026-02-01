/// 用户信息相关 RPC

use serde::{Deserialize, Serialize};

/// 获取用户详情请求
/// 
/// RPC路由: `account/user/detail`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserDetailRequest {
    /// 目标用户ID
    pub target_user_id: u64,
    /// 来源
    pub source: String,
    /// 来源ID
    pub source_id: String,
    
    /// 当前用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 生成用户分享卡片请求
/// 
/// RPC路由: `account/user/share_card`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserShareCardRequest {
    /// 要分享的目标用户ID
    pub target_user_id: u64,
    /// 接收者ID
    pub receiver_id: u64,
    /// 过期时间（秒）（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_seconds: Option<u64>,
    
    /// 分享者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub sharer_id: u64,
}

/// 更新用户信息请求
/// 
/// RPC路由: `account/user/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserUpdateRequest {
    /// 显示名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// 头像URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// 个人简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取用户详情响应
/// 
/// RPC路由: `account/user/detail`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserDetailResponse {
    pub user: serde_json::Value,  // 或者定义具体的 UserInfo 结构
}

/// 更新用户信息响应
/// 
/// RPC路由: `account/user/update`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type AccountUserUpdateResponse = bool;

/// 生成用户分享卡片响应
/// 
/// RPC路由: `account/user/share_card`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserShareCardResponse {
    pub share_key: String,
    pub share_url: String,
    pub expire_at: Option<String>,  // ISO 8601
}

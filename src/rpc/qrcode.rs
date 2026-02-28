// Copyright 2025 Shanghai Boyu Information Technology Co., Ltd.
// https://privchat.dev
//
// Author: zoujiaqing <zoujiaqing@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// 二维码相关 RPC
use serde::{Deserialize, Serialize};

/// 生成二维码请求
///
/// RPC路由: `qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeGenerateRequest {
    /// 二维码类型（user, group, auth, feature）
    pub qr_type: String,
    /// 目标 ID（用户ID/群组ID/会话ID）
    pub target_id: String,
    /// 过期时间（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_seconds: Option<u64>,
    /// 最大使用次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_usage: Option<u32>,
    /// 额外元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// 用户 ID（服务器端填充）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 解析二维码请求
///
/// RPC路由: `qrcode/resolve`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeResolveRequest {
    pub qr_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// 扫描者 ID（服务器端填充）
    #[serde(skip_deserializing, default)]
    pub scanner_id: u64,
}

/// 刷新二维码请求
///
/// RPC路由: `qrcode/refresh`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeRefreshRequest {
    pub qr_type: String,
    pub target_id: String,
    pub creator_id: String,
}

/// 撤销二维码请求
///
/// RPC路由: `qrcode/revoke`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeRevokeRequest {
    pub qr_key: String,
    /// 用户 ID（服务器端填充）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取二维码列表请求
///
/// RPC路由: `qrcode/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeListRequest {
    pub creator_id: String,
    #[serde(default)]
    pub include_revoked: bool,
}

/// 生成用户二维码请求
///
/// RPC路由: `user/qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeGenerateRequest {
    /// 过期时间（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_seconds: Option<u64>,
    /// 用户ID（服务器端填充）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 刷新用户二维码请求
///
/// RPC路由: `user/qrcode/refresh`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeRefreshRequest {
    pub user_id: String,
}

/// 获取用户二维码请求
///
/// RPC路由: `user/qrcode/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeGetRequest {
    pub user_id: String,
}

/// 二维码条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeEntry {
    pub qr_key: String,
    pub qr_code: String,
    pub qr_type: String,
    pub target_id: String,
    pub created_at: String,
    pub expire_at: Option<String>,
    pub used_count: u32,
    pub max_usage: Option<u32>,
    #[serde(default)]
    pub revoked: bool,
}

/// 生成二维码响应
///
/// RPC路由: `qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeGenerateResponse {
    pub qr_key: String,
    pub qr_code: String,
    pub qr_type: String,
    pub target_id: u64,
    pub created_at: String,
    pub expire_at: Option<String>,
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
    pub expire_at: Option<String>,
}

/// 刷新二维码响应
///
/// RPC路由: `qrcode/refresh`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeRefreshResponse {
    pub old_qr_key: String,
    pub new_qr_key: String,
    pub new_qr_code: String,
    pub revoked_at: String,
}

/// 撤销二维码响应
///
/// RPC路由: `qrcode/revoke`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeRevokeResponse {
    pub success: bool,
    pub qr_key: String,
    pub revoked_at: String,
}

/// 获取二维码列表响应
///
/// RPC路由: `qrcode/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeListResponse {
    pub qr_keys: Vec<QRCodeEntry>,
    pub total: usize,
}

/// 生成用户二维码响应
///
/// RPC路由: `user/qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeGenerateResponse {
    pub qr_key: String,
    pub qr_code: String,
    pub created_at: String,
}

/// 刷新用户二维码响应
///
/// RPC路由: `user/qrcode/refresh`
pub type UserQRCodeRefreshResponse = QRCodeRefreshResponse;

/// 获取用户二维码响应
///
/// RPC路由: `user/qrcode/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeGetResponse {
    pub qr_key: String,
    pub qr_code: String,
    pub created_at: String,
    pub used_count: u32,
}

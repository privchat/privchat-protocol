// Copyright 2024 Shanghai Boyu Information Technology Co., Ltd.
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

use crate::protocol::DeviceInfo;
/// 认证相关 RPC
use serde::{Deserialize, Serialize};

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

    /// Token 过期时间（Unix 毫秒时间戳）
    pub expires_at: u64,

    /// 绑定的设备ID（返回给客户端确认）
    ///
    /// 客户端应保存此 device_id，后续所有请求都需要携带
    pub device_id: String,
}

/// 刷新 Token 请求（Phase B1，non-rotation）
///
/// RPC 路由: `account/auth/refresh`
///
/// 语义：
/// - 调用方携带 refresh_token + device_id，服务端签发新的 access token 并原样返回 refresh_token
/// - Phase B1 不做 rotation / grace / reused；Phase B2 将引入
///
/// 错误码：
/// - `10001 InvalidToken`   —— 签名 / 格式 / typ 错误，不可自愈
/// - `10009 RefreshTokenExpired` —— refresh token 已过期
/// - `10010 RefreshTokenRevoked` —— 设备不匹配 / session_version 落后 / 服务端撤销
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRefreshRequest {
    /// 登录或上一次 refresh 时拿到的 refresh token
    pub refresh_token: String,

    /// 当前设备 ID，必须与 refresh_token 绑定的 device_id 一致
    pub device_id: String,
}

/// 刷新 Token 响应（Phase B1）
///
/// - `access_token` 为新签发的 access token
/// - `refresh_token` B1 下原样返回；B2 启用 rotation 后才会下发新值。客户端处理规则：
///     - 字段存在且非空 → 替换本地 refresh token
///     - 字段缺失 / None → 保留本地现有 refresh token（**不要当作清空**）
/// - `expires_at` 为 access token 过期时刻（Unix ms）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRefreshResponse {
    /// 新的 access token
    pub access_token: String,

    /// 新的 refresh token；B1 下可能与入参相同，客户端缺失时应保留旧值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// access token 过期时间（Unix 毫秒）
    pub expires_at: u64,
}

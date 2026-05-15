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

//! 个人名片二维码相关 RPC（QR_CODE_SPEC v1.3）。
//!
//! 字段语义：
//! - `qr_key` 是 `privchat_users.qr_key` 字段（UNIQUE NOT NULL, 16-char base62, 永久）
//! - URL 形态 `https://<host>/privchat:protocol/user/get?qrkey=<qr_key>`
//!
//! 与遗留的通用 `qrcode/*` 模块（[`super::qrcode`]）解耦：
//! 通用模块的 `QRCodeService` (DashMap) 只服务于 Auth / Feature 类型，
//! User / Group 走主表字段。

use serde::{Deserialize, Serialize};

// ---------- user/qrcode/get ----------

/// 读取当前用户名片二维码请求
///
/// RPC 路由: `user/qrcode/get`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserQRCodeGetRequest {
    /// 用户 ID（服务器端从 ctx 填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 读取当前用户名片二维码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeGetResponse {
    /// opaque token（来自 `privchat_users.qr_key`）
    pub qr_key: String,
    /// 已经拼好的完整 URL：
    /// `https://<qr_base_url>/privchat:protocol/user/get?qrkey=<qr_key>`
    pub qr_code: String,
    /// 用户 ID（回显）
    pub user_id: u64,
}

// ---------- user/qrcode/refresh ----------

/// 旋转当前用户名片二维码请求
///
/// RPC 路由: `user/qrcode/refresh`
///
/// 执行 `UPDATE privchat_users SET qr_key=$new WHERE user_id=$self`，
/// 老 qr_key 立即作废。用户主动调用以应对骚扰。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserQRCodeRefreshRequest {
    /// 用户 ID（服务器端从 ctx 填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 旋转当前用户名片二维码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeRefreshResponse {
    /// 旧 qr_key（已作废）
    pub old_qr_key: String,
    /// 新 qr_key
    pub new_qr_key: String,
    /// 已经拼好的新 URL
    pub qr_code: String,
    /// 用户 ID
    pub user_id: u64,
}

// ---------- user/qrcode/resolve ----------

/// 解析对端名片二维码请求
///
/// RPC 路由: `user/qrcode/resolve`
///
/// 任何已登录用户都可调用；服务端 `SELECT user_id FROM privchat_users WHERE qr_key = $1`
/// 反查，返回最小用户卡片。**不返回 qr_key**（避免二维码二次扩散）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeResolveRequest {
    /// 客户端从扫到的 URL 里 percent-decode 之后的 qrkey
    pub qr_key: String,
    /// 操作者 ID（服务器端从 ctx 填充）
    #[serde(skip_deserializing, default)]
    pub operator_id: u64,
}

/// 解析对端名片二维码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQRCodeResolveResponse {
    /// 目标用户 ID
    pub user_id: u64,
    /// 用户名（账号系统唯一标识）
    pub username: String,
    /// 显示名（昵称）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// 头像 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// 用户类型：0=普通用户 / 1=系统用户 / 2=机器人
    pub user_type: i16,
    /// 调用者与目标用户是否已经是好友
    pub is_friend: bool,
    /// 调用者是不是目标用户本身（扫了自己的码）
    pub is_self: bool,
}

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

/// 黑名单相关 RPC
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 添加黑名单请求
///
/// RPC路由: `contact/blacklist/add`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistAddRequest {
    /// 当前用户ID
    pub user_id: u64,
    /// 被拉黑的用户ID
    pub blocked_user_id: u64,
}

/// 移除黑名单请求
///
/// RPC路由: `contact/blacklist/remove`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistRemoveRequest {
    /// 当前用户ID
    pub user_id: u64,
    /// 要移除的用户ID
    pub blocked_user_id: u64,
}

/// 检查黑名单请求
///
/// RPC路由: `contact/blacklist/check`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistCheckRequest {
    /// 当前用户ID
    pub user_id: u64,
    /// 要检查的用户ID
    pub target_user_id: u64,
}

/// 获取黑名单列表请求
///
/// RPC路由: `contact/blacklist/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistListRequest {
    /// 当前用户ID
    pub user_id: u64,
}

/// 添加黑名单响应
///
/// RPC路由: `contact/blacklist/add`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type BlacklistAddResponse = bool;

/// 移除黑名单响应
///
/// RPC路由: `contact/blacklist/remove`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type BlacklistRemoveResponse = bool;

/// 检查黑名单状态响应
///
/// RPC路由: `contact/blacklist/check`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistCheckResponse {
    pub success: bool,
    pub blocked: bool,
}

/// 获取黑名单列表响应
///
/// RPC路由: `contact/blacklist/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistListResponse {
    pub success: bool,
    pub users: Vec<BlacklistUserInfo>,
}

/// 黑名单用户信息（join 行：拉黑者 + 被拉黑者 + 时间 + 原因）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistUserInfo {
    /// 拉黑者 ID
    pub user_id: u64,
    /// 被拉黑用户 ID
    pub blocked_user_id: u64,
    /// 拉黑时间（ISO8601 / RFC3339；服务端用 chrono `DateTime<Utc>` 序列化）
    pub blocked_at: DateTime<Utc>,
    /// 拉黑原因（可选）
    pub reason: Option<String>,
}

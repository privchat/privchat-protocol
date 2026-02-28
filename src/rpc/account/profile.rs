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

/// 账号资料相关 RPC
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 获取个人资料请求
///
/// RPC路由: `account/profile/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfileGetRequest {
    /// 用户ID（服务器端填充，客户端不需要传）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 更新个人资料请求
///
/// RPC路由: `account/profile/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfileUpdateRequest {
    /// 显示名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// 头像 URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// 个人简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// 兼容扩展字段（可选）
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_fields: HashMap<String, String>,
    /// 用户ID（服务器端填充，客户端不需要传）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取个人资料响应
///
/// RPC路由: `account/profile/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfileGetResponse {
    pub status: String,
    pub action: String,
    pub timestamp: String,
}

/// 更新个人资料响应
///
/// RPC路由: `account/profile/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfileUpdateResponse {
    pub status: String,
    pub action: String,
    pub timestamp: String,
}

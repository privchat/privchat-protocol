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
    pub user_id: u64,
    pub username: String,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub user_type: i16,
    pub is_friend: bool,
    pub can_send_message: bool,
    pub source_type: String,
    pub source_id: String,
}

/// 更新用户信息响应
///
/// RPC路由: `account/user/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserUpdateResponse {
    pub status: String,
    pub action: String,
    pub timestamp: String,
}

/// 生成用户分享卡片响应
///
/// RPC路由: `account/user/share_card`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserShareCardResponse {
    pub share_id: String,
    pub target_user_id: u64,
    pub receiver_id: u64,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_at: Option<String>,
}

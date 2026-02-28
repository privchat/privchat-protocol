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

/// 隐私设置相关 RPC
use serde::{Deserialize, Serialize};

/// 获取隐私设置请求
///
/// RPC路由: `account/privacy/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPrivacyGetRequest {
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 更新隐私设置请求
///
/// RPC路由: `account/privacy/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPrivacyUpdateRequest {
    /// 是否允许通过群组添加（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_add_by_group: Option<bool>,
    /// 是否允许通过手机号搜索（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_search_by_phone: Option<bool>,
    /// 是否允许通过用户名搜索（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_search_by_username: Option<bool>,
    /// 是否允许通过邮箱搜索（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_search_by_email: Option<bool>,
    /// 是否允许通过二维码搜索（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_search_by_qrcode: Option<bool>,
    /// 是否允许非好友查看资料（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_view_by_non_friend: Option<bool>,
    /// 是否允许接收非好友消息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_receive_message_from_non_friend: Option<bool>,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取隐私设置响应
///
/// RPC路由: `account/privacy/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPrivacySettings {
    pub user_id: u64,
    pub allow_add_by_group: bool,
    pub allow_search_by_phone: bool,
    pub allow_search_by_username: bool,
    pub allow_search_by_email: bool,
    pub allow_search_by_qrcode: bool,
    pub allow_view_by_non_friend: bool,
    pub allow_receive_message_from_non_friend: bool,
    pub updated_at: String,
}

/// 获取隐私设置响应
///
/// RPC路由: `account/privacy/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPrivacyGetResponse {
    pub user_id: u64,
    pub allow_add_by_group: bool,
    pub allow_search_by_phone: bool,
    pub allow_search_by_username: bool,
    pub allow_search_by_email: bool,
    pub allow_search_by_qrcode: bool,
    pub allow_view_by_non_friend: bool,
    pub allow_receive_message_from_non_friend: bool,
    pub updated_at: String,
}

/// 更新隐私设置响应
///
/// RPC路由: `account/privacy/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPrivacyUpdateResponse {
    pub success: bool,
    pub user_id: u64,
    pub message: String,
    pub allow_add_by_group: bool,
    pub allow_search_by_phone: bool,
    pub allow_search_by_username: bool,
    pub allow_search_by_email: bool,
    pub allow_search_by_qrcode: bool,
    pub allow_view_by_non_friend: bool,
    pub allow_receive_message_from_non_friend: bool,
    pub updated_at: String,
}

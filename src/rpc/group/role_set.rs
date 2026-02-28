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

/// 设置群组成员角色 RPC
///
/// RPC路由: `group/role/set`
use serde::{Deserialize, Serialize};

/// 设置群组成员角色请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRoleSetRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID（必须是群主）
    pub operator_id: u64,
    /// 目标成员ID
    pub user_id: u64,
    /// 目标角色: "admin" | "member"
    pub role: String,
}

/// 设置群组成员角色响应
///
/// RPC路由: `group/role/set`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRoleSetResponse {
    pub group_id: u64,
    pub user_id: u64,
    pub role: String,
    pub updated_at: Option<String>, // ISO 8601，可选
}

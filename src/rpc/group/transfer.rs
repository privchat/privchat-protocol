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

/// 群主转让 RPC
use serde::{Deserialize, Serialize};

/// 转让群主请求
///
/// RPC路由: `group/role/transfer_owner`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTransferOwnerRequest {
    /// 群组ID
    pub group_id: u64,
    /// 当前群主ID
    pub current_owner_id: u64,
    /// 新群主ID
    pub new_owner_id: u64,
}

/// 转让群主响应
///
/// RPC路由: `group/role/transfer_owner`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTransferOwnerResponse {
    pub group_id: u64,
    pub new_owner_id: u64,
    pub transferred_at: Option<String>, // ISO 8601，可选
}

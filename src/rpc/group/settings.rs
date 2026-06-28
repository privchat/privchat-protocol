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

/// 群组设置相关 RPC
use serde::{Deserialize, Serialize};

/// serde 默认值：true（新群字段对老客户端/老数据缺省按宽松处理）
fn default_true() -> bool {
    true
}

/// serde 默认值：加入策略默认 1（允许申请，需审核）
fn default_join_policy() -> u8 {
    1
}

/// 更新群组设置请求
///
/// RPC路由: `group/settings/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsPatch {
    /// 是否开启加群审批（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_need_approval: Option<bool>,
    /// 成员是否可邀请（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_can_invite: Option<bool>,
    /// 群成员之间是否允许私自加好友（可选）。
    /// false 时，仅 `source=group` 来源的好友申请受限；群主/管理员、已是好友不受影响。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_member_add_friend: Option<bool>,
    /// 群是否允许被搜索发现（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_search: Option<bool>,
    /// 加入策略（可选）：0=不允许申请加入 1=允许申请需审核 2=允许直接加入
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_policy: Option<u8>,
    /// 是否全员禁言（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_muted: Option<bool>,
    /// 最大成员数（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_members: Option<u32>,
    /// 群公告（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announcement: Option<String>,
    /// 群描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 更新群组设置请求
///
/// RPC路由: `group/settings/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsUpdateRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID
    pub operator_id: u64,
    /// 更新项
    pub settings: GroupSettingsPatch,
}

/// 获取群组设置请求
///
/// RPC路由: `group/settings/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsGetRequest {
    /// 群组ID
    pub group_id: u64,
    /// 用户ID
    pub user_id: u64,
}

/// 全员禁言请求
///
/// RPC路由: `group/settings/mute_all`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMuteAllRequest {
    /// 群组ID
    pub group_id: u64,
    /// 操作者ID
    pub operator_id: u64,
    /// 是否全员禁言
    #[serde(alias = "mute_all")]
    pub muted: bool,
}

/// 更新群组设置响应
///
/// RPC路由: `group/settings/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsUpdateResponse {
    pub success: bool,
    pub group_id: String,
    pub message: String,
    pub updated_count: u32,
    pub updated_at: u64,
}

/// 获取群组设置响应
///
/// RPC路由: `group/settings/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsData {
    pub join_need_approval: bool,
    pub member_can_invite: bool,
    /// 群成员之间是否允许私自加好友（默认 true）
    #[serde(default = "default_true")]
    pub allow_member_add_friend: bool,
    /// 群是否允许被搜索发现（默认 true）
    #[serde(default = "default_true")]
    pub allow_search: bool,
    /// 加入策略：0=不允许申请加入 1=允许申请需审核 2=允许直接加入（默认 1）
    #[serde(default = "default_join_policy")]
    pub join_policy: u8,
    pub all_muted: bool,
    pub max_members: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announcement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 获取群组设置响应
///
/// RPC路由: `group/settings/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettingsGetResponse {
    pub group_id: u64,
    pub settings: GroupSettingsData,
}

/// 全员禁言响应
///
/// RPC路由: `group/settings/mute_all`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMuteAllResponse {
    pub success: bool,
    pub group_id: String,
    pub all_muted: bool,
    pub message: String,
    pub operator_id: String,
    pub updated_at: u64,
}

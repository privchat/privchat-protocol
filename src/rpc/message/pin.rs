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

/// 群消息置顶相关 RPC 类型定义
///
/// 仅群主/管理员可置顶/取消置顶群内消息；普通成员只读。
use serde::{Deserialize, Serialize};

/// 置顶 / 取消置顶群消息请求
///
/// RPC路由: `message/pin`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePinRequest {
    /// 群组ID
    pub group_id: u64,
    /// 频道ID（消息所在通信通道，用于校验）
    pub channel_id: u64,
    /// 服务端消息ID
    pub message_id: u64,
    /// true=置顶，false=取消置顶
    pub pinned: bool,

    /// 操作者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub operator_id: u64,
}

/// 置顶 / 取消置顶群消息响应
///
/// RPC路由: `message/pin`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePinResponse {
    pub success: bool,
    pub group_id: u64,
    pub message_id: u64,
    pub pinned: bool,
    /// 置顶时间（Unix 毫秒）；取消置顶时为 None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_at: Option<u64>,
    /// 置顶操作者；取消置顶时为 None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_by: Option<u64>,
}

/// 获取群置顶消息列表请求
///
/// RPC路由: `message/pin/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePinListRequest {
    /// 群组ID
    pub group_id: u64,

    /// 请求者ID（服务器端填充）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 单条置顶消息条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedMessageItem {
    pub message_id: u64,
    pub channel_id: u64,
    pub pinned_by: u64,
    pub pinned_at: u64,
}

/// 获取群置顶消息列表响应
///
/// RPC路由: `message/pin/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePinListResponse {
    pub group_id: u64,
    pub items: Vec<PinnedMessageItem>,
}

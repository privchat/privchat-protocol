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

/// 消息反应（Reaction）相关 RPC
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 添加消息反应请求
///
/// RPC路由: `message/reaction/add`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionAddRequest {
    /// 服务端消息ID
    pub server_message_id: u64,
    /// 频道ID（可选，用于验证）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    /// Emoji表情
    pub emoji: String,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 移除消息反应请求
///
/// RPC路由: `message/reaction/remove`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionRemoveRequest {
    /// 服务端消息ID
    pub server_message_id: u64,
    /// Emoji表情
    pub emoji: String,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取消息反应列表请求
///
/// RPC路由: `message/reaction/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionListRequest {
    /// 服务端消息ID
    pub server_message_id: u64,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取消息反应统计请求
///
/// RPC路由: `message/reaction/stats`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionStatsRequest {
    /// 服务端消息ID
    pub server_message_id: u64,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 添加消息反应响应
///
/// RPC路由: `message/reaction/add`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type MessageReactionAddResponse = bool;

/// 移除消息反应响应
///
/// RPC路由: `message/reaction/remove`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type MessageReactionRemoveResponse = bool;

/// 获取消息反应列表响应
///
/// RPC路由: `message/reaction/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionListResponse {
    pub success: bool,
    pub reactions: HashMap<String, Vec<u64>>,
    pub total_count: usize,
}

/// 获取消息反应统计响应
///
/// RPC路由: `message/reaction/stats`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionStatsData {
    pub reactions: HashMap<String, Vec<u64>>,
    pub total_count: usize,
}

/// 获取消息反应统计响应
///
/// RPC路由: `message/reaction/stats`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionStatsResponse {
    pub success: bool,
    pub stats: MessageReactionStatsData,
}

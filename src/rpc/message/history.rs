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

/// 消息历史相关 RPC
use serde::{Deserialize, Serialize};

/// 获取消息历史请求
///
/// RPC路由: `message/history/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryGetRequest {
    /// 用户ID
    pub user_id: u64,
    /// 频道ID
    pub channel_id: u64,
    /// 起始服务端消息ID（可选，用于分页）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_server_message_id: Option<u64>,
    /// 限制数量（可选，默认50）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// 消息历史响应
///
/// RPC路由: `message/history/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryItem {
    pub message_id: u64,
    pub channel_id: u64,
    pub sender_id: u64,
    pub content: String,
    pub message_type: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
    pub revoked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_by: Option<u64>,
}

/// 消息历史响应
///
/// RPC路由: `message/history/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryResponse {
    pub messages: Vec<MessageHistoryItem>,
    #[serde(default)]
    pub total: usize,
    pub has_more: bool,
}

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
    pub timestamp: u64,
    /// per-channel pts（server 端 message_seq）。SDK 回填本地库时必须落此值，
    /// 本地显示排序权威 = (pts, server_message_id)（MESSAGE_HISTORY spec §2.5）。
    /// Option + default 向后兼容老 server。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_seq: Option<i64>,
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

/// 云端历史搜索请求（MESSAGE_HISTORY spec §4）
///
/// RPC路由: `message/history/search`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistorySearchRequest {
    /// 关键词（字符数 2..=64，服务端强制）
    pub query: String,
    /// "GLOBAL" | "CHANNEL"
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u64>,
    /// keyset 游标："created_at:message_id"（服务端 created_at DESC, message_id DESC）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// 搜索命中（snippet 投影——**禁止**回填本地 message 表；完整消息走 get/around）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistorySearchHit {
    pub channel_id: u64,
    pub message_id: u64,
    pub sender_user_id: u64,
    /// 毫秒时间戳
    pub created_at: i64,
    pub message_type: String,
    pub snippet: String,
    /// 命中区间（相对 snippet 的字符偏移 [start,end)），供 UI 高亮
    #[serde(default)]
    pub highlight_ranges: Vec<(u32, u32)>,
}

/// 云端历史搜索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistorySearchResponse {
    pub hits: Vec<MessageHistorySearchHit>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// 消息定位上下文请求（jump-to-message，MESSAGE_HISTORY spec §5）
///
/// RPC路由: `message/history/around`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryAroundRequest {
    pub channel_id: u64,
    /// anchor 消息（不存在/跨频道/软删/撤回 → 服务端统一 not_found）
    pub message_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_limit: Option<u32>,
}

/// 消息定位上下文响应——返回**完整消息**（与 history/get 同视图），SDK 应回填本地缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryAroundResponse {
    /// anchor 之前（ASC）
    pub before_messages: Vec<MessageHistoryItem>,
    pub anchor_message: MessageHistoryItem,
    /// anchor 之后（ASC）
    pub after_messages: Vec<MessageHistoryItem>,
    #[serde(default)]
    pub has_more_before: bool,
    #[serde(default)]
    pub has_more_after: bool,
}

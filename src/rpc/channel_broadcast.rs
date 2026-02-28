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

/// 广播频道相关 RPC（用于订阅号/频道功能）
use serde::{Deserialize, Serialize};

/// 订阅广播频道请求
///
/// RPC路由: `channel/broadcast/subscribe`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBroadcastSubscribeRequest {
    /// 用户ID（服务器端从 ctx 填充）
    #[serde(default)]
    pub user_id: u64,

    /// 广播频道ID
    pub channel_id: u64,
}

/// 订阅广播频道响应
///
/// RPC路由: `channel/broadcast/subscribe`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBroadcastSubscribeResponse {
    pub status: String,
    pub message: String,
    pub channel_id: u64,
    pub user_id: u64,
    pub subscribed_at: String,
}

/// 通用动作响应（当前 create/publish/list/content/list 在服务端返回该形态）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBroadcastActionResponse {
    pub status: String,
    pub action: String,
    pub timestamp: String,
}

/// 创建广播频道请求
///
/// RPC路由: `channel/broadcast/create`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBroadcastCreateRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// 创建广播频道响应
///
/// RPC路由: `channel/broadcast/create`
pub type ChannelBroadcastCreateResponse = ChannelBroadcastActionResponse;

/// 获取广播频道列表请求
///
/// RPC路由: `channel/broadcast/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBroadcastListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

/// 获取广播频道列表响应
///
/// RPC路由: `channel/broadcast/list`
pub type ChannelBroadcastListResponse = ChannelBroadcastActionResponse;

/// 频道内容发布请求
///
/// RPC路由: `channel/content/publish`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelContentPublishRequest {
    pub channel_id: u64,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// 频道内容发布响应
///
/// RPC路由: `channel/content/publish`
pub type ChannelContentPublishResponse = ChannelBroadcastActionResponse;

/// 频道内容列表请求
///
/// RPC路由: `channel/content/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelContentListRequest {
    pub channel_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

/// 频道内容列表响应
///
/// RPC路由: `channel/content/list`
pub type ChannelContentListResponse = ChannelBroadcastActionResponse;

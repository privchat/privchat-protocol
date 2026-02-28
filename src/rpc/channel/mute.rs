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

/// 频道静音 RPC
use serde::{Deserialize, Serialize};

/// 设置频道静音请求
///
/// RPC路由: `channel/mute`
///
/// 设置频道静音后，该频道的新消息将不会推送通知。
/// 这是用户个人的偏好设置，适用于私聊和群聊。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMuteRequest {
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
    /// 频道ID
    pub channel_id: u64,
    /// 是否静音
    pub muted: bool,
}

/// 设置频道静音响应
///
/// RPC路由: `channel/mute`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type ChannelMuteResponse = bool;

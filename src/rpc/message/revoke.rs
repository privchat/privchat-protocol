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

/// 消息撤回 RPC
use serde::{Deserialize, Serialize};

/// 撤回消息请求
///
/// RPC路由: `message/revoke`
///
/// 🔐 安全设计：不传递 user_id
/// - user_id 从服务端 session 中获取（可信来源）
/// - 防止客户端伪造身份
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRevokeRequest {
    /// 服务端消息ID
    pub server_message_id: u64,
    /// 频道ID（从本地数据库查询）
    pub channel_id: u64,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 撤回消息响应
///
/// RPC路由: `message/revoke`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type MessageRevokeResponse = bool;

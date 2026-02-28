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

use crate::presence::*;
use serde::{Deserialize, Serialize};

/// RPC: presence/subscribe
/// 订阅在线状态（打开私聊会话时调用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePresenceRequest {
    /// 要订阅的用户ID列表
    pub user_ids: Vec<u64>,
}

/// 订阅在线状态响应
///
/// RPC路由: `presence/subscribe`
/// Handler 只返回 data 负载；外层 code/message 由 RPC 层封装，此处仅保留业务字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePresenceResponse {
    /// 所有用户的初始在线状态
    pub initial_statuses: std::collections::HashMap<u64, OnlineStatusInfo>,
}

/// RPC: presence/unsubscribe
/// 取消订阅（关闭私聊会话时调用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribePresenceRequest {
    /// 要取消订阅的用户ID列表
    pub user_ids: Vec<u64>,
}

/// 取消订阅在线状态响应
///
/// RPC路由: `presence/unsubscribe`
/// 与 reaction 等一致，data 为裸 bool（true/false），成功失败由外层 code 表示
pub type UnsubscribePresenceResponse = bool;

/// RPC: presence/typing
/// 发送输入状态通知
pub use crate::presence::{TypingIndicatorRequest, TypingIndicatorResponse};

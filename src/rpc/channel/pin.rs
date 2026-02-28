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

/// 频道置顶 RPC
use serde::{Deserialize, Serialize};

/// 置顶频道请求
///
/// RPC路由: `channel/pin`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPinRequest {
    /// 用户ID
    pub user_id: u64,
    /// 频道ID
    pub channel_id: u64,
    /// 是否置顶
    pub pinned: bool,
}

/// 置顶频道响应
///
/// RPC路由: `channel/pin`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type ChannelPinResponse = bool;

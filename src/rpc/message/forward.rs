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

//! 单条转发（MEDIA_REFERENCE_AND_FORWARD_SPEC §6）。
//!
//! 🔴 客户端**不提交**：`sender_id`、消息 payload、`file_id`、`thumbnail_file_id`、
//! hash、CEK。全部由服务端从源消息复制。
//!
//! 「客户端从不指定文件」这一条不是省事，是安全前提：伪造媒体描述符的问题在
//! 构造上就不存在，因此不需要 Telegram `file_reference` 那套 HMAC capability。

use serde::{Deserialize, Serialize};

/// 转发请求。
///
/// 🔴 三个 id 在线上一律是**十进制字符串**（`serde_u64`）：TS 的 `number` 只有
/// 2^53 精度，雪花 ID 走 JSON 数字会被舍入成另一条消息，而且不报错。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageForwardRequest {
    /// 源消息所在会话。
    #[serde(with = "crate::serde_u64")]
    pub source_channel_id: u64,
    /// 要转发的消息。
    #[serde(with = "crate::serde_u64")]
    pub source_message_id: u64,
    /// 转发到哪个会话。
    #[serde(with = "crate::serde_u64")]
    pub target_channel_id: u64,
    /// 幂等键。
    ///
    /// 🔴 **不是全局键**：唯一性作用域是 `(认证 uid, device_id, client_request_id)`。
    /// 做成全局键会让两个账号用同一个 request id 时互相判重——后一个人的转发
    /// 会静默变成「返回前一个人的消息」。
    pub client_request_id: String,
}

/// 转发响应：目标会话里新产生的那条消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageForwardResponse {
    #[serde(with = "crate::serde_u64")]
    pub message_id: u64,
    #[serde(with = "crate::serde_u64")]
    pub channel_id: u64,
    /// 目标会话分配的 PTS。
    pub pts: u64,
    /// 服务端记录的发送时间（毫秒）。
    pub created_at: i64,
    /// 幂等命中：这次请求没有新建消息，返回的是之前那条。
    pub deduplicated: bool,
}

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

/// 频道置顶 RPC
use serde::{Deserialize, Serialize};

/// 置顶频道请求
///
/// RPC路由: `channel/pin`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPinRequest {
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
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

#[cfg(test)]
mod tests {
    use super::ChannelPinRequest;
    use serde_json::json;

    #[test]
    fn pin_request_does_not_require_client_user_id() {
        let request: ChannelPinRequest = serde_json::from_value(json!({
            "channel_id": 42,
            "pinned": true
        }))
        .expect("channel pin payload should not require user_id");

        assert_eq!(request.user_id, 0);
        assert_eq!(request.channel_id, 42);
        assert!(request.pinned);
    }

    #[test]
    fn pin_request_ignores_spoofed_client_user_id() {
        let request: ChannelPinRequest = serde_json::from_value(json!({
            "user_id": 999,
            "channel_id": 42,
            "pinned": false
        }))
        .expect("channel pin payload should deserialize");

        assert_eq!(request.user_id, 0);
        assert_eq!(request.channel_id, 42);
        assert!(!request.pinned);
    }
}

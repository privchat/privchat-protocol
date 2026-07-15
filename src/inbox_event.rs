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

//! User Inbox Event Envelope —— 跨业务用户级"待处理事件"通用契约。
//!
//! 详细规范见 `spec/05-feature/USER_INBOX_EVENT_ENVELOPE_SPEC.md`。本文件
//! 仅定义 wire 结构体，**不携带任何业务语义** —— 业务字段全部放进 [`payload`]
//! 这个 `serde_json::Value`，由各业务的具体 payload 结构体（也定义在
//! `privchat-protocol`）填入。
//!
//! ## v1 关键不变式
//!
//! 1. **online-push-only**：v1 envelope 仅用于 `PushMessageRequest.payload`
//!    的在线推送，不进 DB / 不进离线队列。离线补偿仍走各业务的 `*/pending`
//!    类型 RPC。
//! 2. **`event_id` 仅供 dedupe，不可用于 diff**：v1 没有 cursor，server 不
//!    保证 event_id 在用户维度单调；客户端只能拿 event_id 去重，不能拿它
//!    驱动增量拉取。v2 才会引入 `cursor` 字段（user-scoped monotonic），
//!    `event_id` 的语义跨版本保持稳定。
//! 3. **不冗余 type**：事件类型来自外层 `PushMessageRequest.topic`，envelope
//!    自身**不**重复写 `type` —— 避免一致性纪律负担。
//! 4. **`aggregate_id` 必须是单个稳定标识**：禁止 `"a:b"` 这种复合编码；
//!    复合语义放进 [`payload`]。

use serde::{Deserialize, Serialize};

/// 当前 envelope schema 版本。v1 仅支持在线推送语义。
pub const USER_INBOX_EVENT_SCHEMA_VERSION_V1: u32 = 1;

/// 用户收件箱事件信封 v1。
///
/// 作为 `PushMessageRequest.payload` 的 JSON 主体；外层 `topic` 即事件类型
/// （如 `friend.request.received`）。具体业务字段放 [`payload`] 字段里，由
/// 各业务自己的 payload 结构体（同样定义在 `privchat-protocol`）填入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInboxEventEnvelope {
    /// envelope schema 版本号。当前固定 [`USER_INBOX_EVENT_SCHEMA_VERSION_V1`]。
    pub schema_version: u32,

    /// 全局唯一事件 ID（server snowflake，十进制字符串）。
    ///
    /// **客户端 dedupe 用**。跨 v1/v2 语义稳定，绝不复用为 user-scoped cursor。
    pub event_id: String,

    /// 业务对象类型。例如 `"friend_request"` / `"group_invite"`。小写下划线。
    pub aggregate_type: String,

    /// 业务对象的**单一稳定标识**。禁止复合编码（不写 `"a:b"`）。
    /// 复合语义放 [`payload`]。
    pub aggregate_id: String,

    /// 事件发生时间，毫秒时间戳。
    pub created_at: i64,

    /// 事件过期时间，毫秒时间戳。`None` = 不过期（业务自己决定语义）。
    ///
    /// v1 当前唯一消费者是好友申请推送，over-due 后客户端可丢弃；权威过期
    /// 判断仍由 server 在 `friend/pending` 等接口里执行。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,

    /// 业务自定义负载。每类事件对应一个具体的 payload 结构体（见
    /// `payloads` 子模块），客户端按外层 `topic` 决定如何反序列化。
    pub payload: serde_json::Value,
}

/// v1 事件类型常量（== outer `PushMessageRequest.topic`）。
///
/// envelope 自身不冗余 type 字段，但 server / SDK 双方需要约定可用的 topic
/// 字符串。新增事件类型时同步往这里加。
pub mod topics {
    /// 收到新好友申请。target = 被申请人；aggregate_type = `friend_request`；
    /// aggregate_id = 申请人 user_id 字符串（在接收人视角下 (self, requester)
    /// 唯一对应一条 pending 记录，receiver 由 push 通道隐含）。payload =
    /// [`super::payloads::FriendRequestReceivedPayload`]。
    pub const FRIEND_REQUEST_RECEIVED: &str = "friend.request.received";

    /// 我发出了一条新好友申请——给 requester 自己的其他设备的唤醒事件。
    /// target = requester 自己；aggregate_type = `friend_request`；
    /// aggregate_id = 被申请人 target_user_id 字符串。payload =
    /// [`super::payloads::FriendRequestSentPayload`]。
    pub const FRIEND_REQUEST_SENT: &str = "friend.request.sent";

    /// 一条好友申请的状态发生变化（accepted / rejected / recalled / expired）。
    /// **双方所有设备**都会收到。aggregate_type = `friend_request`；
    /// aggregate_id = peer_user_id（接收人视角下的"对端"，requester 视角下的
    /// 被申请人）。payload = [`super::payloads::FriendRequestStatusChangedPayload`]。
    pub const FRIEND_REQUEST_STATUS_CHANGED: &str = "friend.request.status_changed";
}

/// 各事件类型对应的 typed payload 结构体。
///
/// **不允许在业务代码里直接 `json!({...})` 构造 envelope.payload**——必须用
/// 这里的具体结构体走 [`serde_json::to_value`]。
pub mod payloads {
    use serde::{Deserialize, Serialize};

    /// `friend.request.received` 事件 payload。
    ///
    /// 字段语义：
    /// - `requester_id` / `receiver_id`：双方 user_id；envelope 的
    ///   `aggregate_id` 即 `requester_id`（在接收人视角下唯一锁定一条
    ///   pending request）。
    /// - `message`：申请附言（可为空字符串）。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FriendRequestReceivedPayload {
        pub requester_id: u64,
        pub receiver_id: u64,
        pub message: String,
    }

    /// `friend.request.sent` 事件 payload——A 在某个设备发起申请后，A 自己其他
    /// 设备收到此事件，触发 entity sync 把"我发出的 pending"拉到本地。
    ///
    /// 仍然是 hint，不是事实源；客户端必须靠 `entity/sync_entities("friend")`
    /// 拉到真实的 status / message / source。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FriendRequestSentPayload {
        pub requester_id: u64,
        pub target_user_id: u64,
    }

    /// `friend.request.status_changed` 事件 payload——一条申请的状态从 pending
    /// 切到 accepted / rejected / recalled / expired 时，双方所有设备都收到。
    ///
    /// `new_status` 携带新状态整型值（与 friendships.status 同枚举：
    /// 1=accepted / 3=rejected / 4=recalled / 5=expired）。仍然是 hint，
    /// 权威态从 entity sync 拉。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FriendRequestStatusChangedPayload {
        pub requester_id: u64,
        pub target_user_id: u64,
        pub new_status: i16,
    }
}

impl UserInboxEventEnvelope {
    /// 构造 v1 envelope（[`schema_version`] 自动填 1）。
    ///
    /// 推荐用法：业务侧用 [`serde_json::to_value`] 把自己的 typed payload 结构
    /// 体（参见 [`payloads`]）转 Value 后传入，不要手拼 JSON。
    pub fn new_v1(
        event_id: u64,
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        created_at: i64,
        expires_at: Option<i64>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            schema_version: USER_INBOX_EVENT_SCHEMA_VERSION_V1,
            event_id: event_id.to_string(),
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            created_at,
            expires_at,
            payload,
        }
    }

    /// 序列化成 JSON 字节，供放进 `PushMessageRequest.payload`。
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

#[cfg(test)]
mod tests {
    use super::payloads::FriendRequestReceivedPayload;
    use super::*;

    #[test]
    fn v1_envelope_roundtrip() {
        let payload = FriendRequestReceivedPayload {
            requester_id: 10001,
            receiver_id: 10002,
            message: "我是 Alice".to_string(),
        };
        let env = UserInboxEventEnvelope::new_v1(
            577943304432390144_u64,
            "friend_request",
            "10001",
            1_770_000_000_000,
            Some(1_770_600_000_000),
            serde_json::to_value(&payload).unwrap(),
        );

        let json = serde_json::to_string(&env).unwrap();
        // 不应包含 type 字段（外层 topic 提供）
        assert!(!json.contains("\"type\""));
        // 必须包含 schema_version=1
        assert!(json.contains("\"schema_version\":1"));
        // event_id 是字符串
        assert!(json.contains("\"event_id\":\"577943304432390144\""));

        let parsed: UserInboxEventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.aggregate_type, "friend_request");
        assert_eq!(parsed.aggregate_id, "10001");
        let p: FriendRequestReceivedPayload = serde_json::from_value(parsed.payload).unwrap();
        assert_eq!(p.requester_id, 10001);
        assert_eq!(p.message, "我是 Alice");
    }

    #[test]
    fn expires_at_none_is_omitted() {
        let env = UserInboxEventEnvelope::new_v1(
            1,
            "friend_request",
            "1",
            0,
            None,
            serde_json::json!({}),
        );
        let json = serde_json::to_string(&env).unwrap();
        assert!(!json.contains("expires_at"));
    }
}

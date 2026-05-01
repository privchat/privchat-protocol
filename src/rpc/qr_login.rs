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

//! Web 扫码登录 RPC（spec QR_API §5）。
//!
//! 这是 Web/PC 客户端通过 **未认证 RPC 连接** 发起的扫码登录入口：
//! 客户端建连后立刻调 [routes::qr_login::CREATE_SCENE] 拿到 `scene_id` 和 `qr_token`，
//! 同时 server 把 `scene_id ↔ session_id` 绑进 publisher。后续 mobile 扫码 / 确认 /
//! 拒绝 / 超时事件由 server 直接推回这条 unauth 连接（不再走 HTTP 轮询）。
//!
//! 推送事件 topic：
//! - `qr_login.scanned`     —— mobile 已扫码（state=scanned）
//! - `qr_login.authorized`  —— mobile 确认登录，payload 为 application 登录返回结构
//! - `qr_login.rejected`    —— mobile 拒绝
//! - `qr_login.expired`     —— scene 超时
//!
//! 客户端拿到 `authorized` 事件后用其中的 access/refresh token 登录 application；
//! 是否在当前 unauth 连接上原地 upgrade 取决于 token 是否被 privchat-server 接受。

use crate::protocol::DeviceInfo;
use serde::{Deserialize, Serialize};

/// 创建扫码登录场景请求（spec §5）。
///
/// RPC 路由：[`routes::qr_login::CREATE_SCENE`]（白名单匿名）。
///
/// 同一 unauth 连接刷新二维码会替换之前绑定的 scene；老 scene 不再收到推送。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrLoginCreateSceneRequest {
    /// 业务用途，自由字符串（如 `"login"` / `"link"`）。
    pub purpose: String,

    /// Web/PC 端设备 ID（必填）。建议 UUID；和 mobile 登录一致用于设备审计。
    pub web_device_id: String,

    /// Web/PC 端设备信息（可选，用于 mobile 端展示「在 ChromeBook 登录」之类）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_device_info: Option<DeviceInfo>,

    /// 二维码有效期（秒），可选；服务端有合理默认值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_secs: Option<i64>,
}

/// 创建扫码登录场景响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrLoginCreateSceneResponse {
    /// 服务端生成的 scene_id（mobile 扫码时回传）。
    pub scene_id: String,

    /// 二维码原始内容；mobile 端拿这个去 application HTTP `/platform/qr-login/scan`。
    pub qr_token: String,

    /// 过期时间戳（Unix 毫秒）。
    pub expires_at: i64,

    /// RPC 推送 topic 字符串（仅供调试 / 兼容旧客户端用）。
    /// 当前实现 server 直推到本连接，客户端不需要订阅这个 topic；保留字段。
    pub rpc_topic: String,
}

/// 推送事件统一信封。
///
/// 服务端通过 PushMessageRequest 发送：
/// - `topic` = `qr_login.<event>`（如 `qr_login.scanned`）
/// - `payload` = `serde_json::to_vec(QrLoginPushEvent)`
///
/// 客户端按 `event` 字段分发；`data` 为事件特定 JSON：
/// - scanned  → [`QrLoginScannedData`]
/// - authorized → application 登录返回对象（`MemberLoginResponse`）原样透传
/// - rejected / expired → null
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrLoginPushEvent {
    /// 事件名，如 `"qr_login.scanned"` / `"qr_login.authorized"`。
    pub event: String,

    /// 关联的 scene_id。
    pub scene_id: String,

    /// 状态机当前状态字符串：`created` / `scanned` / `authorized` / `rejected` / `expired`。
    pub state: String,

    /// 事件特定 payload；不同事件结构不一样，使用 untyped JSON 避免协议层和 application
    /// 的 LoginResponse schema 强耦合。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// `qr_login.scanned` 事件 data 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrLoginScannedData {
    pub scanner_uid: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scanner_avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scanner_display_name: Option<String>,
    pub scanned_at: i64,
}

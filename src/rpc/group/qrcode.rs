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

//! 群组二维码相关 RPC（QR_CODE_SPEC v1.3）。
//!
//! 字段语义：
//! - `qr_key` 是 `privchat_groups.qr_key` 字段（UNIQUE NOT NULL, 16-char base62, 永久）
//! - URL 形态 `https://<host>/privchat:protocol/group/{get|join}?qrkey=<qr_key>`
//!
//! v1.0/v1.1 历史字段（已删除，不再兼容）：
//! - ~~`expire_seconds`~~ — 永久二维码无过期
//! - ~~`expire_at`~~ — 同上
//! - ~~URL `&token=` 参数~~ — UNIQUE qr_key 已足够安全
//! - ~~`generate` 路由~~ — 改名为 `get`（读取，不再生成新值）

use serde::{Deserialize, Serialize};

// ---------- group/qrcode/get ----------

/// 读取群二维码请求
///
/// RPC 路由: `group/qrcode/get`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeGetRequest {
    /// 群组 ID
    pub group_id: u64,
    /// 操作者 ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub operator_id: u64,
}

/// 读取群二维码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeGetResponse {
    /// 群二维码 opaque token（直接来自 `privchat_groups.qr_key`）
    pub qr_key: String,
    /// 已经拼好的完整 URL：
    /// `https://<qr_base_url>/privchat:protocol/group/join?qrkey=<qr_key>`
    pub qr_code: String,
    /// 群组 ID（回显）
    pub group_id: u64,
}

// ---------- group/qrcode/refresh ----------

/// 旋转群二维码请求
///
/// RPC 路由: `group/qrcode/refresh`
///
/// Owner/Admin 权限；执行 `UPDATE privchat_groups SET qr_key=$new WHERE group_id=?`，
/// 老 qr_key 立即作废。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeRefreshRequest {
    /// 群组 ID
    pub group_id: u64,
    /// 操作者 ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub operator_id: u64,
}

/// 旋转群二维码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeRefreshResponse {
    /// 旧 qr_key（已作废）
    pub old_qr_key: String,
    /// 新 qr_key
    pub new_qr_key: String,
    /// 已经拼好的新 URL
    pub qr_code: String,
    /// 群组 ID
    pub group_id: u64,
}

// ---------- group/join/qrcode ----------

/// 通过二维码加群请求
///
/// RPC 路由: `group/join/qrcode`
///
/// 客户端扫到 URL → parser 提 `qrkey` → 调本接口。
/// 服务端 `SELECT group_id FROM privchat_groups WHERE qr_key = $1` 反查，
/// 再走与邀请相同的 join_need_approval 流程（method = "qrcode"）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeJoinRequest {
    /// 从二维码 URL 中提取的 qrkey
    pub qr_key: String,
    /// 申请理由（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 用户 ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 通过二维码加群响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeJoinResponse {
    /// `"joined"` 或 `"pending"`（pending = 进入审批队列）
    pub status: String,
    /// 群组 ID（resolve 出来的）
    pub group_id: u64,
    /// 审批申请的 request_id（仅当 status="pending" 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// 提示文案（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 用户 ID（status="joined" 时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,
    /// 加群时间 Unix 毫秒时间戳（status="joined" 时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joined_at: Option<u64>,
}

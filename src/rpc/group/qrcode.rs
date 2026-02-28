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

/// 群组二维码相关 RPC
use serde::{Deserialize, Serialize};

/// 生成群组二维码请求
///
/// RPC路由: `group/qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeGenerateRequest {
    /// 群组ID
    pub group_id: u64,
    /// 过期时间（秒）（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_seconds: Option<u64>,

    /// 操作者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub operator_id: u64,
}

/// 通过二维码加入群组请求
///
/// RPC路由: `group/qrcode/join`
///
/// 客户端扫描二维码后，应在本地解析 URL 提取 qr_key 和 token，然后发送此请求。
///
/// 示例：
/// ```json
/// {
///   "qr_key": "abc123",
///   "token": "xyz",
///   "message": "我想加入群组"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeJoinRequest {
    /// QR Key（从二维码 URL 中提取的 qrkey 参数）
    pub qr_key: String,

    /// Token（可选，从二维码 URL 中提取的 token 参数，用于群组邀请验证）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// 申请理由（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 生成群组二维码响应
///
/// RPC路由: `group/qrcode/generate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeGenerateResponse {
    pub qr_key: String,
    pub qr_code: String,           // privchat://group/get?qrkey=xxx&token=yyy
    pub expire_at: Option<String>, // ISO 8601
    pub group_id: u64,
    pub created_at: String, // ISO 8601
}

/// 通过二维码加入群组响应
///
/// RPC路由: `group/qrcode/join`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupQRCodeJoinResponse {
    pub status: String, // "pending" 或 "joined"
    pub group_id: u64,
    pub request_id: Option<String>, // 如果需要审批
    pub message: Option<String>,
    pub expires_at: Option<String>, // ISO 8601
    pub user_id: Option<u64>,       // 如果已加入
    pub joined_at: Option<String>,  // ISO 8601，如果已加入
}

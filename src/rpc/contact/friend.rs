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

/// 好友相关 RPC
use serde::{Deserialize, Serialize};

/// 申请添加好友请求
///
/// RPC路由: `contact/friend/apply`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendApplyRequest {
    /// 目标用户ID
    pub target_user_id: u64,
    /// 申请消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 来源
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 来源ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,

    /// 申请者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub from_user_id: u64,
}

/// 接受好友申请请求
///
/// RPC路由: `contact/friend/accept`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendAcceptRequest {
    /// 申请者ID
    pub from_user_id: u64,
    /// 回复消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// 接受者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub target_user_id: u64,
}

/// 拒绝好友申请请求
///
/// RPC路由: `contact/friend/reject`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRejectRequest {
    /// 申请者ID
    pub from_user_id: u64,
    /// 拒绝者ID
    pub target_user_id: u64,
    /// 拒绝理由
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// 待处理好友请求列表
///
/// RPC路由: `contact/friend/pending`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendPendingRequest {
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 检查好友关系请求
///
/// RPC路由: `contact/friend/check`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendCheckRequest {
    /// 要检查的用户ID
    pub friend_id: u64,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 移除好友请求
///
/// RPC路由: `contact/friend/remove`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRemoveRequest {
    /// 要移除的好友ID
    pub friend_id: u64,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 发送好友申请响应
///
/// RPC路由: `contact/friend/apply`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendApplyResponse {
    pub user_id: u64,
    pub username: String,
    pub status: String,   // "pending"
    pub added_at: String, // ISO 8601
    pub message: Option<String>,
}

/// 接受好友申请响应
///
/// RPC路由: `contact/friend/accept`
/// 返回频道 ID（channel_id），客户端应使用此 ID 发送消息
pub type FriendAcceptResponse = u64;

/// 拒绝好友申请响应
///
/// RPC路由: `contact/friend/reject`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type FriendRejectResponse = bool;

/// 删除好友响应
///
/// RPC路由: `contact/friend/remove`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type FriendRemoveResponse = bool;

/// 检查好友关系响应
///
/// RPC路由: `contact/friend/check`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendCheckResponse {
    pub is_friend: bool,
    pub status: Option<String>, // "accepted", "pending", "deleted"
}

/// 待处理好友申请条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendPendingItem {
    pub from_user_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub created_at: String,
}

/// 待处理好友申请响应
///
/// RPC路由: `contact/friend/pending`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendPendingResponse {
    pub requests: Vec<FriendPendingItem>, // 注意：服务端返回的是 "requests" 而不是 "pending_requests"
    pub total: usize,
}

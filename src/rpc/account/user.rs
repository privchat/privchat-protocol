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

/// 用户信息相关 RPC
use serde::{Deserialize, Serialize};

/// 用户详情查看来源类型
///
/// 客户端调用 `account/user/detail` 时需通过 `source` 字段指定来源，
/// 服务端根据来源做对应的权限校验。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetailSourceType {
    /// 搜索来源，source_id = 搜索会话 ID
    Search,
    /// 群组来源，source_id = 群 ID
    Group,
    /// 好友来源，source_id = 好友 user_id
    Friend,
    /// 待处理好友申请，source_id 可忽略
    FriendPending,
    /// 名片分享来源，source_id = 分享 ID
    CardShare,
    /// 临时会话来源（聊天界面查看对方资料），source_id = channel_id
    Conversation,
}

impl DetailSourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Group => "group",
            Self::Friend => "friend",
            Self::FriendPending => "friend_pending",
            Self::CardShare => "card_share",
            Self::Conversation => "conversation",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "search" => Some(Self::Search),
            "group" => Some(Self::Group),
            "friend" => Some(Self::Friend),
            "friend_pending" => Some(Self::FriendPending),
            "card_share" => Some(Self::CardShare),
            "conversation" => Some(Self::Conversation),
            _ => None,
        }
    }
}

/// 用户类型
///
/// 对应 `AccountUserDetailResponse.user_type` 和数据库 `user.user_type` 字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserType;

impl UserType {
    /// 普通用户（默认，注册用户）
    pub const NORMAL: i16 = 0;
    /// 系统用户（ID 1~99，系统预定义）
    pub const SYSTEM: i16 = 1;
    /// 机器人（客服、助手等）
    pub const BOT: i16 = 2;

    pub fn label(value: i16) -> &'static str {
        match value {
            Self::NORMAL => "普通用户",
            Self::SYSTEM => "系统用户",
            Self::BOT => "机器人",
            _ => "未知",
        }
    }
}

/// 获取用户详情请求
///
/// RPC路由: `account/user/detail`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserDetailRequest {
    /// 目标用户ID
    pub target_user_id: u64,
    /// 来源
    pub source: String,
    /// 来源ID
    pub source_id: String,

    /// 当前用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 生成用户分享卡片请求
///
/// RPC路由: `account/user/share_card`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserShareCardRequest {
    /// 要分享的目标用户ID
    pub target_user_id: u64,
    /// 接收者ID
    pub receiver_id: u64,
    /// 过期时间（秒）（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_seconds: Option<u64>,

    /// 分享者ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub sharer_id: u64,
}

/// 更新用户信息请求
///
/// RPC路由: `account/user/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserUpdateRequest {
    /// 显示名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// 头像URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// 个人简介（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,

    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 获取用户详情响应
///
/// RPC路由: `account/user/detail`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserDetailResponse {
    pub user_id: u64,
    pub username: String,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub user_type: i16,
    pub is_friend: bool,
    pub can_send_message: bool,
    pub source_type: String,
    pub source_id: String,
    /// 是否已关注（仅 user_type=2 Bot 有意义；非 bot 永远 false）
    /// spec: `02-server/SERVICE_ACCOUNT_FOLLOW_SPEC`
    #[serde(default)]
    pub is_follow: bool,
}

/// 更新用户信息响应
///
/// RPC路由: `account/user/update`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserUpdateResponse {
    pub status: String,
    pub action: String,
    pub timestamp: u64,
}

/// 生成用户分享卡片响应
///
/// RPC路由: `account/user/share_card`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUserShareCardResponse {
    pub share_id: String,
    pub target_user_id: u64,
    pub receiver_id: u64,
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_at: Option<u64>,
}

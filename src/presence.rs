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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 在线状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnlineStatus {
    /// 在线（最近3分钟活跃）
    Online,
    /// 最近在线（1小时内）
    Recently,
    /// 上周在线（7天内）
    LastWeek,
    /// 上月在线（30天内）
    LastMonth,
    /// 很久之前
    LongTimeAgo,
    /// 离线（用户主动设置）
    Offline,
}

impl OnlineStatus {
    /// 从秒数计算在线状态
    pub fn from_elapsed_seconds(elapsed: i64) -> Self {
        match elapsed {
            0..=180 => OnlineStatus::Online,             // 3分钟内
            181..=3600 => OnlineStatus::Recently,        // 1小时内
            3601..=604800 => OnlineStatus::LastWeek,     // 7天内
            604801..=2592000 => OnlineStatus::LastMonth, // 30天内
            _ => OnlineStatus::LongTimeAgo,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            OnlineStatus::Online => "online",
            OnlineStatus::Recently => "recently",
            OnlineStatus::LastWeek => "last_week",
            OnlineStatus::LastMonth => "last_month",
            OnlineStatus::LongTimeAgo => "long_time_ago",
            OnlineStatus::Offline => "offline",
        }
    }
}

/// 在线状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineStatusInfo {
    /// 用户ID
    pub user_id: u64,
    /// 在线状态
    pub status: OnlineStatus,
    /// 最后活跃时间（Unix 时间戳）
    pub last_seen: i64,
    /// 在线设备列表
    pub online_devices: Vec<super::DeviceType>,
}

/// 订阅在线状态请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePresenceRequest {
    /// 要订阅的用户ID列表
    pub user_ids: Vec<u64>,
}

/// 订阅在线状态响应
/// RPC 层只返回 data 负载；外层 code/message 由 RPC 封装，此处仅保留业务字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePresenceResponse {
    /// 初始在线状态（订阅时立即返回）
    pub initial_statuses: HashMap<u64, OnlineStatusInfo>,
}

/// 取消订阅在线状态请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribePresenceRequest {
    /// 要取消订阅的用户ID列表
    pub user_ids: Vec<u64>,
}

/// 取消订阅在线状态响应
/// 与 reaction 等一致，data 为裸 bool，成功失败由外层 code 表示
pub type UnsubscribePresenceResponse = bool;

/// 获取在线状态请求（批量查询）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOnlineStatusRequest {
    /// 要查询的用户ID列表
    pub user_ids: Vec<u64>,
}

/// 获取在线状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOnlineStatusResponse {
    /// 响应码
    pub code: i32,
    /// 响应消息
    pub message: String,
    /// 在线状态映射
    pub statuses: HashMap<u64, OnlineStatusInfo>,
}

/// 在线状态变化通知（服务端主动推送）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineStatusChangeNotification {
    /// 用户ID
    pub user_id: u64,
    /// 旧状态
    pub old_status: OnlineStatus,
    /// 新状态
    pub new_status: OnlineStatus,
    /// 最后活跃时间
    pub last_seen: i64,
    /// 时间戳
    pub timestamp: i64,
}

/// 输入状态动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypingActionType {
    /// 正在输入文字
    Typing,
    /// 正在录音
    Recording,
    /// 正在上传照片
    UploadingPhoto,
    /// 正在上传视频
    UploadingVideo,
    /// 正在上传文件
    UploadingFile,
    /// 正在选择贴纸
    ChoosingSticker,
}

impl TypingActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TypingActionType::Typing => "typing",
            TypingActionType::Recording => "recording",
            TypingActionType::UploadingPhoto => "uploading_photo",
            TypingActionType::UploadingVideo => "uploading_video",
            TypingActionType::UploadingFile => "uploading_file",
            TypingActionType::ChoosingSticker => "choosing_sticker",
        }
    }
}

/// 输入状态通知请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicatorRequest {
    /// 会话ID
    pub channel_id: u64,
    /// 会话类型（1=私聊, 2=群聊）
    pub channel_type: u8,
    /// 是否正在输入
    pub is_typing: bool,
    /// 输入动作类型
    pub action_type: TypingActionType,
}

/// 输入状态通知响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicatorResponse {
    /// 响应码
    pub code: i32,
    /// 响应消息
    pub message: String,
}

/// 输入状态变化通知（服务端推送给会话中的其他成员）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingStatusNotification {
    /// 用户ID
    pub user_id: u64,
    /// 用户名（可选，用于显示）
    pub username: Option<String>,
    /// 会话ID
    pub channel_id: u64,
    /// 会话类型
    pub channel_type: u8,
    /// 是否正在输入
    pub is_typing: bool,
    /// 输入动作类型
    pub action_type: TypingActionType,
    /// 时间戳
    pub timestamp: i64,
}

/// 在线状态隐私设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresencePrivacySettings {
    /// 谁能看到我的在线状态
    pub show_online_to: PrivacyRule,
    /// 谁能看到我的最后上线时间
    pub show_last_seen_to: PrivacyRule,
}

/// 隐私规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyRule {
    /// 所有人
    Everyone,
    /// 仅联系人
    Contacts,
    /// 无人
    Nobody,
    /// 自定义
    Custom {
        /// 允许的用户列表
        allow_users: Vec<u64>,
        /// 拒绝的用户列表
        deny_users: Vec<u64>,
    },
}

impl Default for PresencePrivacySettings {
    fn default() -> Self {
        Self {
            show_online_to: PrivacyRule::Everyone,
            show_last_seen_to: PrivacyRule::Everyone,
        }
    }
}

/// 获取群组在线统计请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGroupOnlineStatsRequest {
    /// 群组ID
    pub group_id: u64,
    /// 是否需要返回在线用户列表（大群不返回）
    pub include_user_list: bool,
}

/// 获取群组在线统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGroupOnlineStatsResponse {
    /// 响应码
    pub code: i32,
    /// 响应消息
    pub message: String,
    /// 群组ID
    pub group_id: u64,
    /// 总成员数
    pub total_members: u32,
    /// 在线成员数
    pub online_count: u32,
    /// 在线用户列表（可选，大群不返回）
    pub online_users: Option<Vec<OnlineStatusInfo>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_online_status_calculation() {
        assert_eq!(OnlineStatus::from_elapsed_seconds(0), OnlineStatus::Online);
        assert_eq!(
            OnlineStatus::from_elapsed_seconds(180),
            OnlineStatus::Online
        );
        assert_eq!(
            OnlineStatus::from_elapsed_seconds(181),
            OnlineStatus::Recently
        );
        assert_eq!(
            OnlineStatus::from_elapsed_seconds(3600),
            OnlineStatus::Recently
        );
        assert_eq!(
            OnlineStatus::from_elapsed_seconds(3601),
            OnlineStatus::LastWeek
        );
    }

    #[test]
    fn test_privacy_rule_default() {
        let settings = PresencePrivacySettings::default();
        assert!(matches!(settings.show_online_to, PrivacyRule::Everyone));
        assert!(matches!(settings.show_last_seen_to, PrivacyRule::Everyone));
    }
}

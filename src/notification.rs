use serde::{Deserialize, Serialize};

/// 系统通知类型枚举
/// 
/// 用于各种会话中的系统通知消息，如好友请求、群组操作、红包等
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum NotificationType {
    // ========== 好友相关 ==========
    
    /// 好友请求已发送
    FriendRequestSent {
        request_id: u64,
        from_user_id: u64,
        to_user_id: u64,
        message: String,
    },
    
    /// 好友请求被接受
    FriendRequestAccepted {
        request_id: u64,
        user_id: u64,
        username: String,
        avatar: Option<String>,
    },
    
    /// 好友请求被拒绝
    FriendRequestRejected {
        request_id: u64,
        user_id: u64,
    },
    
    /// 好友被删除
    FriendDeleted {
        user_id: u64,
        username: String,
    },
    
    // ========== 群组相关 ==========
    
    /// 群组创建
    GroupCreated {
        group_id: u64,
        group_name: String,
        creator_id: u64,
        creator_name: String,
        member_count: u32,
    },
    
    /// 成员加入群组
    GroupMemberJoined {
        group_id: u64,
        group_name: String,
        user_id: u64,
        username: String,
        invited_by: Option<u64>,
        inviter_name: Option<String>,
    },
    
    /// 成员离开群组
    GroupMemberLeft {
        group_id: u64,
        group_name: String,
        user_id: u64,
        username: String,
    },
    
    /// 成员被踢出群组
    GroupMemberKicked {
        group_id: u64,
        group_name: String,
        user_id: u64,
        username: String,
        kicked_by: u64,
        kicker_name: String,
        reason: Option<String>,
    },
    
    /// 群组名称修改
    GroupNameChanged {
        group_id: u64,
        old_name: String,
        new_name: String,
        changed_by: u64,
        changer_name: String,
    },
    
    /// 群组头像修改
    GroupAvatarChanged {
        group_id: u64,
        group_name: String,
        changed_by: u64,
        changer_name: String,
        new_avatar_url: String,
    },
    
    /// 群组公告修改
    GroupAnnouncementChanged {
        group_id: u64,
        group_name: String,
        announcement: String,
        changed_by: u64,
        changer_name: String,
    },
    
    /// 群主转让
    GroupOwnerTransferred {
        group_id: u64,
        group_name: String,
        old_owner_id: u64,
        old_owner_name: String,
        new_owner_id: u64,
        new_owner_name: String,
    },
    
    /// 管理员添加
    GroupAdminAdded {
        group_id: u64,
        group_name: String,
        user_id: u64,
        username: String,
        added_by: u64,
        adder_name: String,
    },
    
    /// 管理员移除
    GroupAdminRemoved {
        group_id: u64,
        group_name: String,
        user_id: u64,
        username: String,
        removed_by: u64,
        remover_name: String,
    },
    
    /// 成员被禁言
    GroupMemberMuted {
        group_id: u64,
        group_name: String,
        user_id: u64,
        username: String,
        duration_seconds: u64,
        muted_by: u64,
        muter_name: String,
        reason: Option<String>,
    },
    
    /// 成员解除禁言
    GroupMemberUnmuted {
        group_id: u64,
        group_name: String,
        user_id: u64,
        username: String,
        unmuted_by: u64,
        unmuter_name: String,
    },
    
    /// 群组被解散
    GroupDismissed {
        group_id: u64,
        group_name: String,
        dismissed_by: u64,
        dismisser_name: String,
    },
    
    // ========== 红包相关 ==========
    
    /// 红包发送
    RedPacketSent {
        red_packet_id: String,
        from_user_id: u64,
        from_username: String,
        total_amount: i64,          // 单位：分
        count: u32,                 // 红包个数
        message: String,            // 祝福语
        red_packet_type: RedPacketType,
    },
    
    /// 红包被领取
    RedPacketReceived {
        red_packet_id: String,
        user_id: u64,
        username: String,
        amount: i64,                // 领取金额（分）
        timestamp: i64,
    },
    
    /// 红包已抢完
    RedPacketEmpty {
        red_packet_id: String,
        total_received: u32,        // 总共被领取数量
        total_amount: i64,          // 总共被领取金额
    },
    
    /// 红包过期
    RedPacketExpired {
        red_packet_id: String,
        remaining_count: u32,       // 剩余个数
        remaining_amount: i64,      // 剩余金额
    },
    
    // ========== 消息相关 ==========
    
    /// 消息被撤回
    MessageRevoked {
        server_message_id: u64,
        channel_id: u64,
        revoked_by: u64,
        revoker_name: String,
        revoked_at: i64,
    },
    
    /// 消息被置顶
    MessagePinned {
        server_message_id: u64,
        channel_id: u64,
        pinned_by: u64,
        pinner_name: String,
        pinned_at: i64,
    },
    
    /// 消息取消置顶
    MessageUnpinned {
        server_message_id: u64,
        channel_id: u64,
        unpinned_by: u64,
        unpinner_name: String,
        unpinned_at: i64,
    },
    
    /// 消息已读（已读回执）
    MessageRead {
        server_message_id: u64,
        channel_id: u64,
        reader_id: u64,
        reader_name: String,
        read_at: i64,
    },
    
    /// 消息被编辑
    MessageEdited {
        server_message_id: u64,
        channel_id: u64,
        editor_id: u64,
        editor_name: String,
        old_content: String,
        new_content: String,
        edited_at: i64,
    },
    
    // ========== 系统相关 ==========
    
    /// 系统维护通知
    SystemMaintenance {
        title: String,
        content: String,
        start_time: i64,
        end_time: i64,
        level: MaintenanceLevel,
    },
    
    /// 系统公告
    SystemAnnouncement {
        announcement_id: u64,
        title: String,
        content: String,
        level: AnnouncementLevel,
        published_at: i64,
    },
    
    /// 版本更新通知
    SystemVersionUpdate {
        version: String,
        description: String,
        update_url: String,
        force_update: bool,
    },
}

/// 红包类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedPacketType {
    /// 普通红包（固定金额）
    Normal,
    /// 拼手气红包（随机金额）
    Lucky,
    /// 专属红包（指定接收人）
    Exclusive { target_user_ids: Vec<u64> },
}

/// 维护级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceLevel {
    /// 常规维护
    Normal,
    /// 紧急维护
    Urgent,
    /// 计划维护
    Scheduled,
}

/// 公告级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnouncementLevel {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 重要
    Important,
    /// 紧急
    Critical,
}

/// 通知消息结构
/// 
/// 封装通知类型和相关元数据，用于在会话中显示系统通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// 通知ID（可选，用于去重）
    pub notification_id: Option<u64>,
    
    /// 通知类型
    pub notification_type: NotificationType,
    
    /// 显示文本（用于UI展示，可以根据语言本地化）
    pub display_text: String,
    
    /// 时间戳
    pub timestamp: i64,
    
    /// 所属会话ID
    pub channel_id: u64,
    
    /// 会话类型（1=私聊, 2=群聊）
    pub channel_type: u8,
    
    /// 是否需要持久化存储
    pub should_persist: bool,
    
    /// 额外元数据（可选）
    pub metadata: Option<serde_json::Value>,
}

impl NotificationMessage {
    /// 创建新的通知消息
    pub fn new(
        notification_type: NotificationType,
        display_text: String,
        channel_id: u64,
        channel_type: u8,
    ) -> Self {
        Self {
            notification_id: None,
            notification_type,
            display_text,
            timestamp: chrono::Utc::now().timestamp(),
            channel_id,
            channel_type,
            should_persist: true,
            metadata: None,
        }
    }
    
    /// 设置通知ID
    pub fn with_notification_id(mut self, id: u64) -> Self {
        self.notification_id = Some(id);
        self
    }
    
    /// 设置是否持久化
    pub fn with_persist(mut self, persist: bool) -> Self {
        self.should_persist = persist;
        self
    }
    
    /// 设置额外元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// 获取通知类型的字符串表示
    pub fn type_str(&self) -> &'static str {
        match &self.notification_type {
            NotificationType::FriendRequestSent { .. } => "friend_request_sent",
            NotificationType::FriendRequestAccepted { .. } => "friend_request_accepted",
            NotificationType::FriendRequestRejected { .. } => "friend_request_rejected",
            NotificationType::FriendDeleted { .. } => "friend_deleted",
            NotificationType::GroupCreated { .. } => "group_created",
            NotificationType::GroupMemberJoined { .. } => "group_member_joined",
            NotificationType::GroupMemberLeft { .. } => "group_member_left",
            NotificationType::GroupMemberKicked { .. } => "group_member_kicked",
            NotificationType::GroupNameChanged { .. } => "group_name_changed",
            NotificationType::GroupAvatarChanged { .. } => "group_avatar_changed",
            NotificationType::GroupAnnouncementChanged { .. } => "group_announcement_changed",
            NotificationType::GroupOwnerTransferred { .. } => "group_owner_transferred",
            NotificationType::GroupAdminAdded { .. } => "group_admin_added",
            NotificationType::GroupAdminRemoved { .. } => "group_admin_removed",
            NotificationType::GroupMemberMuted { .. } => "group_member_muted",
            NotificationType::GroupMemberUnmuted { .. } => "group_member_unmuted",
            NotificationType::GroupDismissed { .. } => "group_dismissed",
            NotificationType::RedPacketSent { .. } => "red_packet_sent",
            NotificationType::RedPacketReceived { .. } => "red_packet_received",
            NotificationType::RedPacketEmpty { .. } => "red_packet_empty",
            NotificationType::RedPacketExpired { .. } => "red_packet_expired",
            NotificationType::MessageRevoked { .. } => "message_revoked",
            NotificationType::MessagePinned { .. } => "message_pinned",
            NotificationType::MessageUnpinned { .. } => "message_unpinned",
            NotificationType::MessageRead { .. } => "message_read",
            NotificationType::MessageEdited { .. } => "message_edited",
            NotificationType::SystemMaintenance { .. } => "system_maintenance",
            NotificationType::SystemAnnouncement { .. } => "system_announcement",
            NotificationType::SystemVersionUpdate { .. } => "system_version_update",
        }
    }
}

/// 辅助函数：生成显示文本
impl NotificationMessage {
    /// 根据通知类型生成默认的显示文本（中文）
    pub fn generate_display_text_cn(notification_type: &NotificationType) -> String {
        match notification_type {
            NotificationType::FriendRequestAccepted { username, .. } => {
                format!("{} 接受了你的好友请求", username)
            }
            NotificationType::FriendDeleted { username, .. } => {
                format!("{} 删除了你的好友关系", username)
            }
            NotificationType::GroupMemberJoined { username, inviter_name, .. } => {
                if let Some(inviter) = inviter_name {
                    format!("{} 邀请 {} 加入了群聊", inviter, username)
                } else {
                    format!("{} 加入了群聊", username)
                }
            }
            NotificationType::GroupMemberLeft { username, .. } => {
                format!("{} 离开了群聊", username)
            }
            NotificationType::GroupMemberKicked { username, kicker_name, reason, .. } => {
                if let Some(r) = reason {
                    format!("{} 将 {} 移出了群聊（原因：{}）", kicker_name, username, r)
                } else {
                    format!("{} 将 {} 移出了群聊", kicker_name, username)
                }
            }
            NotificationType::GroupNameChanged { old_name, new_name, changer_name, .. } => {
                format!("{} 将群名称从「{}」改为「{}」", changer_name, old_name, new_name)
            }
            NotificationType::GroupOwnerTransferred { old_owner_name, new_owner_name, .. } => {
                format!("{} 将群主转让给 {}", old_owner_name, new_owner_name)
            }
            NotificationType::GroupAdminAdded { username, adder_name, .. } => {
                format!("{} 将 {} 设置为管理员", adder_name, username)
            }
            NotificationType::GroupMemberMuted { username, muter_name, duration_seconds, .. } => {
                let duration_text = format_duration(*duration_seconds);
                format!("{} 禁言了 {}（{}）", muter_name, username, duration_text)
            }
            NotificationType::RedPacketSent { from_username, message, .. } => {
                format!("{} 发送了红包「{}」", from_username, message)
            }
            NotificationType::RedPacketReceived { username, amount, .. } => {
                format!("{} 领取了红包（{:.2}元）", username, *amount as f64 / 100.0)
            }
            NotificationType::RedPacketEmpty { .. } => {
                "红包已被抢完".to_string()
            }
            NotificationType::MessageRevoked { revoker_name, .. } => {
                format!("{} 撤回了一条消息", revoker_name)
            }
            NotificationType::MessagePinned { pinner_name, .. } => {
                format!("{} 置顶了一条消息", pinner_name)
            }
            NotificationType::MessageRead { reader_name, .. } => {
                format!("{} 已读", reader_name)
            }
            NotificationType::SystemMaintenance { title, .. } => {
                format!("系统维护通知：{}", title)
            }
            NotificationType::SystemAnnouncement { title, .. } => {
                format!("系统公告：{}", title)
            }
            _ => "系统通知".to_string(),
        }
    }
}

/// 格式化时长
fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}秒", seconds)
    } else if seconds < 3600 {
        format!("{}分钟", seconds / 60)
    } else if seconds < 86400 {
        format!("{}小时", seconds / 3600)
    } else {
        format!("{}天", seconds / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notification_serialization() {
        let notification = NotificationType::GroupMemberJoined {
            group_id: 123,
            group_name: "测试群".to_string(),
            user_id: 456,
            username: "张三".to_string(),
            invited_by: Some(789),
            inviter_name: Some("李四".to_string()),
        };
        
        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("group_member_joined"));
        assert!(json.contains("张三"));
        
        let deserialized: NotificationType = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, NotificationType::GroupMemberJoined { .. }));
    }
    
    #[test]
    fn test_display_text_generation() {
        let notification = NotificationType::GroupMemberJoined {
            group_id: 123,
            group_name: "测试群".to_string(),
            user_id: 456,
            username: "张三".to_string(),
            invited_by: Some(789),
            inviter_name: Some("李四".to_string()),
        };
        
        let text = NotificationMessage::generate_display_text_cn(&notification);
        assert_eq!(text, "李四 邀请 张三 加入了群聊");
    }
    
    #[test]
    fn test_notification_message_builder() {
        let notification_type = NotificationType::FriendRequestAccepted {
            request_id: 1,
            user_id: 123,
            username: "Alice".to_string(),
            avatar: None,
        };
        
        let msg = NotificationMessage::new(
            notification_type.clone(),
            "Alice 接受了你的好友请求".to_string(),
            456,
            1,
        )
        .with_notification_id(789)
        .with_persist(true);
        
        assert_eq!(msg.notification_id, Some(789));
        assert_eq!(msg.channel_id, 456);
        assert!(msg.should_persist);
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30秒");
        assert_eq!(format_duration(120), "2分钟");
        assert_eq!(format_duration(3600), "1小时");
        assert_eq!(format_duration(86400), "1天");
    }
}

/// pts-Based 同步协议
/// 
/// 设计原则：
/// - pts（per-channel monotonic）为权威顺序
/// - local_message_id 用于幂等和关联
/// - 服务器是唯一仲裁方

use serde::{Deserialize, Serialize};

// ============================================================
// ClientSubmit - 客户端提交命令
// ============================================================

/// 客户端提交请求
/// 
/// RPC路由: `sync/submit`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSubmitRequest {
    /// 客户端消息号（Snowflake u64，用于幂等）
    pub local_message_id: u64,
    
    /// 频道 ID
    pub channel_id: u64,
    
    /// 频道类型（1=私聊，2=群聊）
    pub channel_type: u8,
    
    /// 客户端已知的最后 pts（用于间隙检测）
    pub last_pts: u64,
    
    /// 命令类型
    pub command_type: String,
    
    /// 命令负载（JSON）
    pub payload: serde_json::Value,
    
    /// 客户端时间戳（毫秒）
    pub client_timestamp: i64,
    
    /// 设备 ID（可选，用于多设备去重）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

/// 服务器提交响应
/// 
/// RPC路由: `sync/submit`
/// 注意：如果 decision 是 Rejected，SDK 层会返回错误，不会反序列化这个结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSubmitResponse {
    /// 服务器决策
    pub decision: ServerDecision,
    
    /// 分配的 pts（如果 accepted/transformed）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pts: Option<u64>,
    
    /// 服务器消息 ID（如果 accepted/transformed）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_msg_id: Option<u64>,
    
    /// 服务器时间戳
    pub server_timestamp: i64,
    
    /// 关联的 local_message_id
    pub local_message_id: u64,
    
    /// 是否需要补齐（has_gap）
    pub has_gap: bool,
    
    /// 服务器当前最新 pts
    pub current_pts: u64,
}

/// 服务器决策类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerDecision {
    /// 接受（原样接受）
    Accepted,
    /// 转换（服务器修改了某些字段）
    Transformed {
        /// 转换说明
        reason: String,
    },
    /// 拒绝（不符合规则）
    Rejected {
        /// 拒绝原因
        reason: String,
    },
}

// ============================================================
// GetDifference - 获取差异（补齐间隙）
// ============================================================

/// 获取差异请求
/// 
/// RPC路由: `sync/get_difference`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDifferenceRequest {
    /// 频道 ID
    pub channel_id: u64,
    
    /// 频道类型
    pub channel_type: u8,
    
    /// 客户端已知的最后 pts
    pub last_pts: u64,
    
    /// 限制数量（可选，默认 100）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// 获取差异响应
/// 
/// RPC路由: `sync/get_difference`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDifferenceResponse {
    /// Commits 列表（pts 递增）
    pub commits: Vec<ServerCommit>,
    
    /// 服务器当前最新 pts
    pub current_pts: u64,
    
    /// 是否还有更多（需要继续拉取）
    pub has_more: bool,
}

/// 服务器 Commit（权威事实）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCommit {
    /// pts（per-channel 单调递增）
    pub pts: u64,
    
    /// 服务器消息 ID
    pub server_msg_id: u64,
    
    /// 关联的 local_message_id（如果来自客户端）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_message_id: Option<u64>,
    
    /// 频道 ID
    pub channel_id: u64,
    
    /// 频道类型
    pub channel_type: u8,
    
    /// 消息类型
    pub message_type: String,
    
    /// 消息内容（JSON）
    pub content: serde_json::Value,
    
    /// 服务器时间戳（毫秒）
    pub server_timestamp: i64,
    
    /// 发送者 ID
    pub sender_id: u64,
    
    /// 发送者信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_info: Option<SenderInfo>,
}

/// 发送者信息（简化的用户信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderInfo {
    pub user_id: u64,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

// ============================================================
// GetChannelPts - 获取频道当前 pts（用于初始化）
// ============================================================

/// 获取频道 pts 请求
/// 
/// RPC路由: `sync/get_channel_pts`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetChannelPtsRequest {
    /// 频道 ID
    pub channel_id: u64,
    
    /// 频道类型
    pub channel_type: u8,
}

/// 获取频道 pts 响应
/// 
/// RPC路由: `sync/get_channel_pts`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetChannelPtsResponse {
    /// 当前 pts
    pub current_pts: u64,
}

// ============================================================
// BatchGetChannelPts - 批量获取多个频道的 pts
// ============================================================

/// 批量获取频道 pts 请求
/// 
/// RPC路由: `sync/batch_get_channel_pts`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGetChannelPtsRequest {
    /// 频道列表
    pub channels: Vec<ChannelIdentifier>,
}

/// 频道标识符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIdentifier {
    pub channel_id: u64,
    pub channel_type: u8,
}

/// 批量获取频道 pts 响应
/// 
/// RPC路由: `sync/batch_get_channel_pts`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGetChannelPtsResponse {
    /// 频道 pts 映射
    pub channel_pts_map: Vec<ChannelPtsInfo>,
}

/// 频道 pts 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPtsInfo {
    pub channel_id: u64,
    pub channel_type: u8,
    pub current_pts: u64,
}

// ============================================================
// Entity State Sync（实体状态同步，与 PTS 消息流正交）
// ============================================================
// 设计见 privchat-docs/design/ENTITY_SYNC_V1.md

/// 实体同步请求
///
/// RPC 路由: `entity/sync_entities`（待服务端实现）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntitiesRequest {
    /// 实体类型：friend, group, channel, group_member, user, user_settings, user_block 等（受控枚举）
    pub entity_type: String,
    /// 客户端上次同步到的版本号，0 或空表示全量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_version: Option<u64>,
    /// 可选：同步范围，如 group_member 需带 group_id，user 按需拉取时带 user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// 可选：每页数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// 单条实体同步项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntityItem {
    /// 实体 ID（如 user_id, group_id, channel_id）
    pub entity_id: String,
    /// 该实体最新版本号
    pub version: u64,
    /// true 表示服务端已删除（Tombstone）
    #[serde(default)]
    pub deleted: bool,
    /// 实体数据，具体字段随 entity_type 变化
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// 实体同步响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntitiesResponse {
    pub items: Vec<SyncEntityItem>,
    /// 本次同步完成后的最新版本号，客户端下次请求 since_version
    pub next_version: u64,
    /// 是否还有更多数据
    #[serde(default)]
    pub has_more: bool,
    /// 可选：客户端过旧时服务端返回最小支持版本，触发全量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<u64>,
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_submit_request_serialization() {
        let req = ClientSubmitRequest {
            local_message_id: 123456789,
            channel_id: 1001,
            channel_type: 1,
            last_pts: 100,
            command_type: "send_message".to_string(),
            payload: serde_json::json!({"text": "Hello"}),
            client_timestamp: 1700000000000,
            device_id: Some("device_001".to_string()),
        };
        
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("local_message_id"));
        assert!(json.contains("123456789"));
    }

    #[test]
    fn test_server_decision() {
        let decision_accepted = ServerDecision::Accepted;
        let decision_transformed = ServerDecision::Transformed {
            reason: "Content filtered".to_string(),
        };
        let decision_rejected = ServerDecision::Rejected {
            reason: "Spam detected".to_string(),
        };
        
        assert_eq!(decision_accepted, ServerDecision::Accepted);
        assert!(matches!(decision_transformed, ServerDecision::Transformed { .. }));
        assert!(matches!(decision_rejected, ServerDecision::Rejected { .. }));
    }
}

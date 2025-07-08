use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::collections::HashMap;

/// 消息基础trait
pub trait Message: Debug + Send + Sync {
    /// 获取消息类型
    fn message_type(&self) -> MessageType;
}

/// 消息类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// 连接请求 - 客户端发起连接
    ConnectRequest = 1,
    /// 连接响应 - 服务端连接确认
    ConnectResponse = 2,
    /// 断开连接请求 - 客户端主动断开
    DisconnectRequest = 3,
    /// 断开连接响应 - 服务端断开确认
    DisconnectResponse = 4,
    /// 发送消息请求 - 客户端发送消息
    SendRequest = 5,
    /// 发送消息响应 - 服务端发送确认
    SendResponse = 6,
    /// 接收消息请求 - 服务端推送消息
    RecvRequest = 7,
    /// 接收消息响应 - 客户端接收确认
    RecvResponse = 8,
    /// 批量接收消息请求 - 服务端批量推送消息
    RecvBatchRequest = 9,
    /// 批量接收消息响应 - 客户端批量接收确认
    RecvBatchResponse = 10,
    /// 心跳请求 - 客户端发送心跳
    PingRequest = 11,
    /// 心跳响应 - 服务端心跳回复
    PongResponse = 12,
    /// 订阅请求 - 客户端订阅频道
    SubscribeRequest = 13,
    /// 订阅响应 - 服务端订阅确认
    SubscribeResponse = 14,
    /// 推送消息请求 - 服务端推送频道消息
    PublishRequest = 15,
    /// 推送消息响应 - 客户端推送确认
    PublishResponse = 16,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => MessageType::ConnectRequest,
            2 => MessageType::ConnectResponse,
            3 => MessageType::DisconnectRequest,
            4 => MessageType::DisconnectResponse,
            5 => MessageType::SendRequest,
            6 => MessageType::SendResponse,
            7 => MessageType::RecvRequest,
            8 => MessageType::RecvResponse,
            9 => MessageType::RecvBatchRequest,
            10 => MessageType::RecvBatchResponse,
            11 => MessageType::PingRequest,
            12 => MessageType::PongResponse,
            13 => MessageType::SubscribeRequest,
            14 => MessageType::SubscribeResponse,
            15 => MessageType::PublishRequest,
            16 => MessageType::PublishResponse,
            _ => MessageType::ConnectRequest, // 默认值
        }
    }
}

impl From<MessageType> for u8 {
    fn from(msg_type: MessageType) -> Self {
        msg_type as u8
    }
}

/// 消息设置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageSetting {
    pub need_receipt: bool,
    pub signal: u8,
}

impl MessageSetting {
    pub fn new() -> Self {
        Self::default()
    }
}

/// 数据包结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet<T: Message> {
    pub message_type: MessageType,
    pub body: T,
}

impl<T: Message> Packet<T> {
    pub fn new(message_type: MessageType, body: T) -> Self {
        Self {
            message_type,
            body,
        }
    }
}

/// 连接请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    /// 认证类型
    pub auth_type: AuthType,
    /// 认证令牌
    pub auth_token: String,
    /// 客户端信息
    pub client_info: ClientInfo,
    /// 设备信息
    pub device_info: DeviceInfo,
    /// 协议版本
    pub protocol_version: String,
    /// 扩展属性
    pub properties: HashMap<String, String>,
}

/// 连接响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectResponse {
    /// 连接是否成功
    pub success: bool,
    /// 错误码
    pub error_code: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 用户ID
    pub user_id: Option<String>,
    /// 服务器分配的连接ID
    pub connection_id: Option<String>,
    /// 服务器信息
    pub server_info: Option<ServerInfo>,
    /// 心跳间隔（秒）
    pub heartbeat_interval: Option<u64>,
}

/// 认证类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// JWT令牌认证
    JWT,
    /// 用户名密码认证
    UserPassword,
    /// 第三方OAuth认证
    OAuth,
    /// 匿名认证
    Anonymous,
}

/// 客户端信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// 客户端类型
    pub client_type: String,
    /// 客户端版本
    pub version: String,
    /// 操作系统
    pub os: String,
    /// 操作系统版本
    pub os_version: String,
    /// 设备型号
    pub device_model: Option<String>,
    /// 应用包名
    pub app_package: Option<String>,
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备ID
    pub device_id: String,
    /// 设备名称
    pub device_name: String,
    /// 设备类型
    pub device_type: DeviceType,
    /// 推送令牌
    pub push_token: Option<String>,
    /// 设备指纹
    pub device_fingerprint: Option<String>,
}

/// 设备类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Tablet,
    IoT,
    Unknown,
}

/// 服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// 服务器版本
    pub version: String,
    /// 服务器名称
    pub name: String,
    /// 支持的功能
    pub features: Vec<String>,
    /// 最大消息大小
    pub max_message_size: u64,
    /// 连接超时时间
    pub connection_timeout: u64,
}

/// 断开连接请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectRequest {
    /// 断开原因
    pub reason: DisconnectReason,
    /// 附加信息
    pub message: Option<String>,
}

/// 断开连接响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectResponse {
    /// 确认断开
    pub acknowledged: bool,
}

/// 断开连接原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectReason {
    /// 用户主动断开
    UserInitiated,
    /// 服务器关闭
    ServerShutdown,
    /// 认证失败
    AuthenticationFailed,
    /// 协议错误
    ProtocolError,
    /// 超时
    Timeout,
    /// 重复连接
    DuplicateConnection,
    /// 服务器维护
    ServerMaintenance,
}

/// 通用消息包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePacket {
    /// 消息类型
    pub message_type: String,
    /// 消息ID
    pub message_id: Option<String>,
    /// 时间戳
    pub timestamp: u64,
    /// 消息体
    pub payload: serde_json::Value,
    /// 扩展头部
    pub headers: HashMap<String, String>,
}

/// 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// 错误码
    pub error_code: String,
    /// 错误信息
    pub error_message: String,
    /// 错误详情
    pub error_details: Option<HashMap<String, String>>,
    /// 时间戳
    pub timestamp: u64,
}

/// 发送消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendRequest {
    pub setting: MessageSetting,
    pub client_seq: u32,
    pub client_msg_no: String,
    pub stream_no: String,
    pub channel_id: String,
    pub channel_type: u8,
    pub expire: u32,
    pub from_uid: String,
    pub topic: String,
    pub payload: Vec<u8>,
}

impl SendRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SendRequest, self)
    }
    
    pub fn verify_string(&self) -> String {
        format!("{}:{}:{}", self.client_msg_no, self.channel_id, self.from_uid)
    }
}

/// 发送确认消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendResponse {
    pub client_seq: u32,
    pub message_id: BigInt,
    pub message_seq: u32,
    pub reason_code: u8,
}

impl SendResponse {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SendResponse, self)
    }
}

/// 接收消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecvRequest {
    pub setting: MessageSetting,
    pub msg_key: String,
    pub message_id: BigInt,
    pub message_seq: u32,
    pub client_msg_no: String,
    pub stream_no: String,
    pub stream_seq: u32,
    pub stream_flag: u8,
    pub timestamp: u32,
    pub channel_id: String,
    pub channel_type: u8,
    pub expire: u32,
    pub topic: String,
    pub from_uid: String,
    pub payload: Vec<u8>,
}

impl RecvRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::RecvRequest, self)
    }
    
    pub fn verify_string(&self) -> String {
        format!("{}:{}:{}", self.message_id, self.channel_id, self.from_uid)
    }
}

/// 接收确认消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecvResponse {
    pub succeed: bool,
    pub message: Option<String>,
}

impl RecvResponse {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::RecvResponse, self)
    }
}

/// 心跳消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PingRequest {
    pub timestamp: i64,
}

impl PingRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PingRequest, self)
    }
}

/// 心跳回复消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PongResponse {
    pub timestamp: i64,
}

impl PongResponse {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PongResponse, self)
    }
}

// DisconnectRequest 已在上面定义，删除重复定义

/// 订阅消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub setting: u8,
    pub client_msg_no: String,
    pub channel_id: String,
    pub channel_type: u8,
    pub action: u8,
    pub param: String,
}

impl SubscribeRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SubscribeRequest, self)
    }
}

/// 订阅确认消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscribeResponse {
    pub client_msg_no: String,
    pub channel_id: String,
    pub channel_type: u8,
    pub action: u8,
    pub reason_code: u8,
}

impl SubscribeResponse {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SubscribeResponse, self)
    }
}

/// 批量接收消息 - 用于服务器向客户端批量推送消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecvBatchRequest {
    /// 批量消息列表
    pub messages: Vec<RecvRequest>
}

impl RecvBatchRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::RecvBatchRequest, self)
    }
    
    /// 创建单个批次的批量消息
    pub fn single_batch(messages: Vec<RecvRequest>) -> Self {
        Self {
            messages,
        }
    }
    
    /// 创建多批次中的一个批次
    pub fn multi_batch(
        messages: Vec<RecvRequest>,
    ) -> Self {
        Self {
            messages,
        }
    }
    
    /// 获取消息数量
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
    
    /// 检查是否为空批次
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// 批量接收确认消息（客户端 → 服务端）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecvBatchResponse {
    pub succeed: bool,
    pub message: Option<String>,
}

impl RecvBatchResponse {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::RecvBatchResponse, self)
    }
    
    /// 创建成功确认
    pub fn success() -> Self {
        Self {
            succeed: true,
            message: Some("批量消息接收成功".to_string()),
        }
    }
    
    /// 创建失败确认
    pub fn failure(error_msg: &str) -> Self {
        Self {
            succeed: false,
            message: Some(error_msg.to_string()),
        }
    }
}

// DisconnectResponse 已在上面定义，删除重复定义

/// 频道推送消息（服务端 → 客户端广播）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishRequest {
    pub channel_id: String,        // 频道ID
    pub topic: Option<String>,     // 主题/标签（可选）
    pub timestamp: u64,            // 推送时间戳
    pub payload: Vec<u8>,          // 消息内容
    pub publisher: Option<String>, // 发布者（可选，可能是系统/机器人）
    pub message_id: Option<String>, // 消息ID（用于去重）
}

impl PublishRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PublishRequest, self)
    }
    
    /// 创建系统推送消息
    pub fn system_push(channel_id: &str, payload: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let message_id = format!("sys_{}_{}", channel_id, timestamp);
        
        Self {
            channel_id: channel_id.to_string(),
            topic: None,
            timestamp,
            payload,
            publisher: Some("system".to_string()),
            message_id: Some(message_id),
        }
    }
    
    /// 创建主题推送消息
    pub fn topic_push(channel_id: &str, topic: &str, payload: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let message_id = format!("topic_{}_{}", topic, timestamp);
        
        Self {
            channel_id: channel_id.to_string(),
            topic: Some(topic.to_string()),
            timestamp,
            payload,
            publisher: None,
            message_id: Some(message_id),
        }
    }
}

/// 推送确认消息（客户端 → 服务端）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishResponse {
    pub succeed: bool,
    pub message: Option<String>,
}

impl PublishResponse {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PublishResponse, self)
    }
    
    /// 创建成功确认
    pub fn success() -> Self {
        Self {
            succeed: true,
            message: Some("推送消息接收成功".to_string()),
        }
    }
    
    /// 创建失败确认
    pub fn failure(error_msg: &str) -> Self {
        Self {
            succeed: false,
            message: Some(error_msg.to_string()),
        }
    }
}

// 为所有消息类型实现 Message trait
impl Message for ConnectRequest {
    fn message_type(&self) -> MessageType {
        MessageType::ConnectRequest
    }
}

impl Message for ConnectResponse {
    fn message_type(&self) -> MessageType {
        MessageType::ConnectResponse
    }
}

impl Message for SendRequest {
    fn message_type(&self) -> MessageType {
        MessageType::SendRequest
    }
}

impl Message for SendResponse {
    fn message_type(&self) -> MessageType {
        MessageType::SendResponse
    }
}

impl Message for RecvRequest {
    fn message_type(&self) -> MessageType {
        MessageType::RecvRequest
    }
}

impl Message for RecvResponse {
    fn message_type(&self) -> MessageType {
        MessageType::RecvResponse
    }
}

impl Message for PingRequest {
    fn message_type(&self) -> MessageType {
        MessageType::PingRequest
    }
}

impl Message for PongResponse {
    fn message_type(&self) -> MessageType {
        MessageType::PongResponse
    }
}

impl Message for DisconnectRequest {
    fn message_type(&self) -> MessageType {
        MessageType::DisconnectRequest
    }
}

impl Message for SubscribeRequest {
    fn message_type(&self) -> MessageType {
        MessageType::SubscribeRequest
    }
}

impl Message for SubscribeResponse {
    fn message_type(&self) -> MessageType {
        MessageType::SubscribeResponse
    }
}

impl Message for RecvBatchRequest {
    fn message_type(&self) -> MessageType {
        MessageType::RecvBatchRequest
    }
}

impl Message for RecvBatchResponse {
    fn message_type(&self) -> MessageType {
        MessageType::RecvBatchResponse
    }
}

impl Message for DisconnectResponse {
    fn message_type(&self) -> MessageType {
        MessageType::DisconnectResponse
    }
}

impl Message for PublishRequest {
    fn message_type(&self) -> MessageType {
        MessageType::PublishRequest
    }
}

impl Message for PublishResponse {
    fn message_type(&self) -> MessageType {
        MessageType::PublishResponse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let connect_msg = ConnectRequest::new();
        let packet = connect_msg.create_packet();
        assert_eq!(packet.message_type, MessageType::ConnectRequest);
        
        let send_msg = SendRequest::new();
        let packet = send_msg.create_packet();
        assert_eq!(packet.message_type, MessageType::SendRequest);
    }
    
    #[test]
    fn test_message_types() {
        assert_eq!(MessageType::ConnectRequest as u8, 1);
        assert_eq!(MessageType::ConnectResponse as u8, 2);
        assert_eq!(MessageType::DisconnectRequest as u8, 3);
        assert_eq!(MessageType::DisconnectResponse as u8, 4);
        assert_eq!(MessageType::SendRequest as u8, 5);
        assert_eq!(MessageType::SendResponse as u8, 6);
        assert_eq!(MessageType::RecvRequest as u8, 7);
        assert_eq!(MessageType::RecvResponse as u8, 8);
        assert_eq!(MessageType::RecvBatchRequest as u8, 9);        // 批量接收消息
        assert_eq!(MessageType::RecvBatchResponse as u8, 10);     // 批量接收确认
        assert_eq!(MessageType::PingRequest as u8, 11);
        assert_eq!(MessageType::PongResponse as u8, 12);
        assert_eq!(MessageType::SubscribeRequest as u8, 13);
        assert_eq!(MessageType::SubscribeResponse as u8, 14);
        assert_eq!(MessageType::PublishRequest as u8, 15);          // 推送消息
        assert_eq!(MessageType::PublishResponse as u8, 16);       // 推送确认
    }
    
    #[test]
    fn test_message_type_conversion() {
        // 测试 u8 到 MessageType 的转换
        assert_eq!(MessageType::from(1u8), MessageType::ConnectRequest);
        assert_eq!(MessageType::from(2u8), MessageType::ConnectResponse);
        assert_eq!(MessageType::from(3u8), MessageType::DisconnectRequest);
        assert_eq!(MessageType::from(16u8), MessageType::PublishResponse);
        
        // 测试无效值的转换（应该返回默认值）
        assert_eq!(MessageType::from(0u8), MessageType::ConnectRequest);
        assert_eq!(MessageType::from(255u8), MessageType::ConnectRequest);
        
        // 测试 MessageType 到 u8 的转换
        assert_eq!(u8::from(MessageType::ConnectRequest), 1);
        assert_eq!(u8::from(MessageType::ConnectResponse), 2);
        assert_eq!(u8::from(MessageType::PublishResponse), 16);
        
        // 测试双向转换一致性
        for i in 1u8..=16u8 {
            let msg_type = MessageType::from(i);
            let back_to_u8 = u8::from(msg_type);
            assert_eq!(back_to_u8, i, "转换不一致：{} -> {:?} -> {}", i, msg_type, back_to_u8);
        }
    }
    
    #[test]
    fn test_message_setting() {
        let setting = MessageSetting::new();
        assert_eq!(setting.need_receipt, false);
        assert_eq!(setting.signal, 0);
        
        let setting2 = MessageSetting {
            need_receipt: true,
            signal: 1,
        };
        assert_eq!(setting2.need_receipt, true);
        assert_eq!(setting2.signal, 1);
    }
    
    #[test]
    fn test_batch_message() {
        // 创建批量消息
        let mut messages = Vec::new();
        for i in 1..=3 {
            let mut recv_msg = RecvRequest::new();
            recv_msg.message_id = BigInt::from(i);
            recv_msg.from_uid = format!("user_{}", i);
            recv_msg.payload = format!("Message {}", i).into_bytes();
            messages.push(recv_msg);
        }
        
        let batch_msg = RecvBatchRequest::single_batch(messages);
        assert_eq!(batch_msg.message_count(), 3);
        
        let packet = batch_msg.create_packet();
        assert_eq!(packet.message_type, MessageType::RecvBatchRequest);
    }
    
    #[test]
    fn test_publish_message() {
        // 测试系统推送消息
        let system_msg = PublishRequest::system_push("news_channel", "系统通知内容".as_bytes().to_vec());
        assert_eq!(system_msg.channel_id, "news_channel");
        assert_eq!(system_msg.publisher, Some("system".to_string()));
        assert!(system_msg.message_id.is_some());
        
        let packet = system_msg.create_packet();
        assert_eq!(packet.message_type, MessageType::PublishRequest);
        
        // 测试主题推送消息
        let topic_msg = PublishRequest::topic_push("tech_channel", "rust", "Rust新特性介绍".as_bytes().to_vec());
        assert_eq!(topic_msg.channel_id, "tech_channel");
        assert_eq!(topic_msg.topic, Some("rust".to_string()));
        assert!(topic_msg.message_id.is_some());
        
        // 测试确认消息
        let ack = PublishResponse::success();
        assert_eq!(ack.succeed, true);
        assert_eq!(ack.message, Some("推送消息接收成功".to_string()));
        
        let ack_packet = ack.create_packet();
        assert_eq!(ack_packet.message_type, MessageType::PublishResponse);
    }
    
    #[test]
    fn test_disconnect_ack_message() {
        let disconnect_ack = DisconnectResponse::success();
        assert_eq!(disconnect_ack.succeed, true);
        assert_eq!(disconnect_ack.message, Some("断开连接成功".to_string()));
        
        let packet = disconnect_ack.create_packet();
        assert_eq!(packet.message_type, MessageType::DisconnectResponse);
    }
    
    #[test]
    fn test_recv_batch_ack_message() {
        let batch_ack = RecvBatchResponse::success();
        assert_eq!(batch_ack.succeed, true);
        assert_eq!(batch_ack.message, Some("批量消息接收成功".to_string()));
        
        let packet = batch_ack.create_packet();
        assert_eq!(packet.message_type, MessageType::RecvBatchResponse);
    }
}
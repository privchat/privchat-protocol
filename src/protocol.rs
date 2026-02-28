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
use std::fmt::Debug;

/// 消息基础trait
pub trait Message: Debug + Send + Sync {
    /// 获取消息类型
    fn message_type(&self) -> MessageType;
}

/// 消息类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// 连接请求 - 客户端发起连接
    AuthorizationRequest = 1,
    /// 连接响应 - 服务端连接确认
    AuthorizationResponse = 2,
    /// 断开连接请求 - 客户端主动断开
    DisconnectRequest = 3,
    /// 断开连接响应 - 服务端断开确认
    DisconnectResponse = 4,
    /// 发送消息请求 - 客户端发送消息
    SendMessageRequest = 5,
    /// 发送消息响应 - 服务端发送确认
    SendMessageResponse = 6,
    /// 推送消息请求 - 服务端推送消息
    PushMessageRequest = 7,
    /// 推送消息响应 - 客户端接收确认
    PushMessageResponse = 8,
    /// 批量接收消息请求 - 服务端批量推送消息
    PushBatchRequest = 9,
    /// 批量接收消息响应 - 客户端批量接收确认
    PushBatchResponse = 10,
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
    /// RPC消息请求 - 客户端发送RPC消息
    RpcRequest = 17,
    /// RPC消息响应 - 服务端发送RPC消息
    RpcResponse = 18,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => MessageType::AuthorizationRequest,
            2 => MessageType::AuthorizationResponse,
            3 => MessageType::DisconnectRequest,
            4 => MessageType::DisconnectResponse,
            5 => MessageType::SendMessageRequest,
            6 => MessageType::SendMessageResponse,
            7 => MessageType::PushMessageRequest,
            8 => MessageType::PushMessageResponse,
            9 => MessageType::PushBatchRequest,
            10 => MessageType::PushBatchResponse,
            11 => MessageType::PingRequest,
            12 => MessageType::PongResponse,
            13 => MessageType::SubscribeRequest,
            14 => MessageType::SubscribeResponse,
            15 => MessageType::PublishRequest,
            16 => MessageType::PublishResponse,
            17 => MessageType::RpcRequest,
            18 => MessageType::RpcResponse,
            _ => MessageType::AuthorizationRequest, // 默认值
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
        Self { message_type, body }
    }
}

/// 连接请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
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

impl AuthorizationRequest {
    pub fn new() -> Self {
        Self {
            auth_type: AuthType::JWT,
            auth_token: String::new(),
            client_info: ClientInfo {
                client_type: String::new(),
                version: String::new(),
                os: String::new(),
                os_version: String::new(),
                device_model: None,
                app_package: None,
            },
            device_info: DeviceInfo {
                device_id: String::new(),
                device_type: DeviceType::Unknown,
                app_id: String::new(),
                push_token: None,
                push_channel: None,
                device_name: String::new(),
                device_model: None,
                os_version: None,
                app_version: None,
                manufacturer: None,
                device_fingerprint: None,
            },
            protocol_version: crate::version::VERSION.to_string(),
            properties: HashMap::new(),
        }
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::AuthorizationRequest, self)
    }
}

/// 连接响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    /// 连接是否成功
    pub success: bool,
    /// 错误码
    pub error_code: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 用户ID
    pub user_id: Option<u64>,
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
    pub device_id: String,
    pub device_type: DeviceType,
    pub app_id: String,
    pub push_token: Option<String>,
    pub push_channel: Option<String>,
    pub device_name: String,
    pub device_model: Option<String>,
    pub os_version: Option<String>,
    pub app_version: Option<String>,
    pub manufacturer: Option<String>,
    pub device_fingerprint: Option<String>,
}

/// 设备类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(non_camel_case_types)]
pub enum DeviceType {
    #[serde(rename = "ios")]
    iOS,
    #[serde(rename = "android")]
    Android,
    #[serde(rename = "web")]
    Web,
    #[serde(rename = "macos")]
    MacOS,
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "iot")]
    IoT,
    #[serde(rename = "unknown")]
    Unknown,
}

impl DeviceType {
    pub fn as_str(&self) -> &str {
        match self {
            DeviceType::iOS => "ios",
            DeviceType::Android => "android",
            DeviceType::Web => "web",
            DeviceType::MacOS => "macos",
            DeviceType::Windows => "windows",
            DeviceType::Linux => "linux",
            DeviceType::IoT => "iot",
            DeviceType::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ios" => DeviceType::iOS,
            "android" => DeviceType::Android,
            "web" => DeviceType::Web,
            "macos" => DeviceType::MacOS,
            "windows" => DeviceType::Windows,
            "linux" | "freebsd" | "unix" => DeviceType::Linux,
            "iot" => DeviceType::IoT,
            _ => DeviceType::Unknown,
        }
    }
}

/// 服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub name: String,
    pub features: Vec<String>,
    pub max_message_size: u64,
    pub connection_timeout: u64,
}

/// 断开连接请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectRequest {
    pub reason: DisconnectReason,
    pub message: Option<String>,
}

/// 断开连接响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectResponse {
    pub acknowledged: bool,
}

/// 断开连接原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectReason {
    UserInitiated,
    ServerShutdown,
    AuthenticationFailed,
    ProtocolError,
    Timeout,
    DuplicateConnection,
    ServerMaintenance,
}

/// 通用消息包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePacket {
    pub message_type: String,
    pub server_message_id: Option<u64>,
    pub timestamp: u64,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
}

/// 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub error_message: String,
    pub error_details: Option<HashMap<String, String>>,
    pub timestamp: u64,
}

/// 发送消息请求
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub setting: MessageSetting,
    pub client_seq: u32,
    pub local_message_id: u64,
    pub stream_no: String,
    pub channel_id: u64,
    pub message_type: u32,
    pub expire: u32,
    pub from_uid: u64,
    pub topic: String,
    pub payload: Vec<u8>,
}

impl SendMessageRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SendMessageRequest, self)
    }

    pub fn verify_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.local_message_id, self.channel_id, self.from_uid
        )
    }
}

/// 发送消息响应
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub client_seq: u32,
    pub server_message_id: u64,
    pub message_seq: u32,
    pub reason_code: u32,
}

impl SendMessageResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SendMessageResponse, self)
    }
}

/// 推送消息请求（服务端推送给客户端）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushMessageRequest {
    pub setting: MessageSetting,
    pub msg_key: String,
    pub server_message_id: u64,
    pub message_seq: u32,
    pub local_message_id: u64,
    pub stream_no: String,
    pub stream_seq: u32,
    pub stream_flag: u8,
    pub timestamp: u32,
    pub channel_id: u64,
    pub channel_type: u8,
    pub message_type: u32,
    pub expire: u32,
    pub topic: String,
    pub from_uid: u64,
    pub payload: Vec<u8>,
}

impl PushMessageRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushMessageRequest, self)
    }

    pub fn verify_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.server_message_id, self.channel_id, self.from_uid
        )
    }
}

/// 推送消息响应（客户端确认接收）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushMessageResponse {
    pub succeed: bool,
    pub message: Option<String>,
}

impl PushMessageResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushMessageResponse, self)
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

/// 订阅消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub setting: u8,
    pub local_message_id: u64,
    pub channel_id: u64,
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
    pub local_message_id: u64,
    pub channel_id: u64,
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

/// 批量接收消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushBatchRequest {
    pub messages: Vec<PushMessageRequest>,
}

impl PushBatchRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushBatchRequest, self)
    }

    pub fn single_batch(messages: Vec<PushMessageRequest>) -> Self {
        Self { messages }
    }

    pub fn multi_batch(messages: Vec<PushMessageRequest>) -> Self {
        Self { messages }
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// 批量接收确认消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushBatchResponse {
    pub succeed: bool,
    pub message: Option<String>,
}

impl PushBatchResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushBatchResponse, self)
    }

    pub fn success() -> Self {
        Self {
            succeed: true,
            message: Some("批量消息接收成功".to_string()),
        }
    }

    pub fn failure(error_msg: &str) -> Self {
        Self {
            succeed: false,
            message: Some(error_msg.to_string()),
        }
    }
}

/// 频道推送消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishRequest {
    pub channel_id: u64,
    pub topic: Option<String>,
    pub timestamp: u64,
    pub payload: Vec<u8>,
    pub publisher: Option<String>,
    pub server_message_id: Option<u64>,
}

impl PublishRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PublishRequest, self)
    }

    pub fn system_push(channel_id: u64, payload: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let server_message_id = timestamp;
        Self {
            channel_id,
            topic: None,
            timestamp,
            payload,
            publisher: Some("system".to_string()),
            server_message_id: Some(server_message_id),
        }
    }

    pub fn topic_push(channel_id: u64, topic: &str, payload: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let server_message_id = timestamp;
        Self {
            channel_id,
            topic: Some(topic.to_string()),
            timestamp,
            payload,
            publisher: None,
            server_message_id: Some(server_message_id),
        }
    }
}

/// 推送确认消息
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

    pub fn success() -> Self {
        Self {
            succeed: true,
            message: Some("推送消息接收成功".to_string()),
        }
    }

    pub fn failure(error_msg: &str) -> Self {
        Self {
            succeed: false,
            message: Some(error_msg.to_string()),
        }
    }
}

/// RPC 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub route: String,
    pub body: serde_json::Value,
}

impl RpcRequest {
    pub fn new() -> Self {
        Self {
            route: String::new(),
            body: serde_json::Value::Null,
        }
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::RpcRequest, self)
    }
}

/// RPC 响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl RpcResponse {
    pub fn new() -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: None,
        }
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::RpcResponse, self)
    }

    pub fn success(data: serde_json::Value) -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: Some(data),
        }
    }

    pub fn success_empty() -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: None,
        }
    }

    pub fn error(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    #[inline]
    pub fn is_ok(&self) -> bool {
        self.code == 0
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        self.code != 0
    }
}

impl Message for AuthorizationRequest {
    fn message_type(&self) -> MessageType {
        MessageType::AuthorizationRequest
    }
}

impl Message for AuthorizationResponse {
    fn message_type(&self) -> MessageType {
        MessageType::AuthorizationResponse
    }
}

impl Message for SendMessageRequest {
    fn message_type(&self) -> MessageType {
        MessageType::SendMessageRequest
    }
}

impl Message for SendMessageResponse {
    fn message_type(&self) -> MessageType {
        MessageType::SendMessageResponse
    }
}

impl Message for PushMessageRequest {
    fn message_type(&self) -> MessageType {
        MessageType::PushMessageRequest
    }
}

impl Message for PushMessageResponse {
    fn message_type(&self) -> MessageType {
        MessageType::PushMessageResponse
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

impl Message for PushBatchRequest {
    fn message_type(&self) -> MessageType {
        MessageType::PushBatchRequest
    }
}

impl Message for PushBatchResponse {
    fn message_type(&self) -> MessageType {
        MessageType::PushBatchResponse
    }
}

impl DisconnectResponse {
    pub fn new() -> Self {
        Self {
            acknowledged: false,
        }
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::DisconnectResponse, self)
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

impl Message for RpcRequest {
    fn message_type(&self) -> MessageType {
        MessageType::RpcRequest
    }
}

impl Message for RpcResponse {
    fn message_type(&self) -> MessageType {
        MessageType::RpcResponse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let connect_msg = AuthorizationRequest::new();
        let packet = connect_msg.create_packet();
        assert_eq!(packet.message_type, MessageType::AuthorizationRequest);

        let send_msg = SendMessageRequest::new();
        let packet = send_msg.create_packet();
        assert_eq!(packet.message_type, MessageType::SendMessageRequest);
    }

    #[test]
    fn test_message_types() {
        assert_eq!(MessageType::AuthorizationRequest as u8, 1);
        assert_eq!(MessageType::SendMessageRequest as u8, 5);
        assert_eq!(MessageType::PushMessageRequest as u8, 7);
    }

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(MessageType::from(1u8), MessageType::AuthorizationRequest);
        assert_eq!(u8::from(MessageType::AuthorizationRequest), 1);
    }

    #[test]
    fn test_message_setting() {
        let setting = MessageSetting::new();
        assert_eq!(setting.need_receipt, false);
    }

    #[test]
    fn test_batch_message() {
        let mut messages = Vec::new();
        for i in 1..=3 {
            let mut recv_msg = PushMessageRequest::new();
            recv_msg.server_message_id = i as u64;
            recv_msg.from_uid = i as u64;
            recv_msg.channel_id = 1;
            recv_msg.payload = format!("Message {}", i).into_bytes();
            messages.push(recv_msg);
        }
        let batch_msg = PushBatchRequest::single_batch(messages);
        assert_eq!(batch_msg.message_count(), 3);
    }

    #[test]
    fn test_publish_message() {
        let system_msg = PublishRequest::system_push(12345, "系统通知内容".as_bytes().to_vec());
        assert_eq!(system_msg.channel_id, 12345);
    }

    #[test]
    fn test_disconnect_ack_message() {
        let disconnect_ack = DisconnectResponse { acknowledged: true };
        assert_eq!(disconnect_ack.acknowledged, true);
    }

    #[test]
    fn test_recv_batch_ack_message() {
        let batch_ack = PushBatchResponse::success();
        assert_eq!(batch_ack.succeed, true);
    }
}

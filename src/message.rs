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
        Self {
            message_type,
            body,
        }
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
            protocol_version: "1.0".to_string(),
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
/// 
/// 用于设备管理、安全审计和推送通知
/// 
/// 推送通知说明：
/// - iOS: 使用 APNs
/// - Android 海外: 使用 FCM (Firebase Cloud Messaging)
/// - 华为: 使用 HMS Push
/// - 小米: 使用 MiPush
/// - OPPO: 使用 OPPO Push
/// - vivo: 使用 vivo Push
/// - 魅族: 使用 Flyme Push
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备ID（**必需**）
    /// 
    /// 每个设备应有唯一的 device_id，用于：
    /// 1. JWT token 绑定和验证
    /// 2. 设备管理（查看/踢出设备）
    /// 3. 推送通知路由
    pub device_id: String,
    
    /// 设备类型（**必需**）
    pub device_type: DeviceType,
    
    /// 应用ID（**必需**）
    /// 
    /// 用于 JWT token 验证，标识具体的应用
    /// 例如: "ios", "android", "web", "macos", "windows", "linux"
    /// 
    /// 注意：同一个 device_type 可能对应多个不同的应用
    pub app_id: String,
    
    /// 推送 Token（**强烈建议提供**）
    /// 
    /// - iOS: APNs Device Token (64字符十六进制)
    /// - Android: FCM Token / HMS Token / MiPush Token 等
    /// - Web: Web Push Subscription (JSON)
    /// 
    /// 用于接收推送通知，在后台时唤醒应用
    pub push_token: Option<String>,
    
    /// 推送通道类型（Android 必需）
    /// 
    /// 值: "apns", "fcm", "hms", "mipush", "oppo", "vivo", "meizu"
    /// 
    /// Android 需要指定具体厂商通道：
    /// - 华为设备: "hms"
    /// - 小米设备: "mipush"
    /// - OPPO设备: "oppo"
    /// - vivo设备: "vivo"
    /// - 魅族设备: "meizu"
    /// - 其他/海外: "fcm"
    pub push_channel: Option<String>,
    
    /// 设备名称（可选）
    /// 
    /// 用于设备管理界面显示，例如：
    /// - "我的 iPhone"
    /// - "Chrome 浏览器"
    /// - "办公室电脑"
    pub device_name: String,
    
    /// 设备型号（可选）
    /// 
    /// 例如: "iPhone 15 Pro", "Samsung Galaxy S23", "Xiaomi 14"
    pub device_model: Option<String>,
    
    /// 操作系统版本（可选）
    /// 
    /// 例如: "iOS 17.2", "Android 14", "Windows 11"
    pub os_version: Option<String>,
    
    /// 应用版本（可选）
    /// 
    /// 例如: "1.0.0", "2.3.1"
    pub app_version: Option<String>,
    
    /// 厂商（Android 必需）
    /// 
    /// 值: "huawei", "xiaomi", "oppo", "vivo", "meizu", "samsung", "google" 等
    /// 
    /// 用于选择正确的推送通道
    pub manufacturer: Option<String>,
    
    /// 设备指纹（可选）
    /// 
    /// 用于设备唯一性识别和安全审计
    pub device_fingerprint: Option<String>,
}

/// 设备类型
/// 
/// 用于确定推送通道和设备管理
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(non_camel_case_types)]
pub enum DeviceType {
    /// iOS 设备 (iPhone/iPad)
    /// 推送: APNs
    #[serde(rename = "ios")]
    iOS,
    
    /// Android 设备 (手机/平板)
    /// 推送: FCM/HMS/MiPush/OPPO/vivo/Meizu (根据厂商)
    #[serde(rename = "android")]
    Android,
    
    /// Web 浏览器
    /// 推送: Web Push API
    #[serde(rename = "web")]
    Web,
    
    /// macOS 桌面应用
    /// 推送: APNs (macOS)
    #[serde(rename = "macos")]
    MacOS,
    
    /// Windows 桌面应用
    /// 推送: WNS (Windows Notification Service)
    #[serde(rename = "windows")]
    Windows,
    
    /// Linux/Unix 桌面应用 (包括 FreeBSD 等)
    /// 推送: 通常不支持系统级推送
    #[serde(rename = "linux")]
    Linux,
    
    /// IoT 设备 (智能音箱、智能硬件、手表等)
    #[serde(rename = "iot")]
    IoT,
    
    /// 未知设备类型
    #[serde(rename = "unknown")]
    Unknown,
}

impl DeviceType {
    /// 转换为字符串
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
    
    /// 从字符串解析
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
    /// 服务端消息ID
    pub server_message_id: Option<u64>,
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

/// 发送消息请求
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub setting: MessageSetting,
    pub client_seq: u32,
    pub local_message_id: u64,
    pub stream_no: String,
    pub channel_id: u64,
    pub channel_type: u8,
    pub expire: u32,
    pub from_uid: u64,
    pub topic: String,
    pub payload: Vec<u8>,  // JSON 格式，包含 message_type, content, metadata
}

impl SendMessageRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::SendMessageRequest, self)
    }
    
    pub fn verify_string(&self) -> String {
        format!("{}:{}:{}", self.local_message_id, self.channel_id, self.from_uid)
    }
}

/// 发送消息响应
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub client_seq: u32,
    pub server_message_id: u64,
    pub message_seq: u32,
    pub reason_code: u8,
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
        format!("{}:{}:{}", self.server_message_id, self.channel_id, self.from_uid)
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

// DisconnectRequest 已在上面定义，删除重复定义

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

/// 批量接收消息 - 用于服务器向客户端批量推送消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushBatchRequest {
    /// 批量消息列表
    pub messages: Vec<PushMessageRequest>
}

impl PushBatchRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PushBatchRequest, self)
    }
    
    /// 创建单个批次的批量消息
    pub fn single_batch(messages: Vec<PushMessageRequest>) -> Self {
        Self {
            messages,
        }
    }
    
    /// 创建多批次中的一个批次
    pub fn multi_batch(
        messages: Vec<PushMessageRequest>,
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
    pub channel_id: u64,          // 频道ID
    pub topic: Option<String>,     // 主题/标签（可选）
    pub timestamp: u64,            // 推送时间戳
    pub payload: Vec<u8>,          // 消息内容
    pub publisher: Option<String>, // 发布者（可选，可能是系统/机器人）
    pub server_message_id: Option<u64>,   // 服务端消息ID（用于去重）
}

impl PublishRequest {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::PublishRequest, self)
    }
    
    /// 创建系统推送消息
    pub fn system_push(channel_id: u64, payload: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // 使用 Snowflake 或时间戳生成 server_message_id
        let server_message_id = timestamp; // 简化处理，实际应该使用 Snowflake
        
        Self {
            channel_id,
            topic: None,
            timestamp,
            payload,
            publisher: Some("system".to_string()),
            server_message_id: Some(server_message_id),
        }
    }
    
    /// 创建主题推送消息
    pub fn topic_push(channel_id: u64, topic: &str, payload: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // 使用 Snowflake 或时间戳生成 server_message_id
        let server_message_id = timestamp; // 简化处理，实际应该使用 Snowflake
        
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

/// RPC 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// 路由路径，格式：system/module/action
    pub route: String,
    /// 请求参数 JSON
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
    /// 状态码
    pub code: i32,
    /// 响应消息
    pub message: String,
    /// 响应数据
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

    /// 创建成功响应
    /// 
    /// RPC 层不是 HTTP，code == 0 永远表示 success
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: Some(data),
        }
    }

    /// 创建成功响应（无数据）
    /// 
    /// RPC 层不是 HTTP，code == 0 永远表示 success
    pub fn success_empty() -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: None,
        }
    }

    /// 创建错误响应
    /// 
    /// code != 0 表示错误，常见的错误码：
    /// - 400: ValidationError
    /// - 401: Unauthorized
    /// - 403: Forbidden
    /// - 404: NotFound
    /// - 500: InternalError
    pub fn error(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    /// 检查响应是否成功
    /// 
    /// # 返回
    /// - `true`: code == 0，表示成功
    /// - `false`: code != 0，表示错误
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.code == 0
    }

    /// 检查响应是否失败
    /// 
    /// # 返回
    /// - `true`: code != 0，表示错误
    /// - `false`: code == 0，表示成功
    #[inline]
    pub fn is_err(&self) -> bool {
        self.code != 0
    }
}

// 为所有消息类型实现 Message trait
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
        assert_eq!(MessageType::AuthorizationResponse as u8, 2);
        assert_eq!(MessageType::DisconnectRequest as u8, 3);
        assert_eq!(MessageType::DisconnectResponse as u8, 4);
        assert_eq!(MessageType::SendMessageRequest as u8, 5);
        assert_eq!(MessageType::SendMessageResponse as u8, 6);
        assert_eq!(MessageType::PushMessageRequest as u8, 7);
        assert_eq!(MessageType::PushMessageResponse as u8, 8);
        assert_eq!(MessageType::PushBatchRequest as u8, 9);        // 批量接收消息
        assert_eq!(MessageType::PushBatchResponse as u8, 10);     // 批量接收确认
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
        assert_eq!(MessageType::from(1u8), MessageType::AuthorizationRequest);
        assert_eq!(MessageType::from(2u8), MessageType::AuthorizationResponse);
        assert_eq!(MessageType::from(3u8), MessageType::DisconnectRequest);
        assert_eq!(MessageType::from(16u8), MessageType::PublishResponse);
        
        // 测试无效值的转换（应该返回默认值）
        assert_eq!(MessageType::from(0u8), MessageType::AuthorizationRequest);
        assert_eq!(MessageType::from(255u8), MessageType::AuthorizationRequest);
        
        // 测试 MessageType 到 u8 的转换
        assert_eq!(u8::from(MessageType::AuthorizationRequest), 1);
        assert_eq!(u8::from(MessageType::AuthorizationResponse), 2);
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
            let mut recv_msg = PushMessageRequest::new();
            recv_msg.server_message_id = i as u64;
            recv_msg.from_uid = i as u64;
            recv_msg.channel_id = 1;
            recv_msg.payload = format!("Message {}", i).into_bytes();
            messages.push(recv_msg);
        }
        
        let batch_msg = PushBatchRequest::single_batch(messages);
        assert_eq!(batch_msg.message_count(), 3);
        
        let packet = batch_msg.create_packet();
        assert_eq!(packet.message_type, MessageType::PushBatchRequest);
    }
    
    #[test]
    fn test_publish_message() {
        // 测试系统推送消息
        let system_msg = PublishRequest::system_push(12345, "系统通知内容".as_bytes().to_vec());
        assert_eq!(system_msg.channel_id, 12345);
        assert_eq!(system_msg.publisher, Some("system".to_string()));
        assert!(system_msg.server_message_id.is_some());
        
        let packet = system_msg.create_packet();
        assert_eq!(packet.message_type, MessageType::PublishRequest);
        
        // 测试主题推送消息
        let topic_msg = PublishRequest::topic_push(67890, "rust", "Rust新特性介绍".as_bytes().to_vec());
        assert_eq!(topic_msg.channel_id, 67890);
        assert_eq!(topic_msg.topic, Some("rust".to_string()));
        assert!(topic_msg.server_message_id.is_some());
        
        // 测试确认消息
        let ack = PublishResponse::success();
        assert_eq!(ack.succeed, true);
        assert_eq!(ack.message, Some("推送消息接收成功".to_string()));
        
        let ack_packet = ack.create_packet();
        assert_eq!(ack_packet.message_type, MessageType::PublishResponse);
    }
    
    #[test]
    fn test_disconnect_ack_message() {
        let disconnect_ack = DisconnectResponse {
            acknowledged: true,
        };
        assert_eq!(disconnect_ack.acknowledged, true);
        
        let packet = disconnect_ack.create_packet();
        assert_eq!(packet.message_type, MessageType::DisconnectResponse);
    }
    
    #[test]
    fn test_recv_batch_ack_message() {
        let batch_ack = PushBatchResponse::success();
        assert_eq!(batch_ack.succeed, true);
        assert_eq!(batch_ack.message, Some("批量消息接收成功".to_string()));
        
        let packet = batch_ack.create_packet();
        assert_eq!(packet.message_type, MessageType::PushBatchResponse);
    }
}
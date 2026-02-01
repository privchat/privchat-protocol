# PrivChat Protocol

一个用于即时通讯（IM）应用的 Rust 协议库，提供完整的消息定义和类型系统。

## 📋 项目简介

PrivChat Protocol 是一个专为即时通讯应用设计的协议库，支持完整的 IM 功能，包括连接管理、消息收发、心跳检测、频道订阅、批量消息推送等。该库采用 Request/Response 消息模式，提供类型安全的消息处理机制。

## ✨ 主要特性

- 🔗 **完整的连接生命周期管理** - 支持连接建立、维持和断开
- 💬 **实时消息通信** - 支持单向和双向消息传输
- 📦 **批量消息处理** - 支持批量消息推送和确认
- 🔔 **频道订阅系统** - 支持发布/订阅模式的消息分发
- 💓 **心跳检测机制** - 保持连接活跃状态
- 🎯 **类型安全** - 基于 Rust 的强类型系统
- 📋 **统一消息接口** - 提供一致的消息处理 API

## 🗂️ 消息类型

本协议支持 16 种消息类型，按功能分为以下几类：

### 连接管理 (1-4)
| biz_type | 消息类型 | 描述 | 通信方向 |
|----------|----------|------|----------|
| 1 | `AuthorizationRequest` | 连接请求 | 客户端 → 服务端 |
| 2 | `AuthorizationResponse` | 连接响应 | 服务端 → 客户端 |
| 3 | `DisconnectRequest` | 断开连接请求 | 客户端 → 服务端 |
| 4 | `DisconnectResponse` | 断开连接响应 | 服务端 → 客户端 |

### 消息收发 (5-10)
| biz_type | 消息类型 | 描述 | 通信方向 |
|----------|----------|------|----------|
| 5 | `SendRequest` | 发送消息请求 | 客户端 → 服务端 |
| 6 | `SendResponse` | 发送消息响应 | 服务端 → 客户端 |
| 7 | `RecvRequest` | 接收消息请求 | 服务端 → 客户端 |
| 8 | `RecvResponse` | 接收消息响应 | 客户端 → 服务端 |
| 9 | `PushBatchRequest` | 批量接收消息请求 | 服务端 → 客户端 |
| 10 | `PushBatchResponse` | 批量接收消息响应 | 客户端 → 服务端 |

### 心跳检测 (11-12)
| biz_type | 消息类型 | 描述 | 通信方向 |
|----------|----------|------|----------|
| 11 | `PingRequest` | 心跳请求 | 客户端 → 服务端 |
| 12 | `PongResponse` | 心跳响应 | 服务端 → 客户端 |

### 频道系统 (13-16)
| biz_type | 消息类型 | 描述 | 通信方向 |
|----------|----------|------|----------|
| 13 | `SubscribeRequest` | 订阅请求 | 客户端 → 服务端 |
| 14 | `SubscribeResponse` | 订阅响应 | 服务端 → 客户端 |
| 15 | `PublishRequest` | 推送消息请求 | 服务端 → 客户端 |
| 16 | `PublishResponse` | 推送消息响应 | 客户端 → 服务端 |

## 🚀 快速开始

### 添加依赖

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
privchat-protocol = { path = "../privchat-protocol" }
num-bigint = "0.4"
```

### 基本使用

```rust
use privchat_protocol::message::*;
use num_bigint::BigInt;

fn main() {
    // 创建连接请求
    let connect_req = AuthorizationRequest {
        version: 1,
        device_id: "device_123".to_string(),
        uid: "user_123".to_string(),
        token: "secret_token".to_string(),
        ..Default::default()
    };

    // 创建数据包
    let packet = connect_req.create_packet();
    println!("消息类型: {:?}", packet.message_type);

    // 创建发送消息
    let send_req = SendRequest {
        from_uid: "user_123".to_string(),
        channel_id: "channel_456".to_string(),
        payload: "Hello, World!".as_bytes().to_vec(),
        ..Default::default()
    };

    let send_packet = send_req.create_packet();
    println!("发送消息: {:?}", send_packet.message_type);
}
```

### 批量消息处理

```rust
use privchat_protocol::message::*;

// 创建批量消息
let mut messages = Vec::new();
for i in 1..=5 {
    let mut recv_msg = RecvRequest::new();
    recv_msg.server_message_id = BigInt::from(i);
    recv_msg.from_uid = format!("user_{}", i);
    recv_msg.payload = format!("Message {}", i).into_bytes();
    messages.push(recv_msg);
}

let batch_req = PushBatchRequest::single_batch(messages);
println!("批量消息数量: {}", batch_req.message_count());

// 创建批量确认
let batch_resp = PushBatchResponse::success();
```

### 频道推送

```rust
use privchat_protocol::message::*;

// 系统推送
let system_push = PublishRequest::system_push(
    "news_channel", 
    "系统维护通知".as_bytes().to_vec()
);

// 主题推送
let topic_push = PublishRequest::topic_push(
    "tech_channel", 
    "rust", 
    "Rust 1.75.0 发布".as_bytes().to_vec()
);

// 推送确认
let push_ack = PublishResponse::success();
```

## 🔧 消息结构

### 核心类型

- **`MessageType`** - 消息类型枚举（1-16）
- **`Message`** - 消息基础 trait
- **`Packet<T>`** - 消息包装结构
- **`MessageSetting`** - 消息设置

### 消息设置

```rust
pub struct MessageSetting {
    pub need_receipt: bool,  // 是否需要回执
    pub signal: u8,          // 信号标识
}
```

### 数据包结构

```rust
pub struct Packet<T: Message> {
    pub message_type: MessageType,
    pub body: T,
}
```

## 📚 API 文档

### 消息创建

所有消息类型都提供以下方法：

- `new()` - 创建默认实例
- `create_packet()` - 创建包装后的数据包
- `message_type()` - 获取消息类型

### 类型转换

`MessageType` 支持与 `u8` 类型的双向转换：

```rust
// u8 到 MessageType
let msg_type = MessageType::from(5u8);  // SendRequest
assert_eq!(msg_type, MessageType::SendRequest);

// MessageType 到 u8
let biz_type = u8::from(MessageType::SendRequest);
assert_eq!(biz_type, 5);

// 直接转换
let biz_type = MessageType::SendRequest as u8;
assert_eq!(biz_type, 5);
```

**注意**：无效的 u8 值（0 或 >16）会转换为 `AuthorizationRequest`（默认值）。

### 便捷方法

部分消息类型提供便捷创建方法：

**PublishRequest:**
- `system_push(channel_id, payload)` - 创建系统推送
- `topic_push(channel_id, topic, payload)` - 创建主题推送

**响应消息:**
- `success()` - 创建成功响应
- `failure(error_msg)` - 创建失败响应

## 🏗️ 构建和测试

### 构建项目

```bash
cargo build
```

### 运行测试

```bash
cargo test
```

### 运行示例

```bash
cargo run --example basic_usage
```

## 📖 示例程序

项目包含完整的示例程序 `examples/basic_usage.rs`，展示了所有 16 种消息类型的使用方法。

运行示例查看输出：

```bash
cargo run --example basic_usage
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request 来改进这个项目。

## 📄 许可证

本项目采用 Apache-2.0 许可证，详见 [LICENSE](LICENSE) 文件。

## 🔗 相关项目

- [msgtrans](https://github.com/zoujiaqing/msgtrans) - 统一传输层框架

---

📧 如有问题，请提交 Issue 或联系维护者。

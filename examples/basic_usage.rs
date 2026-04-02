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

use privchat_protocol::protocol::*;
use std::collections::HashMap;

fn main() {
    println!("=== Privchat Protocol 消息示例 ===\n");

    // 1. 连接请求/响应
    let connect_req = AuthorizationRequest {
        auth_type: AuthType::JWT,
        auth_token: "secret_token".to_string(),
        client_info: ClientInfo {
            client_type: "android".to_string(),
            version: "1.0.0".to_string(),
            os: "android".to_string(),
            os_version: "14".to_string(),
            device_model: Some("Pixel".to_string()),
            app_package: Some("dev.privchat.app".to_string()),
        },
        device_info: DeviceInfo {
            device_id: "device_123".to_string(),
            device_type: DeviceType::Android,
            app_id: "privchat-app".to_string(),
            push_token: None,
            push_channel: None,
            device_name: "Pixel".to_string(),
            device_model: Some("Pixel 8".to_string()),
            os_version: Some("14".to_string()),
            app_version: Some("1.0.0".to_string()),
            manufacturer: Some("Google".to_string()),
            device_fingerprint: None,
        },
        protocol_version: "1.0".to_string(),
        properties: HashMap::new(),
    };

    let connect_resp = AuthorizationResponse {
        success: true,
        error_code: None,
        error_message: None,
        session_id: Some("session_1".to_string()),
        user_id: Some(123),
        connection_id: Some("conn_1".to_string()),
        server_info: Some(ServerInfo {
            version: "0.1.0".to_string(),
            name: "privchat-server".to_string(),
            features: vec!["rpc".to_string(), "push".to_string()],
            max_message_size: 1024 * 1024,
            connection_timeout: 30,
        }),
        heartbeat_interval: Some(30),
    };

    println!("连接请求: {:?}", connect_req);
    println!("连接响应: {:?}", connect_resp);
    println!();

    // 2. 发送消息请求/响应
    let send_req = SendMessageRequest {
        setting: MessageSetting {
            need_receipt: true,
            signal: 0,
        },
        client_seq: 1,
        local_message_id: 0,
        stream_no: "stream_001".to_string(),
        channel_id: 456,
        message_type: 0, // ContentMessageType::Text
        expire: 3600,
        from_uid: 123,
        topic: "chat".to_string(),
        payload: "Hello, World!".as_bytes().to_vec(),
    };

    let send_resp = SendMessageResponse {
        client_seq: 1,
        server_message_id: 999,
        message_seq: 1,
        reason_code: 0,
    };

    println!("发送请求: {:?}", send_req);
    println!("发送响应: {:?}", send_resp);
    println!();

    // 3. 接收消息请求/响应
    let recv_req = PushMessageRequest {
        setting: MessageSetting {
            need_receipt: true,
            signal: 0,
        },
        msg_key: "msg_key_001".to_string(),
        server_message_id: 999,
        message_seq: 1,
        local_message_id: 0,
        stream_no: "stream_001".to_string(),
        stream_seq: 1,
        stream_flag: 0,
        timestamp: 1234567890,
        channel_id: 789,
        channel_type: 1,
        message_type: 0, // ContentMessageType::Text
        expire: 3600,
        topic: "chat".to_string(),
        from_uid: 456,
        payload: "Hello back!".as_bytes().to_vec(),
    };

    let recv_resp = PushMessageResponse {
        succeed: true,
        message: Some("消息接收成功".to_string()),
    };

    println!("接收请求: {:?}", recv_req);
    println!("接收响应: {:?}", recv_resp);
    println!();

    // 4. 心跳请求/响应
    let ping_req = PingRequest {
        timestamp: 1234567890,
    };

    let pong_resp = PongResponse {
        timestamp: 1234567890,
    };

    println!("心跳请求: {:?}", ping_req);
    println!("心跳响应: {:?}", pong_resp);
    println!();

    // 5. 断开连接请求/响应
    let disconnect_req = DisconnectRequest {
        reason: DisconnectReason::UserInitiated,
        message: Some("用户主动断开".to_string()),
    };

    let disconnect_resp = DisconnectResponse { acknowledged: true };

    println!("断开连接请求: {:?}", disconnect_req);
    println!("断开连接响应: {:?}", disconnect_resp);
    println!();

    // 6. 订阅请求/响应
    let subscribe_req = SubscribeRequest {
        setting: 1,
        local_message_id: 1001,
        channel_id: 2001,
        channel_type: 2,
        action: 1,
        param: "all".to_string(),
    };

    let subscribe_resp = SubscribeResponse {
        local_message_id: 1001,
        channel_id: 2001,
        channel_type: 2,
        action: 1,
        reason_code: 0,
    };

    println!("订阅请求: {:?}", subscribe_req);
    println!("订阅响应: {:?}", subscribe_resp);
    println!();

    // 7. 批量接收请求/响应
    let mut batch_messages = Vec::new();
    for i in 1..=3 {
        let mut msg = PushMessageRequest::new();
        msg.server_message_id = i;
        msg.from_uid = i;
        msg.payload = format!("批量消息 {}", i).as_bytes().to_vec();
        batch_messages.push(msg);
    }

    let batch_req = PushBatchRequest::single_batch(batch_messages);
    let batch_resp = PushBatchResponse::success();

    println!("批量接收请求: {:?}", batch_req);
    println!("批量接收响应: {:?}", batch_resp);
    println!();

    // 8. 推送请求/响应
    let publish_req = PublishRequest::system_push(2001, "系统通知消息".as_bytes().to_vec());
    let publish_resp = PublishResponse::success();

    println!("推送请求: {:?}", publish_req);
    println!("推送响应: {:?}", publish_resp);
    println!();

    // 9. 创建数据包示例
    let packet = connect_req.create_packet();
    println!("连接请求数据包: {:?}", packet);

    // 10. 消息类型验证
    println!("=== 消息类型验证 ===");
    for i in 1u8..=16u8 {
        let msg_type = MessageType::from(i);
        let description = match i {
            1 => "连接请求",
            2 => "连接响应",
            3 => "断开连接请求",
            4 => "断开连接响应",
            5 => "发送消息请求",
            6 => "发送消息响应",
            7 => "接收消息请求",
            8 => "接收消息响应",
            9 => "批量接收消息请求",
            10 => "批量接收消息响应",
            11 => "心跳请求",
            12 => "心跳响应",
            13 => "订阅请求",
            14 => "订阅响应",
            15 => "推送消息请求",
            16 => "推送消息响应",
            _ => "未知类型",
        };
        println!("类型 {}: {:?} - {}", i, msg_type, description);
    }

    println!("\n=== 示例完成 ===");
}

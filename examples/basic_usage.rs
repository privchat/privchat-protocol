use privchat_protocol::protocol::*;
use num_bigint::BigInt;

fn main() {
    println!("=== Privchat Protocol 消息示例 ===\n");

    // 1. 连接请求/响应
    let connect_req = AuthorizationRequest {
        version: 1,
        device_id: "device_123".to_string(),
        device_flag: 1,
        client_timestamp: 1234567890,
        uid: "user_123".to_string(),
        token: "secret_token".to_string(),
        client_key: "client_public_key".to_string(),
    };
    
    let connect_resp = AuthorizationResponse {
        protocol_version: 1,
        server_key: "server_public_key".to_string(),
        salt: "random_salt".to_string(),
        time_diff: BigInt::from(100),
        reason_code: 0,
        node_id: BigInt::from(1),
    };
    
    println!("连接请求: {:?}", connect_req);
    println!("连接响应: {:?}", connect_resp);
    println!();

    // 2. 发送消息请求/响应
    let send_req = SendMessageRequest {
        setting: MessageSetting { need_receipt: true, signal: 0 },
        client_seq: 1,
        local_message_id: 0,
        stream_no: "stream_001".to_string(),
        channel_id: 456,
        channel_type: 1,
        message_type: 0, // ContentMessageType::Text
        expire: 3600,
        from_uid: 123,
        topic: "chat".to_string(),
        payload: "Hello, World!".as_bytes().to_vec(),
    };
    
    let send_resp = SendMessageResponse {
        client_seq: 1,
        server_message_id: BigInt::from(999),
        message_seq: 1,
        reason_code: 0,
    };
    
    println!("发送请求: {:?}", send_req);
    println!("发送响应: {:?}", send_resp);
    println!();

    // 3. 接收消息请求/响应
    let recv_req = PushMessageRequest {
        setting: MessageSetting { need_receipt: true, signal: 0 },
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
        reason_code: 0,
        reason: "用户主动断开".to_string(),
    };
    
    let disconnect_resp = DisconnectResponse::success();
    
    println!("断开连接请求: {:?}", disconnect_req);
    println!("断开连接响应: {:?}", disconnect_resp);
    println!();

    // 6. 订阅请求/响应
    let subscribe_req = SubscribeRequest {
        setting: 1,
        local_message_id: "sub_001".to_string(),
        channel_id: "news_channel".to_string(),
        channel_type: 2,
        action: 1,
        param: "all".to_string(),
    };
    
    let subscribe_resp = SubscribeResponse {
        local_message_id: "sub_001".to_string(),
        channel_id: "news_channel".to_string(),
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
        msg.server_message_id = BigInt::from(i);
        msg.from_uid = format!("user_{}", i);
        msg.payload = format!("批量消息 {}", i).as_bytes().to_vec();
        batch_messages.push(msg);
    }
    
    let batch_req = PushBatchRequest::single_batch(batch_messages);
    let batch_resp = PushBatchResponse::success();
    
    println!("批量接收请求: {:?}", batch_req);
    println!("批量接收响应: {:?}", batch_resp);
    println!();

    // 8. 推送请求/响应
    let publish_req = PublishRequest::system_push("news_channel", "系统通知消息".as_bytes().to_vec());
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
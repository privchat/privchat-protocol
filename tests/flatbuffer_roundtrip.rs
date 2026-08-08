//! Round-trip tests for every message in the PrivChat protocol.
//!
//! For each message:
//!   owned struct → encode → decode → owned struct  (assert all fields equal)
//!
//! This protects against schema drift between the .fbs files and the
//! `FlatBufferMessage` impls.

use privchat_protocol::*;
use std::collections::HashMap;

fn roundtrip<T>(msg: &T) -> T
where
    T: FlatBufferMessage,
{
    let bytes = encode_message(msg).expect("encode failed");
    decode_message::<T>(&bytes).expect("decode failed")
}

#[test]
fn ping_request_roundtrip() {
    let msg = PingRequest {
        timestamp: 1714680000_000,
    };
    let got = roundtrip(&msg);
    assert_eq!(got.timestamp, msg.timestamp);
}

#[test]
fn pong_response_roundtrip() {
    let msg = PongResponse { timestamp: -42 };
    let got = roundtrip(&msg);
    assert_eq!(got.timestamp, msg.timestamp);
}

#[test]
fn disconnect_request_roundtrip() {
    let cases = [
        DisconnectRequest {
            reason: DisconnectReason::UserInitiated,
            message: Some("bye".to_string()),
        },
        DisconnectRequest {
            reason: DisconnectReason::ServerMaintenance,
            message: None,
        },
        DisconnectRequest {
            reason: DisconnectReason::AuthenticationFailed,
            message: Some(String::new()),
        },
    ];
    for msg in &cases {
        let got = roundtrip(msg);
        assert_eq!(got.reason, msg.reason);
        assert_eq!(got.message, msg.message);
    }
}

#[test]
fn disconnect_response_roundtrip() {
    for ack in [true, false] {
        let msg = DisconnectResponse { acknowledged: ack };
        assert_eq!(roundtrip(&msg).acknowledged, ack);
    }
}

#[test]
fn send_message_request_roundtrip() {
    let mut msg = SendMessageRequest::new();
    msg.setting = MessageSetting {
        need_receipt: true,
        signal: 7,
    };
    msg.client_seq = 42;
    msg.local_message_id = 0xDEADBEEF_CAFEBABE;
    msg.stream_no = "stream-1".to_string();
    msg.channel_id = 10001;
    msg.message_type = 99;
    msg.expire = 3600;
    msg.from_uid = 20002;
    msg.topic = "general".to_string();
    msg.payload = b"hello world \x00\xff".to_vec();

    let got = roundtrip(&msg);
    assert_eq!(got.setting, msg.setting);
    assert_eq!(got.client_seq, msg.client_seq);
    assert_eq!(got.local_message_id, msg.local_message_id);
    assert_eq!(got.stream_no, msg.stream_no);
    assert_eq!(got.channel_id, msg.channel_id);
    assert_eq!(got.message_type, msg.message_type);
    assert_eq!(got.expire, msg.expire);
    assert_eq!(got.from_uid, msg.from_uid);
    assert_eq!(got.topic, msg.topic);
    assert_eq!(got.payload, msg.payload);
}

#[test]
fn send_message_response_roundtrip() {
    let msg = SendMessageResponse {
        client_seq: 99,
        server_message_id: 0xCAFE_BABE,
        message_seq: 7,
        reason_code: 0,
    };
    let got = roundtrip(&msg);
    assert_eq!(got.client_seq, msg.client_seq);
    assert_eq!(got.server_message_id, msg.server_message_id);
    assert_eq!(got.message_seq, msg.message_seq);
    assert_eq!(got.reason_code, msg.reason_code);
}

#[test]
fn push_message_request_roundtrip() {
    let msg = PushMessageRequest {
        setting: MessageSetting {
            need_receipt: false,
            signal: 1,
        },
        msg_key: "key-abc".to_string(),
        server_message_id: 12345,
        message_seq: 10,
        local_message_id: 67890,
        stream_no: "s1".to_string(),
        stream_seq: 5,
        stream_flag: 2,
        timestamp: 1714680000,
        channel_id: 555,
        channel_type: 3,
        message_type: 4,
        expire: 600,
        topic: "topic".to_string(),
        from_uid: 99,
        payload: vec![1, 2, 3, 4, 5],
        deleted: true,
    };
    let got = roundtrip(&msg);
    assert_eq!(got.msg_key, msg.msg_key);
    assert_eq!(got.server_message_id, msg.server_message_id);
    assert_eq!(got.timestamp, msg.timestamp);
    assert_eq!(got.payload, msg.payload);
    assert_eq!(got.deleted, msg.deleted);
}

#[test]
fn push_message_response_roundtrip() {
    let cases = [
        PushMessageResponse {
            succeed: true,
            message: None,
        },
        PushMessageResponse {
            succeed: false,
            message: Some("err".to_string()),
        },
    ];
    for msg in &cases {
        let got = roundtrip(msg);
        assert_eq!(got.succeed, msg.succeed);
        assert_eq!(got.message, msg.message);
    }
}

#[test]
fn push_batch_roundtrip() {
    let inner = PushMessageRequest {
        setting: MessageSetting::default(),
        msg_key: "k".to_string(),
        server_message_id: 1,
        message_seq: 0,
        local_message_id: 0,
        stream_no: String::new(),
        stream_seq: 0,
        stream_flag: 0,
        timestamp: 0,
        channel_id: 100,
        channel_type: 0,
        message_type: 0,
        expire: 0,
        topic: String::new(),
        from_uid: 0,
        payload: b"abc".to_vec(),
        deleted: false,
    };
    let msg = PushBatchRequest {
        messages: vec![inner.clone(), inner],
    };
    let got = roundtrip(&msg);
    assert_eq!(got.messages.len(), 2);
    assert_eq!(got.messages[0].msg_key, "k");
    assert_eq!(got.messages[1].payload, b"abc");
}

#[test]
fn push_batch_response_roundtrip() {
    let msg = PushBatchResponse::success();
    let got = roundtrip(&msg);
    assert_eq!(got.succeed, true);
    assert!(got.message.is_some());
}

#[test]
fn subscribe_roundtrip() {
    let req = SubscribeRequest {
        setting: 0b1010,
        local_message_id: 100,
        channel_id: 200,
        channel_type: 1,
        action: 2,
        param: "filter=on".to_string(),
    };
    let got = roundtrip(&req);
    assert_eq!(got.setting, req.setting);
    assert_eq!(got.channel_id, req.channel_id);
    assert_eq!(got.param, req.param);

    let resp = SubscribeResponse {
        local_message_id: 100,
        channel_id: 200,
        channel_type: 1,
        action: 2,
        reason_code: 0,
    };
    let got = roundtrip(&resp);
    assert_eq!(got.reason_code, resp.reason_code);
}

#[test]
fn publish_roundtrip() {
    let req = PublishRequest {
        channel_id: 42,
        topic: Some("news".to_string()),
        timestamp: 1234567890,
        payload: b"event".to_vec(),
        publisher: Some("system".to_string()),
        server_message_id: Some(7),
    };
    let got = roundtrip(&req);
    assert_eq!(got.channel_id, req.channel_id);
    assert_eq!(got.topic, req.topic);
    assert_eq!(got.publisher, req.publisher);
    assert_eq!(got.server_message_id, req.server_message_id);
    assert_eq!(got.payload, req.payload);

    // Option=None roundtrip
    let req2 = PublishRequest {
        channel_id: 1,
        topic: None,
        timestamp: 0,
        payload: vec![],
        publisher: None,
        server_message_id: None,
    };
    let got = roundtrip(&req2);
    assert_eq!(got.topic, None);
    assert_eq!(got.publisher, None);
    assert_eq!(got.server_message_id, None);

    let resp = PublishResponse::failure("nope");
    let got = roundtrip(&resp);
    assert_eq!(got.succeed, false);
    assert_eq!(got.message.as_deref(), Some("nope"));
}

#[test]
fn rpc_request_roundtrip() {
    let req = RpcRequest {
        route: "account/user/find".to_string(),
        body: br#"{"user_id":"10001"}"#.to_vec(),
    };
    let got = roundtrip(&req);
    assert_eq!(got.route, req.route);
    assert_eq!(got.body, req.body);
}

#[test]
fn rpc_response_roundtrip() {
    let resp_with_data = RpcResponse::success(b"data".to_vec());
    let got = roundtrip(&resp_with_data);
    assert_eq!(got.code, 0);
    assert_eq!(got.data, Some(b"data".to_vec()));

    let resp_empty = RpcResponse::success_empty();
    let got = roundtrip(&resp_empty);
    assert_eq!(got.code, 0);
    assert_eq!(got.data, None); // empty [ubyte] decodes back to None

    let resp_err = RpcResponse::error(500, "boom".to_string());
    let got = roundtrip(&resp_err);
    assert_eq!(got.code, 500);
    assert_eq!(got.message, "boom");
    assert_eq!(got.data, None);
}

#[test]
fn transfer_request_roundtrip() {
    let req = TransferRequest {
        request_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        channel_id: 2817,
        route: "game/poker/raise".to_string(),
        body: br#"{"amount":200}"#.to_vec(),
    };
    let got = roundtrip(&req);
    assert_eq!(got.request_id, req.request_id);
    assert_eq!(got.channel_id, req.channel_id);
    assert_eq!(got.route, req.route);
    assert_eq!(got.body, req.body);
}

#[test]
fn transfer_response_roundtrip() {
    let req_id = "550e8400-e29b-41d4-a716-446655440000".to_string();

    let resp_with_data = TransferResponse::success(req_id.clone(), 2817, b"raise-result".to_vec());
    let got = roundtrip(&resp_with_data);
    assert_eq!(got.request_id, req_id);
    assert_eq!(got.channel_id, 2817);
    assert_eq!(got.code, 0);
    assert_eq!(got.data, Some(b"raise-result".to_vec()));

    let resp_empty = TransferResponse::success_empty(req_id.clone(), 2817);
    let got = roundtrip(&resp_empty);
    assert_eq!(got.request_id, req_id);
    assert_eq!(got.code, 0);
    assert_eq!(got.data, None); // empty [ubyte] decodes back to None

    let resp_err =
        TransferResponse::error(req_id.clone(), 2817, 20902, "service not found".to_string());
    let got = roundtrip(&resp_err);
    assert_eq!(got.request_id, req_id);
    assert_eq!(got.channel_id, 2817);
    assert_eq!(got.code, 20902);
    assert_eq!(got.message, "service not found");
    assert_eq!(got.data, None);
}

#[test]
fn authorization_request_roundtrip() {
    let mut props = HashMap::new();
    props.insert("locale".to_string(), "zh-CN".to_string());
    props.insert("tz".to_string(), "Asia/Shanghai".to_string());

    let req = AuthorizationRequest {
        auth_type: AuthType::JWT,
        auth_token: "eyJ...".to_string(),
        client_info: ClientInfo {
            client_type: "web".to_string(),
            version: "0.1.0".to_string(),
            os: "macos".to_string(),
            os_version: "14.5".to_string(),
            device_model: Some("MBP".to_string()),
            app_package: None,
        },
        device_info: DeviceInfo {
            device_id: "dev-1".to_string(),
            device_type: DeviceType::MacOS,
            app_id: "com.privchat".to_string(),
            push_token: Some("apns-token".to_string()),
            push_channel: None,
            device_name: "Jiaqing's MBP".to_string(),
            device_model: Some("M2".to_string()),
            os_version: Some("14.5".to_string()),
            app_version: Some("0.1".to_string()),
            manufacturer: Some("Apple".to_string()),
            device_fingerprint: None,
        },
        protocol_version: VERSION.to_string(),
        properties: props.clone(),
    };

    let got = roundtrip(&req);
    assert_eq!(got.auth_type, req.auth_type);
    assert_eq!(got.auth_token, req.auth_token);
    assert_eq!(got.client_info.client_type, "web");
    assert_eq!(got.client_info.device_model, Some("MBP".to_string()));
    assert_eq!(got.client_info.app_package, None);
    assert_eq!(got.device_info.device_type, DeviceType::MacOS);
    assert_eq!(got.device_info.push_token, Some("apns-token".to_string()));
    assert_eq!(got.device_info.push_channel, None);
    assert_eq!(got.protocol_version, req.protocol_version);
    assert_eq!(got.properties, props);
}

#[test]
fn authorization_response_roundtrip() {
    let resp = AuthorizationResponse {
        success: true,
        error_code: None,
        error_message: None,
        session_id: Some("sess-123".to_string()),
        user_id: Some(10001),
        connection_id: Some("conn-1".to_string()),
        server_info: Some(ServerInfo {
            version: "1.0".to_string(),
            name: "privchat".to_string(),
            features: vec!["e2ee".to_string(), "media".to_string()],
            max_message_size: 1024 * 1024,
            connection_timeout: 30,
        }),
        heartbeat_interval: Some(60),
    };
    let got = roundtrip(&resp);
    assert_eq!(got.success, true);
    assert_eq!(got.user_id, Some(10001));
    assert_eq!(got.session_id, Some("sess-123".to_string()));
    let si = got.server_info.unwrap();
    assert_eq!(si.features, vec!["e2ee", "media"]);
    assert_eq!(si.max_message_size, 1024 * 1024);
    assert_eq!(got.heartbeat_interval, Some(60));

    // Error response: success=false, no IDs
    let err = AuthorizationResponse {
        success: false,
        error_code: Some(401),
        error_message: Some("invalid token".to_string()),
        session_id: None,
        user_id: None,
        connection_id: None,
        server_info: None,
        heartbeat_interval: None,
    };
    let got = roundtrip(&err);
    assert_eq!(got.success, false);
    assert_eq!(got.error_code, Some(401));
    assert_eq!(got.user_id, None);
    assert_eq!(got.server_info.is_none(), true);
}

#[test]
fn message_type_values_locked() {
    // Wire-numeric stability — these MUST NOT change.
    assert_eq!(MessageType::Unknown as u8, 0);
    assert_eq!(MessageType::AuthorizationRequest as u8, 1);
    assert_eq!(MessageType::AuthorizationResponse as u8, 2);
    assert_eq!(MessageType::DisconnectRequest as u8, 3);
    assert_eq!(MessageType::DisconnectResponse as u8, 4);
    assert_eq!(MessageType::SendMessageRequest as u8, 5);
    assert_eq!(MessageType::SendMessageResponse as u8, 6);
    assert_eq!(MessageType::PushMessageRequest as u8, 7);
    assert_eq!(MessageType::PushMessageResponse as u8, 8);
    assert_eq!(MessageType::PushBatchRequest as u8, 9);
    assert_eq!(MessageType::PushBatchResponse as u8, 10);
    assert_eq!(MessageType::PingRequest as u8, 11);
    assert_eq!(MessageType::PongResponse as u8, 12);
    assert_eq!(MessageType::SubscribeRequest as u8, 13);
    assert_eq!(MessageType::SubscribeResponse as u8, 14);
    assert_eq!(MessageType::PublishRequest as u8, 15);
    assert_eq!(MessageType::PublishResponse as u8, 16);
    assert_eq!(MessageType::RpcRequest as u8, 17);
    assert_eq!(MessageType::RpcResponse as u8, 18);
    assert_eq!(MessageType::TransferRequest as u8, 19);
    assert_eq!(MessageType::TransferResponse as u8, 20);

    // From<u8> handles known values + falls back to Unknown for unknown.
    assert_eq!(MessageType::from(5), MessageType::SendMessageRequest);
    assert_eq!(MessageType::from(19), MessageType::TransferRequest);
    assert_eq!(MessageType::from(20), MessageType::TransferResponse);
    assert_eq!(MessageType::from(0), MessageType::Unknown);
    assert_eq!(MessageType::from(99), MessageType::Unknown); // safer than legacy "fallback to AuthRequest"
}

// ------------------------------------------------------------------
// MessagePayloadEnvelope (content.fbs)
// ------------------------------------------------------------------

#[test]
fn payload_envelope_text_no_metadata_roundtrip() {
    // Text / system messages: no metadata.
    let env = MessagePayloadEnvelope {
        content: "你好世界 hello".to_string(),
        metadata: None,
        reply_to_message_id: Some(0xCAFE_BABE),
        mentioned_user_ids: vec![100, 200, 300],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    assert_eq!(got.content, env.content);
    assert!(got.metadata.is_none());
    assert_eq!(got.reply_to_message_id, Some(0xCAFE_BABE));
    assert_eq!(got.mentioned_user_ids, vec![100, 200, 300]);
    assert!(got.message_source.is_none());
}

#[test]
fn payload_envelope_image_metadata_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Image(ImageMetadata {
            file_id: 12345,
            url: Some("https://cdn.example/x.jpg".to_string()),
            width: 1920,
            height: 1080,
            thumbnail_file_id: Some(67890),
            thumbnail_url: Some("https://cdn.example/x_thumb.jpg".to_string()),
            file_name: Some("photo.jpg".to_string()),
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    match got.metadata {
        Some(MessageMetadata::Image(img)) => {
            assert_eq!(img.file_id, 12345);
            assert_eq!(img.url.as_deref(), Some("https://cdn.example/x.jpg"));
            assert_eq!(img.width, 1920);
            assert_eq!(img.height, 1080);
            // 缩略图引用必须过 wire（Scheme B：thumbnail_file_id -> get_url -> cek）。
            assert_eq!(img.thumbnail_file_id, Some(67890));
            assert_eq!(
                img.thumbnail_url.as_deref(),
                Some("https://cdn.example/x_thumb.jpg")
            );
            assert_eq!(img.file_name.as_deref(), Some("photo.jpg"));
        }
        other => panic!("expected Image metadata, got {:?}", other),
    }
}

#[test]
fn payload_envelope_image_metadata_no_thumbnail_roundtrip() {
    // v1 加密附件可能没有 legacy thumbnail_url；thumbnail_file_id 缺省也要 round-trip 成 None。
    let env = MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Image(ImageMetadata {
            file_id: 1,
            url: None,
            width: 10,
            height: 10,
            thumbnail_file_id: None,
            thumbnail_url: None,
            file_name: None,
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    match roundtrip(&env).metadata {
        Some(MessageMetadata::Image(img)) => {
            assert_eq!(img.thumbnail_file_id, None);
            assert_eq!(img.thumbnail_url, None);
            assert_eq!(img.file_name, None);
        }
        other => panic!("expected Image metadata, got {:?}", other),
    }
}

#[test]
fn payload_envelope_voice_metadata_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Voice(VoiceMetadata {
            file_id: 999,
            duration: 7,
            file_name: Some("voice.m4a".to_string()),
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    match got.metadata {
        Some(MessageMetadata::Voice(voice)) => {
            assert_eq!(voice.file_id, 999);
            assert_eq!(voice.duration, 7);
            assert_eq!(voice.file_name.as_deref(), Some("voice.m4a"));
        }
        other => panic!("expected Voice metadata, got {:?}", other),
    }
}

#[test]
fn payload_envelope_video_metadata_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: "看视频".to_string(),
        metadata: Some(MessageMetadata::Video(VideoMetadata {
            file_id: 5555,
            duration: 60,
            width: 1280,
            height: 720,
            thumbnail_file_id: Some(5556),
            thumbnail_width: Some(320),
            thumbnail_height: Some(180),
            thumbnail_url: Some("https://cdn.example/v_thumb.jpg".to_string()),
            file_name: Some("clip.mp4".to_string()),
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    match got.metadata {
        Some(MessageMetadata::Video(v)) => {
            assert_eq!(v.file_id, 5555);
            assert_eq!(v.duration, 60);
            assert_eq!(v.width, 1280);
            assert_eq!(v.thumbnail_file_id, Some(5556));
            assert_eq!(v.thumbnail_width, Some(320));
            assert_eq!(v.thumbnail_height, Some(180));
            assert_eq!(
                v.thumbnail_url.as_deref(),
                Some("https://cdn.example/v_thumb.jpg")
            );
            assert_eq!(v.file_name.as_deref(), Some("clip.mp4"));
        }
        other => panic!("expected Video metadata, got {:?}", other),
    }

    // None on optional thumbnail fields → 0 on wire → None back.
    let env2 = MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Video(VideoMetadata {
            file_id: 1,
            duration: 5,
            width: 100,
            height: 100,
            thumbnail_file_id: None,
            thumbnail_width: None,
            thumbnail_height: None,
            thumbnail_url: None,
            file_name: None,
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got2 = roundtrip(&env2);
    if let Some(MessageMetadata::Video(v)) = got2.metadata {
        assert_eq!(v.thumbnail_file_id, None);
        assert_eq!(v.thumbnail_width, None);
        assert_eq!(v.thumbnail_url, None);
        assert_eq!(v.file_name, None);
    } else {
        panic!("expected Video metadata");
    }
}

#[test]
fn payload_envelope_file_metadata_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: "report.pdf".to_string(),
        metadata: Some(MessageMetadata::File(FileMetadata {
            file_id: 42,
            file_name: Some("report.pdf".to_string()),
            file_size: 1_048_576,
            mime_type: Some("application/pdf".to_string()),
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    match got.metadata {
        Some(MessageMetadata::File(file)) => {
            assert_eq!(file.file_id, 42);
            assert_eq!(file.file_name.as_deref(), Some("report.pdf"));
            assert_eq!(file.file_size, 1_048_576);
            assert_eq!(file.mime_type.as_deref(), Some("application/pdf"));
        }
        other => panic!("expected File metadata, got {:?}", other),
    }
}

#[test]
fn payload_envelope_location_metadata_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: "Shanghai".to_string(),
        metadata: Some(MessageMetadata::Location(LocationMetadata {
            latitude: 31.2304,
            longitude: 121.4737,
            coordinate_system: None,
            name: None,
            address: None,
            poi_id: None,
            poi_source: None,
            thumbnail_file_id: None,
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    if let Some(MessageMetadata::Location(loc)) = got.metadata {
        assert!((loc.latitude - 31.2304).abs() < 1e-9);
        assert!((loc.longitude - 121.4737).abs() < 1e-9);
        assert_eq!(loc.coordinate_system, None);
        assert_eq!(loc.thumbnail_file_id, None);
    } else {
        panic!("expected Location");
    }
}

#[test]
fn payload_envelope_location_extended_metadata_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: "Starbucks".to_string(),
        metadata: Some(MessageMetadata::Location(LocationMetadata {
            latitude: 39.9087,
            longitude: 116.3975,
            coordinate_system: Some("gcj02".to_string()),
            name: Some("Starbucks Guomao".to_string()),
            address: Some("Beijing CBD".to_string()),
            poi_id: Some("amap-poi-1".to_string()),
            poi_source: Some("amap".to_string()),
            thumbnail_file_id: Some(8848),
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    if let Some(MessageMetadata::Location(loc)) = got.metadata {
        assert!((loc.latitude - 39.9087).abs() < 1e-9);
        assert!((loc.longitude - 116.3975).abs() < 1e-9);
        assert_eq!(loc.coordinate_system.as_deref(), Some("gcj02"));
        assert_eq!(loc.name.as_deref(), Some("Starbucks Guomao"));
        assert_eq!(loc.address.as_deref(), Some("Beijing CBD"));
        assert_eq!(loc.poi_id.as_deref(), Some("amap-poi-1"));
        assert_eq!(loc.poi_source.as_deref(), Some("amap"));
        assert_eq!(loc.thumbnail_file_id, Some(8848));
    } else {
        panic!("expected Location");
    }
}

#[test]
fn payload_envelope_contact_card_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::ContactCard(ContactCardMetadata {
            user_id: 10001,
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    assert!(matches!(
        got.metadata,
        Some(MessageMetadata::ContactCard(ContactCardMetadata {
            user_id: 10001
        }))
    ));
}

#[test]
fn payload_envelope_sticker_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Sticker(StickerMetadata {
            sticker_id: "smile-001".to_string(),
            image_url: "https://stickers.example/s001.webp".to_string(),
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    if let Some(MessageMetadata::Sticker(s)) = got.metadata {
        assert_eq!(s.sticker_id, "smile-001");
        assert_eq!(s.image_url, "https://stickers.example/s001.webp");
    } else {
        panic!("expected Sticker");
    }
}

#[test]
fn payload_envelope_forward_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: "transcript".to_string(),
        metadata: Some(MessageMetadata::Forward(ForwardMetadata {
            messages: vec![
                ForwardMessageRef {
                    message_id: Some(111),
                    content: Some("hi".to_string()),
                    extra: br#"{"vendor":"x"}"#.to_vec(),
                },
                ForwardMessageRef {
                    message_id: None,
                    content: None,
                    extra: vec![],
                },
            ],
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    if let Some(MessageMetadata::Forward(fwd)) = got.metadata {
        assert_eq!(fwd.messages.len(), 2);
        assert_eq!(fwd.messages[0].message_id, Some(111));
        assert_eq!(fwd.messages[0].content.as_deref(), Some("hi"));
        assert_eq!(fwd.messages[0].extra, br#"{"vendor":"x"}"#.to_vec());
        assert_eq!(fwd.messages[1].message_id, None);
        assert_eq!(fwd.messages[1].content, None);
        assert!(fwd.messages[1].extra.is_empty());
    } else {
        panic!("expected Forward");
    }
}

#[test]
fn payload_envelope_link_roundtrip() {
    let env = MessagePayloadEnvelope {
        content: "check this out".to_string(),
        metadata: Some(MessageMetadata::Link(LinkMetadata {
            url: "https://example.com/article".to_string(),
            title: Some("Article".to_string()),
            description: Some("Long description here".to_string()),
            thumbnail_file_id: Some(789),
        })),
        reply_to_message_id: Some(42),
        mentioned_user_ids: vec![],
        message_source: None,
        forward_origin: None,
    };
    let got = roundtrip(&env);
    if let Some(MessageMetadata::Link(l)) = got.metadata {
        assert_eq!(l.url, "https://example.com/article");
        assert_eq!(l.title.as_deref(), Some("Article"));
        assert_eq!(l.thumbnail_file_id, Some(789));
    } else {
        panic!("expected Link");
    }
    assert_eq!(got.reply_to_message_id, Some(42));
}

#[test]
fn payload_envelope_message_source_roundtrip() {
    // Stranger message: source = qrcode
    let env = MessagePayloadEnvelope {
        content: "hi from a qr add".to_string(),
        metadata: None,
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: Some(MessageSource {
            source_type: "qrcode".to_string(),
            source_id: "qr-token-abc".to_string(),
        }),
        forward_origin: None,
    };
    let got = roundtrip(&env);
    let src = got.message_source.expect("source must roundtrip");
    assert_eq!(src.source_type, "qrcode");
    assert_eq!(src.source_id, "qr-token-abc");
}

#[test]
fn payload_envelope_full_combination() {
    // Worst-case: text with reply + mentions + source
    let env = MessagePayloadEnvelope {
        content: "@bob @charlie 看下这条".to_string(),
        metadata: None,
        reply_to_message_id: Some(0xDEAD_BEEF_CAFE_BABE),
        mentioned_user_ids: vec![10001, 10002, 10003],
        message_source: Some(MessageSource {
            source_type: "search".to_string(),
            source_id: "session-99".to_string(),
        }),
        forward_origin: None,
    };
    let got = roundtrip(&env);
    assert_eq!(got.content, env.content);
    assert_eq!(got.reply_to_message_id, Some(0xDEAD_BEEF_CAFE_BABE));
    assert_eq!(got.mentioned_user_ids, vec![10001, 10002, 10003]);
    assert_eq!(got.message_source.unwrap().source_type, "search");
}

#[test]
fn enum_zero_is_unknown() {
    // FlatBuffers default-value safety: every enum's 0 must be the "unset" tag.
    assert_eq!(AuthType::Unspecified as u8, 0);
    assert_eq!(DeviceType::Unknown as u8, 0);
    assert_eq!(DisconnectReason::Unknown as u8, 0);
    assert_eq!(MessageType::Unknown as u8, 0);
}

// ---------------------------------------------------------------------------
// REAL outbound send path: legacy attachment JSON (as SDK builds it) ->
// from_legacy -> encode -> decode -> typed metadata MUST retain file_id +
// thumbnail_file_id. This mirrors process_outbound_file exactly and guards the
// 20006 MessageContentInvalid failure mode (server can't find file_id).
// ---------------------------------------------------------------------------
#[test]
fn outbound_image_legacy_json_preserves_file_id_through_wire() {
    use privchat_protocol::message::{ContentMessageType, LocalMessagePayloadEnvelope};

    // Exactly the shape SDK's process_outbound_file puts in attachment_content.
    let attachment = serde_json::json!({
        "file_type": "image",
        "file_id": 166u64,
        "thumbnail_file_id": 165u64,
        "filename": "x.jpg",
        "mime_type": "image/jpeg",
        "storage_source_id": 0,
        "file_size": 1852262u64,
        "file_url": "http://127.0.0.1:8000/images/166.jpg",
        "thumbnail_url": "http://127.0.0.1:8000/images/165.webp"
    });
    let legacy = LocalMessagePayloadEnvelope {
        content: "[image]".to_string(),
        metadata: Some(attachment),
        reply_to_message_id: None,
        mentioned_user_ids: None,
        message_source: None,
        forward_origin: None,
    };

    let typed = MessagePayloadEnvelope::from_legacy(&legacy, ContentMessageType::Image);
    let bytes = encode_message(&typed).expect("encode");
    let decoded = decode_message::<MessagePayloadEnvelope>(&bytes).expect("decode");

    match decoded.metadata {
        Some(MessageMetadata::Image(img)) => {
            assert_eq!(
                img.file_id, 166,
                "main file_id must survive the real send path"
            );
            assert_eq!(
                img.thumbnail_file_id,
                Some(165),
                "thumbnail_file_id must survive"
            );
        }
        other => panic!("expected Image metadata with file_id, got {:?}", other),
    }
}

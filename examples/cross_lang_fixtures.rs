//! Cross-language fixture generator / verifier.
//!
//! Two modes:
//!   `cargo run --example cross_lang_fixtures` (default: dump)
//!     Writes canonical .bin files + manifest.json under
//!     `../privchat-sdk-typescript/tests/fixtures/from-rust/`.
//!     The TS test reads these and asserts decoded fields match the manifest.
//!
//!   `cargo run --example cross_lang_fixtures verify`
//!     Reads .bin files under `../privchat-sdk-typescript/tests/fixtures/from-ts/`
//!     and asserts each decodes to the same canonical Rust struct that the
//!     dump mode emits. This proves TS encode is byte-compatible with the
//!     Rust decoder.

use privchat_protocol::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn ts_fixtures_root() -> PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .expect("crate has parent")
        .join("privchat-sdk-typescript/tests/fixtures")
}

// ------------------------------------------------------------------
// Canonical fixture set. Both modes use these exact values.
// ------------------------------------------------------------------

fn ping_fixture() -> PingRequest {
    PingRequest { timestamp: 1_714_680_000_000 }
}

fn pong_fixture() -> PongResponse {
    PongResponse { timestamp: 1_714_680_001_500 }
}

fn subscribe_request_fixture() -> SubscribeRequest {
    SubscribeRequest {
        setting: 0x07,
        local_message_id: 42,
        channel_id: 900_710_001,
        channel_type: 2,
        action: 1,
        param: "history=true&limit=20".to_string(),
    }
}

fn subscribe_response_fixture() -> SubscribeResponse {
    SubscribeResponse {
        local_message_id: 42,
        channel_id: 900_710_001,
        channel_type: 2,
        action: 1,
        reason_code: 0,
    }
}

fn send_request_fixture() -> SendMessageRequest {
    SendMessageRequest {
        setting: MessageSetting { need_receipt: true, signal: 0 },
        client_seq: 42,
        local_message_id: 900_710_001,
        stream_no: String::new(),
        channel_id: 12_345,
        message_type: 0,
        expire: 0,
        from_uid: 999,
        topic: String::new(),
        payload: br#"{"content":"hi"}"#.to_vec(),
    }
}

fn send_response_fixture() -> SendMessageResponse {
    SendMessageResponse {
        client_seq: 42,
        server_message_id: 700_110_001,
        message_seq: 100,
        reason_code: 0,
    }
}

fn push_message_fixture() -> PushMessageRequest {
    PushMessageRequest {
        setting: MessageSetting { need_receipt: true, signal: 0 },
        msg_key: "k-1".to_string(),
        server_message_id: 700_110_001,
        message_seq: 100,
        local_message_id: 900_710_001,
        stream_no: String::new(),
        stream_seq: 0,
        stream_flag: 0,
        timestamp: 1_714_680_000,
        channel_id: 12_345,
        channel_type: 1,
        message_type: 0,
        expire: 0,
        topic: String::new(),
        from_uid: 999,
        payload: br#"{"content":"hi"}"#.to_vec(),
        deleted: false,
    }
}

fn push_batch_fixture() -> PushBatchRequest {
    let mut a = push_message_fixture();
    a.msg_key = "k-1".to_string();
    a.server_message_id = 1;
    a.message_seq = 1;

    let mut b = push_message_fixture();
    b.msg_key = "k-2".to_string();
    b.server_message_id = 2;
    b.message_seq = 2;

    let mut c = push_message_fixture();
    c.msg_key = "k-3".to_string();
    c.server_message_id = 3;
    c.message_seq = 3;
    c.deleted = true;

    PushBatchRequest { messages: vec![a, b, c] }
}

fn auth_request_fixture() -> AuthorizationRequest {
    let mut props = HashMap::new();
    props.insert("region".to_string(), "us-west".to_string());
    props.insert("tenant".to_string(), "demo".to_string());

    AuthorizationRequest {
        auth_type: AuthType::JWT,
        auth_token: "eyJhbGciOi...".to_string(),
        client_info: ClientInfo {
            client_type: "web".to_string(),
            version: "0.1.0".to_string(),
            os: "macOS".to_string(),
            os_version: "15.0".to_string(),
            device_model: Some("MacBookPro18,1".to_string()),
            app_package: Some("com.privchat.web".to_string()),
        },
        device_info: DeviceInfo {
            device_id: "dev-123".to_string(),
            device_type: DeviceType::Web,
            app_id: "app-1".to_string(),
            push_token: Some("fcm-token".to_string()),
            push_channel: Some("fcm".to_string()),
            device_name: "Chrome 130".to_string(),
            device_model: None,
            os_version: Some("15.0".to_string()),
            app_version: Some("0.1.0".to_string()),
            manufacturer: Some("Apple".to_string()),
            device_fingerprint: Some("fp-abc".to_string()),
        },
        protocol_version: "1.0".to_string(),
        properties: props,
    }
}

fn auth_response_fixture() -> AuthorizationResponse {
    AuthorizationResponse {
        success: true,
        error_code: None,
        error_message: None,
        session_id: Some("sess-1".to_string()),
        user_id: Some(900_710_001),
        connection_id: Some("conn-1".to_string()),
        server_info: Some(ServerInfo {
            version: "1.0.0".to_string(),
            name: "privchat".to_string(),
            features: vec!["fb".to_string(), "multi-device".to_string()],
            max_message_size: 4_194_304,
            connection_timeout: 60,
        }),
        heartbeat_interval: Some(30),
    }
}

fn payload_text_fixture() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: "hello world".to_string(),
        metadata: None,
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
    }
}

fn payload_image_fixture() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Image(ImageMetadata {
            file_id: 500_110_001,
            url: Some("https://cdn.example/img.jpg".to_string()),
            width: 1920,
            height: 1080,
        })),
        reply_to_message_id: Some(700_110_001),
        mentioned_user_ids: vec![1001, 1002],
        message_source: Some(MessageSource {
            source_type: "group".to_string(),
            source_id: "g-42".to_string(),
        }),
    }
}

fn payload_video_fixture() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Video(VideoMetadata {
            file_id: 500_110_005,
            duration: 30,
            width: 1280,
            height: 720,
            thumbnail_file_id: Some(500_110_006),
            thumbnail_width: Some(640),
            thumbnail_height: Some(360),
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
    }
}

fn payload_forward_fixture() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Forward(ForwardMetadata {
            messages: vec![
                ForwardMessageRef {
                    message_id: Some(700_110_001),
                    content: Some("hi".to_string()),
                    extra: vec![],
                },
                ForwardMessageRef {
                    message_id: None,
                    content: Some("inline only".to_string()),
                    extra: br#"{"v":1}"#.to_vec(),
                },
                ForwardMessageRef {
                    message_id: Some(700_110_002),
                    content: None,
                    extra: vec![],
                },
            ],
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
    }
}

fn payload_link_fixture() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Link(LinkMetadata {
            url: "https://example.com/a".to_string(),
            title: Some("Example".to_string()),
            description: Some("A link".to_string()),
            thumbnail_file_id: Some(500_110_008),
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
    }
}

// ------------------------------------------------------------------
// Dump
// ------------------------------------------------------------------

fn dump() {
    let dir = ts_fixtures_root().join("from-rust");
    fs::create_dir_all(&dir).expect("create from-rust dir");

    let mut manifest = serde_json::Map::new();

    macro_rules! emit {
        ($name:expr, $msg:expr) => {{
            let bytes = encode_message(&$msg).expect("encode");
            fs::write(dir.join(format!("{}.bin", $name)), &bytes).expect("write bin");
            manifest.insert(
                $name.to_string(),
                json!({
                    "byte_length": bytes.len(),
                    "value": serde_json::to_value(&$msg).expect("serde"),
                }),
            );
        }};
    }

    emit!("ping", ping_fixture());
    emit!("pong", pong_fixture());
    emit!("subscribe_request", subscribe_request_fixture());
    emit!("subscribe_response", subscribe_response_fixture());
    emit!("send_request", send_request_fixture());
    emit!("send_response", send_response_fixture());
    emit!("push_message", push_message_fixture());
    emit!("push_batch", push_batch_fixture());
    emit!("auth_request", auth_request_fixture());
    emit!("auth_response", auth_response_fixture());
    emit!("payload_text", payload_text_fixture());
    emit!("payload_image", payload_image_fixture());
    emit!("payload_video", payload_video_fixture());
    emit!("payload_forward", payload_forward_fixture());
    emit!("payload_link", payload_link_fixture());

    let manifest_path = dir.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&serde_json::Value::Object(manifest)).expect("manifest json"),
    )
    .expect("write manifest");

    println!(
        "wrote {} fixtures + manifest.json → {}",
        15,
        dir.display()
    );
}

// ------------------------------------------------------------------
// Verify
// ------------------------------------------------------------------

fn verify() {
    let dir = ts_fixtures_root().join("from-ts");
    if !dir.exists() {
        eprintln!(
            "from-ts/ not found at {}\nrun the TS cross-lang test first to produce fixtures",
            dir.display()
        );
        std::process::exit(1);
    }

    let read = |name: &str| -> Vec<u8> {
        fs::read(dir.join(format!("{}.bin", name)))
            .unwrap_or_else(|e| panic!("read {}.bin: {}", name, e))
    };

    let mut failures: Vec<String> = vec![];

    macro_rules! check {
        ($name:expr, $ty:ty, $expected:expr, $eq:expr) => {{
            let bytes = read($name);
            match decode_message::<$ty>(&bytes) {
                Ok(got) => {
                    let ok: bool = $eq(&got, &$expected);
                    if !ok {
                        failures.push(format!(
                            "{}: decode succeeded but value mismatch\n  expected: {:?}\n  got:      {:?}",
                            $name, $expected, got
                        ));
                    } else {
                        println!("ok  {}", $name);
                    }
                }
                Err(e) => failures.push(format!("{}: decode failed: {:?}", $name, e)),
            }
        }};
    }

    check!("ping", PingRequest, ping_fixture(), |a: &PingRequest, b: &PingRequest| a.timestamp == b.timestamp);
    check!("pong", PongResponse, pong_fixture(), |a: &PongResponse, b: &PongResponse| a.timestamp == b.timestamp);
    check!("subscribe_request", SubscribeRequest, subscribe_request_fixture(), eq_subscribe_req);
    check!("subscribe_response", SubscribeResponse, subscribe_response_fixture(), eq_subscribe_resp);
    check!("send_request", SendMessageRequest, send_request_fixture(), eq_send_req);
    check!("send_response", SendMessageResponse, send_response_fixture(), eq_send_resp);
    check!("push_message", PushMessageRequest, push_message_fixture(), eq_push_msg);
    check!("push_batch", PushBatchRequest, push_batch_fixture(), eq_push_batch);
    check!("auth_request", AuthorizationRequest, auth_request_fixture(), eq_auth_req);
    check!("auth_response", AuthorizationResponse, auth_response_fixture(), eq_auth_resp);
    check!("payload_text", MessagePayloadEnvelope, payload_text_fixture(), |a: &MessagePayloadEnvelope, b: &MessagePayloadEnvelope| a == b);
    check!("payload_image", MessagePayloadEnvelope, payload_image_fixture(), |a: &MessagePayloadEnvelope, b: &MessagePayloadEnvelope| a == b);
    check!("payload_video", MessagePayloadEnvelope, payload_video_fixture(), |a: &MessagePayloadEnvelope, b: &MessagePayloadEnvelope| a == b);
    check!("payload_forward", MessagePayloadEnvelope, payload_forward_fixture(), |a: &MessagePayloadEnvelope, b: &MessagePayloadEnvelope| a == b);
    check!("payload_link", MessagePayloadEnvelope, payload_link_fixture(), |a: &MessagePayloadEnvelope, b: &MessagePayloadEnvelope| a == b);

    if !failures.is_empty() {
        eprintln!("\n{} fixture(s) failed:", failures.len());
        for f in &failures {
            eprintln!("\n{}", f);
        }
        std::process::exit(1);
    }
    println!("\nall {} TS-produced fixtures verified", 15);
}

// Equality helpers (these structs don't all derive PartialEq).

fn eq_subscribe_req(a: &SubscribeRequest, b: &SubscribeRequest) -> bool {
    a.setting == b.setting
        && a.local_message_id == b.local_message_id
        && a.channel_id == b.channel_id
        && a.channel_type == b.channel_type
        && a.action == b.action
        && a.param == b.param
}

fn eq_subscribe_resp(a: &SubscribeResponse, b: &SubscribeResponse) -> bool {
    a.local_message_id == b.local_message_id
        && a.channel_id == b.channel_id
        && a.channel_type == b.channel_type
        && a.action == b.action
        && a.reason_code == b.reason_code
}

fn eq_setting(a: &MessageSetting, b: &MessageSetting) -> bool {
    a.need_receipt == b.need_receipt && a.signal == b.signal
}

fn eq_send_req(a: &SendMessageRequest, b: &SendMessageRequest) -> bool {
    eq_setting(&a.setting, &b.setting)
        && a.client_seq == b.client_seq
        && a.local_message_id == b.local_message_id
        && a.stream_no == b.stream_no
        && a.channel_id == b.channel_id
        && a.message_type == b.message_type
        && a.expire == b.expire
        && a.from_uid == b.from_uid
        && a.topic == b.topic
        && a.payload == b.payload
}

fn eq_send_resp(a: &SendMessageResponse, b: &SendMessageResponse) -> bool {
    a.client_seq == b.client_seq
        && a.server_message_id == b.server_message_id
        && a.message_seq == b.message_seq
        && a.reason_code == b.reason_code
}

fn eq_push_msg(a: &PushMessageRequest, b: &PushMessageRequest) -> bool {
    eq_setting(&a.setting, &b.setting)
        && a.msg_key == b.msg_key
        && a.server_message_id == b.server_message_id
        && a.message_seq == b.message_seq
        && a.local_message_id == b.local_message_id
        && a.stream_no == b.stream_no
        && a.stream_seq == b.stream_seq
        && a.stream_flag == b.stream_flag
        && a.timestamp == b.timestamp
        && a.channel_id == b.channel_id
        && a.channel_type == b.channel_type
        && a.message_type == b.message_type
        && a.expire == b.expire
        && a.topic == b.topic
        && a.from_uid == b.from_uid
        && a.payload == b.payload
        && a.deleted == b.deleted
}

fn eq_push_batch(a: &PushBatchRequest, b: &PushBatchRequest) -> bool {
    a.messages.len() == b.messages.len()
        && a.messages.iter().zip(b.messages.iter()).all(|(x, y)| eq_push_msg(x, y))
}

fn eq_client_info(a: &ClientInfo, b: &ClientInfo) -> bool {
    a.client_type == b.client_type
        && a.version == b.version
        && a.os == b.os
        && a.os_version == b.os_version
        && a.device_model == b.device_model
        && a.app_package == b.app_package
}

fn eq_device_info(a: &DeviceInfo, b: &DeviceInfo) -> bool {
    a.device_id == b.device_id
        && a.device_type == b.device_type
        && a.app_id == b.app_id
        && a.push_token == b.push_token
        && a.push_channel == b.push_channel
        && a.device_name == b.device_name
        && a.device_model == b.device_model
        && a.os_version == b.os_version
        && a.app_version == b.app_version
        && a.manufacturer == b.manufacturer
        && a.device_fingerprint == b.device_fingerprint
}

fn eq_server_info(a: &ServerInfo, b: &ServerInfo) -> bool {
    a.version == b.version
        && a.name == b.name
        && a.features == b.features
        && a.max_message_size == b.max_message_size
        && a.connection_timeout == b.connection_timeout
}

fn eq_auth_req(a: &AuthorizationRequest, b: &AuthorizationRequest) -> bool {
    a.auth_type == b.auth_type
        && a.auth_token == b.auth_token
        && eq_client_info(&a.client_info, &b.client_info)
        && eq_device_info(&a.device_info, &b.device_info)
        && a.protocol_version == b.protocol_version
        && a.properties == b.properties
}

fn eq_auth_resp(a: &AuthorizationResponse, b: &AuthorizationResponse) -> bool {
    a.success == b.success
        && a.error_code == b.error_code
        && a.error_message == b.error_message
        && a.session_id == b.session_id
        && a.user_id == b.user_id
        && a.connection_id == b.connection_id
        && match (&a.server_info, &b.server_info) {
            (Some(x), Some(y)) => eq_server_info(x, y),
            (None, None) => true,
            _ => false,
        }
        && a.heartbeat_interval == b.heartbeat_interval
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("dump");
    match mode {
        "dump" => dump(),
        "verify" => verify(),
        other => {
            eprintln!("unknown mode: {} (expected dump | verify)", other);
            std::process::exit(2);
        }
    }
}

//! Payload-layer codec benchmark: serde_json vs FlatBuffers head-to-head,
//! plus the composite "full inbound message" path that mirrors what the
//! server actually does on every received message.
//!
//! Sections:
//!
//! 1. **Pure envelope codec (4 variants)** — Text / Reply / Image / FullMetadata
//!    Single-layer encode/decode of `MessagePayloadEnvelope`.
//!
//! 2. **Full inbound path** — The server's real hot path:
//!    `wire bytes → SendMessageRequest → MessagePayloadEnvelope`.
//!    Compares two-layer JSON vs two-layer FlatBuffers.
//!    *This is the headline number* — directly proportional to server CPU.
//!
//! 3. **Bridge cost** — `legacy JSON envelope → typed → FB`. This is the
//!    SDK *output* path (NOT the server inbound hot path). Reported
//!    separately so it isn't conflated with server CPU savings.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use privchat_protocol::message::{ContentMessageType, LocalMessagePayloadEnvelope};
use privchat_protocol::{
    decode_message, encode_message, ImageMetadata, MessageMetadata, MessagePayloadEnvelope,
    MessageSetting, MessageSource, SendMessageRequest,
};

// ---------------------------------------------------------------------------
// Fixture builders
// ---------------------------------------------------------------------------

const MENTION_BODY: &str = "@bob @charlie 看下这条 hello world this is a moderately sized chat";

/// Plain text — most common bubble; no metadata, no reply, no mentions.
fn text_typed() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: "你好世界 hello world this is a moderately sized chat message body \
                  that mirrors a realistic IM payload across the wire."
            .to_string(),
        metadata: None,
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
    }
}
fn text_legacy() -> LocalMessagePayloadEnvelope {
    LocalMessagePayloadEnvelope {
        content: text_typed().content,
        metadata: None,
        reply_to_message_id: None,
        mentioned_user_ids: None,
        message_source: None,
    }
}

/// Reply — text body referencing another message.
fn reply_typed() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: "回复消息内容".to_string(),
        metadata: None,
        reply_to_message_id: Some(0xDEAD_BEEF_CAFE_BABE),
        mentioned_user_ids: vec![],
        message_source: None,
    }
}
fn reply_legacy() -> LocalMessagePayloadEnvelope {
    LocalMessagePayloadEnvelope {
        content: "回复消息内容".to_string(),
        metadata: None,
        reply_to_message_id: Some("16045690984833335486".to_string()),
        mentioned_user_ids: None,
        message_source: None,
    }
}

/// Image — typical media message with metadata.
fn image_typed() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(MessageMetadata::Image(ImageMetadata {
            file_id: 12345,
            url: Some("https://cdn.example.com/img/9f3a/x.jpg".to_string()),
            width: 1920,
            height: 1080,
        })),
        reply_to_message_id: None,
        mentioned_user_ids: vec![],
        message_source: None,
    }
}
fn image_legacy() -> LocalMessagePayloadEnvelope {
    LocalMessagePayloadEnvelope {
        content: String::new(),
        metadata: Some(serde_json::json!({
            "file_id": 12345_u64,
            "url": "https://cdn.example.com/img/9f3a/x.jpg",
            "width": 1920_u32,
            "height": 1080_u32,
        })),
        reply_to_message_id: None,
        mentioned_user_ids: None,
        message_source: None,
    }
}

/// Worst-case: full envelope — content + image metadata + reply +
/// mentions + stranger source.
fn full_typed() -> MessagePayloadEnvelope {
    MessagePayloadEnvelope {
        content: MENTION_BODY.to_string(),
        metadata: Some(MessageMetadata::Image(ImageMetadata {
            file_id: 99999,
            url: Some("https://cdn.example.com/img/abc/full.jpg".to_string()),
            width: 1920,
            height: 1080,
        })),
        reply_to_message_id: Some(0xDEAD_BEEF_CAFE_BABE),
        mentioned_user_ids: vec![10001, 10002, 10003],
        message_source: Some(MessageSource {
            source_type: "search".to_string(),
            source_id: "session-99".to_string(),
        }),
    }
}
fn full_legacy() -> LocalMessagePayloadEnvelope {
    LocalMessagePayloadEnvelope {
        content: MENTION_BODY.to_string(),
        metadata: Some(serde_json::json!({
            "file_id": 99999_u64,
            "url": "https://cdn.example.com/img/abc/full.jpg",
            "width": 1920_u32,
            "height": 1080_u32,
        })),
        reply_to_message_id: Some("16045690984833335486".to_string()),
        mentioned_user_ids: Some(vec![10001_u64, 10002, 10003]),
        message_source: Some(MessageSource {
            source_type: "search".to_string(),
            source_id: "session-99".to_string(),
        }),
    }
}

// ---------------------------------------------------------------------------
// 1. Pure envelope codec
// ---------------------------------------------------------------------------

fn run_envelope_variant(
    c: &mut Criterion,
    label: &str,
    typed: MessagePayloadEnvelope,
    legacy: LocalMessagePayloadEnvelope,
) {
    let json_bytes = serde_json::to_vec(&legacy).unwrap();
    let fb_bytes = encode_message(&typed).unwrap();

    eprintln!(
        "[wire size] envelope/{}: JSON={}B, FlatBuffers={}B (Δ {:+.0}%)",
        label,
        json_bytes.len(),
        fb_bytes.len(),
        (fb_bytes.len() as f64 - json_bytes.len() as f64) / json_bytes.len() as f64 * 100.0
    );

    let group_name = format!("Envelope_{}", label);
    let mut group = c.benchmark_group(group_name);
    group.throughput(Throughput::Elements(1));

    group.bench_function("encode/json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&legacy)).unwrap())
    });
    group.bench_function("encode/flatbuffers", |b| {
        b.iter(|| encode_message(black_box(&typed)).unwrap())
    });

    group.bench_function("decode/json", |b| {
        b.iter(|| {
            let v: LocalMessagePayloadEnvelope =
                serde_json::from_slice(black_box(&json_bytes)).unwrap();
            v
        })
    });
    group.bench_function("decode/flatbuffers", |b| {
        b.iter(|| {
            let v: MessagePayloadEnvelope = decode_message(black_box(&fb_bytes)).unwrap();
            v
        })
    });

    group.finish();
}

fn bench_envelope_variants(c: &mut Criterion) {
    run_envelope_variant(c, "Text", text_typed(), text_legacy());
    run_envelope_variant(c, "Reply", reply_typed(), reply_legacy());
    run_envelope_variant(c, "Image", image_typed(), image_legacy());
    run_envelope_variant(c, "FullMetadata", full_typed(), full_legacy());
}

// ---------------------------------------------------------------------------
// 2. Full inbound path: SendMessageRequest + nested envelope
//
// This is what `privchat-server` actually does on every received message.
// The SDK uses `MessageSetting`, `client_seq`, `from_uid`, etc. as outer
// fields and stuffs the payload envelope as opaque bytes inside.
// ---------------------------------------------------------------------------

fn make_send_request_with_payload(payload_bytes: Vec<u8>) -> SendMessageRequest {
    SendMessageRequest {
        setting: MessageSetting {
            need_receipt: true,
            signal: 0,
        },
        client_seq: 42,
        local_message_id: 0xDEAD_BEEF_CAFE_BABE,
        stream_no: "stream-1".to_string(),
        channel_id: 10001,
        message_type: ContentMessageType::Text as u32,
        expire: 3600,
        from_uid: 20002,
        topic: "general".to_string(),
        payload: payload_bytes,
    }
}

fn bench_full_inbound(c: &mut Criterion) {
    let typed_env = full_typed();
    let legacy_env = full_legacy();

    // ---- pre-encode each path's wire bytes ----

    // OLD wire: SendMessageRequest is JSON, payload is JSON envelope.
    let old_payload_bytes = serde_json::to_vec(&legacy_env).unwrap();
    let old_send = make_send_request_with_payload(old_payload_bytes.clone());
    let old_wire = serde_json::to_vec(&old_send).unwrap();

    // NEW wire: SendMessageRequest is FB, payload is FB envelope.
    let new_payload_bytes = encode_message(&typed_env).unwrap();
    let new_send = make_send_request_with_payload(new_payload_bytes.clone());
    let new_wire = encode_message(&new_send).unwrap();

    eprintln!(
        "[wire size] full inbound (SendMessageRequest+envelope): \
         JSON={}B, FlatBuffers={}B (Δ {:+.0}%)",
        old_wire.len(),
        new_wire.len(),
        (new_wire.len() as f64 - old_wire.len() as f64) / old_wire.len() as f64 * 100.0
    );

    let mut group = c.benchmark_group("FullInbound_SendMessage_with_Envelope");
    group.throughput(Throughput::Elements(1));

    // ---- DECODE — server hot path ----
    //
    // Old: parse SendMessageRequest from JSON, then parse its JSON payload
    //      into a LocalMessagePayloadEnvelope (Value-typed metadata).
    group.bench_function("decode/json/two_layer", |b| {
        b.iter(|| {
            let req: SendMessageRequest =
                serde_json::from_slice(black_box(&old_wire)).unwrap();
            let env: LocalMessagePayloadEnvelope =
                serde_json::from_slice(&req.payload).unwrap();
            (req, env)
        })
    });
    // New: FB decode SendMessageRequest, then FB decode the payload envelope
    //      to typed enum metadata. No JSON parsing on this path.
    group.bench_function("decode/flatbuffers/two_layer", |b| {
        b.iter(|| {
            let req: SendMessageRequest = decode_message(black_box(&new_wire)).unwrap();
            let env: MessagePayloadEnvelope = decode_message(&req.payload).unwrap();
            (req, env)
        })
    });

    // ---- ENCODE — SDK send path ----
    group.bench_function("encode/json/two_layer", |b| {
        b.iter(|| {
            let payload = serde_json::to_vec(black_box(&legacy_env)).unwrap();
            let req = make_send_request_with_payload(payload);
            serde_json::to_vec(&req).unwrap()
        })
    });
    group.bench_function("encode/flatbuffers/two_layer", |b| {
        b.iter(|| {
            let payload = encode_message(black_box(&typed_env)).unwrap();
            let req = make_send_request_with_payload(payload);
            encode_message(&req).unwrap()
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 3. Bridge cost — SDK output path (NOT server inbound)
//
// Reported separately so the bridge overhead isn't mistaken for server CPU.
// ---------------------------------------------------------------------------

fn bench_bridge(c: &mut Criterion) {
    let legacy = full_legacy();

    let mut group = c.benchmark_group("SDK_Output_Bridge");
    group.throughput(Throughput::Elements(1));

    // Bridge: legacy JSON envelope → typed → FB encode.
    // Mirrors what `privchat-sdk::build_send_message_request` now does.
    group.bench_function("legacy_to_typed_to_fb", |b| {
        b.iter(|| {
            let typed = MessagePayloadEnvelope::from_legacy(
                black_box(&legacy),
                ContentMessageType::Image,
            );
            encode_message(&typed).unwrap()
        })
    });

    // For comparison: pure FB encode (when the SDK is migrated to typed
    // construction directly, no bridge needed).
    let typed = MessagePayloadEnvelope::from_legacy(&legacy, ContentMessageType::Image);
    group.bench_function("typed_to_fb_only", |b| {
        b.iter(|| encode_message(black_box(&typed)).unwrap())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_envelope_variants,
    bench_full_inbound,
    bench_bridge
);
criterion_main!(benches);

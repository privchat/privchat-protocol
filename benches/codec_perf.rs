//! Codec microbench: serde_json vs FlatBuffers, head-to-head.
//!
//! Measures encode/decode + wire size for the three highest-frequency
//! message types in the PrivChat protocol:
//!   - SendMessageRequest (client → server hot path)
//!   - PushMessageRequest (server → client hot path)
//!   - PushBatchRequest with N=10 (server fan-out / catch-up)
//!
//! Both paths use the SAME owned struct; only the wire codec differs.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use privchat_protocol::*;

fn make_send_message() -> SendMessageRequest {
    SendMessageRequest {
        setting: MessageSetting {
            need_receipt: true,
            signal: 0,
        },
        client_seq: 42,
        local_message_id: 0xDEAD_BEEF_CAFE_BABE,
        stream_no: "stream-1".to_string(),
        channel_id: 10001,
        message_type: 0, // ContentMessageType::Text
        expire: 3600,
        from_uid: 20002,
        topic: "general".to_string(),
        // ~120 bytes — typical IM text payload (UTF-8 chinese + ascii mix)
        payload: "你好世界 hello world this is a moderately sized chat message body \
                  that mirrors a realistic IM payload across the wire."
            .as_bytes()
            .to_vec(),
    }
}

fn make_push_message() -> PushMessageRequest {
    PushMessageRequest {
        setting: MessageSetting {
            need_receipt: false,
            signal: 1,
        },
        msg_key: "msg-key-abc-123".to_string(),
        server_message_id: 9_876_543_210,
        message_seq: 100,
        local_message_id: 0xDEAD_BEEF_CAFE_BABE,
        stream_no: "stream-1".to_string(),
        stream_seq: 5,
        stream_flag: 2,
        timestamp: 1_700_000_000,
        channel_id: 10001,
        channel_type: 1,
        message_type: 0, // ContentMessageType::Text
        expire: 3600,
        topic: "general".to_string(),
        from_uid: 20002,
        payload: "你好世界 hello world this is a moderately sized chat message body \
                  that mirrors a realistic IM payload across the wire."
            .as_bytes()
            .to_vec(),
        deleted: false,
    }
}

fn make_push_batch(count: usize) -> PushBatchRequest {
    PushBatchRequest {
        messages: (0..count).map(|_| make_push_message()).collect(),
    }
}

// ---------------------------------------------------------------------------
// SendMessageRequest
// ---------------------------------------------------------------------------

fn bench_send_message(c: &mut Criterion) {
    let msg = make_send_message();

    // Pre-encode for decode bench.
    let json_bytes = serde_json::to_vec(&msg).unwrap();
    let fb_bytes = encode_message(&msg).unwrap();

    eprintln!(
        "[wire size] SendMessageRequest: JSON={}B, FlatBuffers={}B (Δ {:+.0}%)",
        json_bytes.len(),
        fb_bytes.len(),
        (fb_bytes.len() as f64 - json_bytes.len() as f64) / json_bytes.len() as f64 * 100.0
    );

    let mut group = c.benchmark_group("SendMessageRequest");
    group.throughput(Throughput::Elements(1));

    group.bench_function("encode/json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&msg)).unwrap())
    });
    group.bench_function("encode/flatbuffers", |b| {
        b.iter(|| encode_message(black_box(&msg)).unwrap())
    });

    group.bench_function("decode/json", |b| {
        b.iter(|| {
            let v: SendMessageRequest =
                serde_json::from_slice(black_box(&json_bytes)).unwrap();
            v
        })
    });
    group.bench_function("decode/flatbuffers", |b| {
        b.iter(|| {
            let v: SendMessageRequest =
                decode_message(black_box(&fb_bytes)).unwrap();
            v
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// PushMessageRequest
// ---------------------------------------------------------------------------

fn bench_push_message(c: &mut Criterion) {
    let msg = make_push_message();

    let json_bytes = serde_json::to_vec(&msg).unwrap();
    let fb_bytes = encode_message(&msg).unwrap();

    eprintln!(
        "[wire size] PushMessageRequest: JSON={}B, FlatBuffers={}B (Δ {:+.0}%)",
        json_bytes.len(),
        fb_bytes.len(),
        (fb_bytes.len() as f64 - json_bytes.len() as f64) / json_bytes.len() as f64 * 100.0
    );

    let mut group = c.benchmark_group("PushMessageRequest");
    group.throughput(Throughput::Elements(1));

    group.bench_function("encode/json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&msg)).unwrap())
    });
    group.bench_function("encode/flatbuffers", |b| {
        b.iter(|| encode_message(black_box(&msg)).unwrap())
    });

    group.bench_function("decode/json", |b| {
        b.iter(|| {
            let v: PushMessageRequest =
                serde_json::from_slice(black_box(&json_bytes)).unwrap();
            v
        })
    });
    group.bench_function("decode/flatbuffers", |b| {
        b.iter(|| {
            let v: PushMessageRequest =
                decode_message(black_box(&fb_bytes)).unwrap();
            v
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// PushBatchRequest (N=10) — server fan-out / catch-up hot path
// ---------------------------------------------------------------------------

fn bench_push_batch_10(c: &mut Criterion) {
    let msg = make_push_batch(10);

    let json_bytes = serde_json::to_vec(&msg).unwrap();
    let fb_bytes = encode_message(&msg).unwrap();

    eprintln!(
        "[wire size] PushBatchRequest(N=10): JSON={}B, FlatBuffers={}B (Δ {:+.0}%)",
        json_bytes.len(),
        fb_bytes.len(),
        (fb_bytes.len() as f64 - json_bytes.len() as f64) / json_bytes.len() as f64 * 100.0
    );

    let mut group = c.benchmark_group("PushBatchRequest_N10");
    group.throughput(Throughput::Elements(10));

    group.bench_function("encode/json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&msg)).unwrap())
    });
    group.bench_function("encode/flatbuffers", |b| {
        b.iter(|| encode_message(black_box(&msg)).unwrap())
    });

    group.bench_function("decode/json", |b| {
        b.iter(|| {
            let v: PushBatchRequest =
                serde_json::from_slice(black_box(&json_bytes)).unwrap();
            v
        })
    });
    group.bench_function("decode/flatbuffers", |b| {
        b.iter(|| {
            let v: PushBatchRequest =
                decode_message(black_box(&fb_bytes)).unwrap();
            v
        })
    });

    group.finish();
}

criterion_group!(benches, bench_send_message, bench_push_message, bench_push_batch_10);
criterion_main!(benches);

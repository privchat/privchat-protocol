//! 附件加密的契约测试。
//!
//! 要防的是对象存储服务商：桶里那段字节是对方唯一拿得到的东西，
//! 所以这里验的核心是「那段字节泄露不了明文，且改一个比特就解不开」。
//!
//! 分块格式又多出一类攻击面——每块自带 tag，单独看都是合法的。所以下面有一整组
//! 用例专门验：块乱序、跨对象替换、截断、改头，全都必须失败。

use privchat_protocol::attachment_crypto::{
    chunk_count_for, decrypt_attachment, decrypt_downloaded_attachment_bytes, encrypt_attachment,
    encrypt_attachment_with_chunk_size, key_id_of, sealed_len, AttachmentHeader, AttachmentOpener,
    AttachmentSealer, HEADER_LEN,
};

fn key(seed: u8) -> [u8; 32] {
    [seed; 32]
}

/// 小块，让几十字节的测试数据也能真的分成多块。
const SMALL: u32 = 8;

fn seal_small(plain: &[u8], k: &[u8; 32]) -> Vec<u8> {
    encrypt_attachment_with_chunk_size(plain, k, 1, SMALL).expect("encrypt")
}

/// 把 blob 拆成 (header, chunks)，方便下面按块做手术。
fn split_chunks(blob: &[u8]) -> (Vec<u8>, Vec<Vec<u8>>) {
    let header = AttachmentHeader::parse(blob).expect("header");
    let mut rest = &blob[HEADER_LEN..];
    let mut chunks = Vec::new();
    for _ in 0..header.chunk_count {
        let len = u32::from_be_bytes(rest[0..4].try_into().unwrap()) as usize;
        let end = 4 + 16 + len;
        chunks.push(rest[..end].to_vec());
        rest = &rest[end..];
    }
    assert!(rest.is_empty(), "chunk walk must consume the whole blob");
    (blob[..HEADER_LEN].to_vec(), chunks)
}

fn reassemble(header: &[u8], chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut out = header.to_vec();
    for c in chunks {
        out.extend_from_slice(c);
    }
    out
}

#[test]
fn round_trips_across_chunk_boundaries() {
    let k = key(7);
    // 空、单块、正好整除、跨块余数——边界都要过。
    for len in [0usize, 1, 7, 8, 9, 16, 17, 100] {
        let plain: Vec<u8> = (0..len).map(|i| i as u8).collect();
        let blob = seal_small(&plain, &k);
        assert_eq!(decrypt_attachment(&blob, &k).expect("decrypt"), plain, "len {len}");
    }
}

#[test]
fn round_trips_at_the_real_chunk_size() {
    let k = key(7);
    let plain: Vec<u8> = (0..3_000_000).map(|i| (i % 251) as u8).collect();
    let blob = encrypt_attachment(&plain, &k, 1).expect("encrypt");
    assert_eq!(AttachmentHeader::parse(&blob).unwrap().chunk_count, 3);
    assert_eq!(decrypt_attachment(&blob, &k).expect("decrypt"), plain);
}

/// 密文里绝不能出现明文片段——这正是整个方案要防的事。
#[test]
fn the_ciphertext_reveals_nothing_of_the_plaintext() {
    let plain = b"a recognisable marker string";
    let blob = seal_small(plain, &key(7));
    for window in 8..=plain.len() {
        assert!(
            blob.windows(window).all(|w| w != &plain[..window]),
            "明文出现在密文里"
        );
    }
}

/// 同一份明文两次加密必须产出不同密文：salt 与 nonce_prefix 都是随机的。
///
/// 秒传**不**依赖密文相同——第二个人根本不上传（服务端按 dedup_id 命中就直接建引用），
/// 所以密文的不确定性和秒传并不冲突。
#[test]
fn the_same_plaintext_never_produces_the_same_ciphertext() {
    let k = key(7);
    let a = seal_small(b"identical bytes", &k);
    let b = seal_small(b"identical bytes", &k);
    assert_ne!(a, b, "每个对象必须有自己的 salt 与 nonce 前缀");
    assert_eq!(decrypt_attachment(&a, &k).unwrap(), b"identical bytes");
    assert_eq!(decrypt_attachment(&b, &k).unwrap(), b"identical bytes");
}

#[test]
fn a_wrong_key_is_rejected_rather_than_returning_garbage() {
    let blob = seal_small(b"x", &key(7));
    assert!(decrypt_attachment(&blob, &key(8)).is_err());
}

#[test]
fn tampering_with_a_chunk_is_detected() {
    let k = key(7);
    let mut blob = seal_small(b"payload spanning chunks", &k);
    let last = blob.len() - 1;
    blob[last] ^= 0xff;
    assert!(decrypt_attachment(&blob, &k).is_err());
}

/// 🔴 乱序必须被拒。每块自带合法 tag，只有把块序号绑进 AAD 才拦得住——
/// 否则攻击者可以随意重排一份文件的内容而认证依然通过。
#[test]
fn reordering_chunks_is_rejected() {
    let k = key(7);
    let plain: Vec<u8> = (0..32).collect();
    let blob = seal_small(&plain, &k);
    let (header, mut chunks) = split_chunks(&blob);
    assert!(chunks.len() >= 4);
    chunks.swap(0, 1);
    assert!(decrypt_attachment(&reassemble(&header, &chunks), &k).is_err());
}

/// 🔴 跨对象替换必须被拒。两份文件用同一把站点密钥，但 salt 不同 → header 摘要不同
/// → AAD 不同。没有这条绑定，A 的块可以嫁接进 B。
#[test]
fn substituting_a_chunk_from_another_object_is_rejected() {
    let k = key(7);
    let a = seal_small(&(0u8..32).collect::<Vec<_>>(), &k);
    let b = seal_small(&(100u8..132).collect::<Vec<_>>(), &k);
    let (header_a, mut chunks_a) = split_chunks(&a);
    let (_, chunks_b) = split_chunks(&b);
    chunks_a[1] = chunks_b[1].clone();
    assert!(decrypt_attachment(&reassemble(&header_a, &chunks_a), &k).is_err());
}

/// 🔴 截断必须被拒。块数和总长写在受认证的头里，砍掉末块立刻暴露。
#[test]
fn truncating_the_last_chunk_is_rejected() {
    let k = key(7);
    let blob = seal_small(&(0u8..32).collect::<Vec<_>>(), &k);
    let (header, mut chunks) = split_chunks(&blob);
    chunks.pop();
    assert!(decrypt_attachment(&reassemble(&header, &chunks), &k).is_err());
}

/// 流式解密同样不能漏截断：少调一次 open_chunk 就必须在 finish 处报错，
/// 否则调用方拿到一份短了的文件却没有任何错误。
#[test]
fn a_streaming_reader_that_stops_early_fails_at_finish() {
    let k = key(7);
    let blob = seal_small(&(0u8..32).collect::<Vec<_>>(), &k);
    let (_, chunks) = split_chunks(&blob);
    let mut opener = AttachmentOpener::new(&blob, &k).expect("opener");
    opener.open_chunk(&chunks[0]).expect("first chunk opens");
    assert!(opener.finish().is_err(), "提前收尾必须报错");
}

/// 改头的任何一个字节都必须让全部块解不开——头摘要在每块 AAD 里。
#[test]
fn tampering_with_the_header_is_rejected() {
    let k = key(7);
    let blob = seal_small(&(0u8..32).collect::<Vec<_>>(), &k);
    for offset in [3usize, 5, 21, 29] {
        let mut tampered = blob.clone();
        tampered[offset] ^= 0x01;
        assert!(
            decrypt_attachment(&tampered, &k).is_err(),
            "header 第 {offset} 字节被改却仍解得开"
        );
    }
}

/// 头自洽性：块数与总长必须互相印证，改一个而不改另一个要在解密之前就被拒。
#[test]
fn an_inconsistent_header_is_rejected_before_decryption() {
    let k = key(7);
    let mut blob = seal_small(&(0u8..32).collect::<Vec<_>>(), &k);
    blob[32..36].copy_from_slice(&99u32.to_be_bytes()); // chunk_count
    let err = AttachmentHeader::parse(&blob).expect_err("must refuse");
    assert!(err.contains("inconsistent"), "{err}");
    assert!(decrypt_attachment(&blob, &k).is_err());
}

/// key_id 自描述：轮换期两代对象并存，解密方按 blob 自己挑密钥，
/// 不依赖任何服务端字段。
#[test]
fn the_key_id_travels_with_the_object() {
    for id in [0u8, 1, 42, 255] {
        let blob = encrypt_attachment_with_chunk_size(b"x", &key(1), id, SMALL).expect("encrypt");
        assert_eq!(key_id_of(&blob), Some(id));
    }
    assert_eq!(key_id_of(b"short"), None, "非附件 blob 不得误判");
}

/// 密文长度必须能在**拿到密钥之前**算准：上传 token 要签字节数，而密钥随 token
/// 才回来。算错的话 token 签的大小和实际上传的对不上，服务端直接拒。
#[test]
fn the_sealed_size_is_predictable_without_the_key() {
    let k = key(3);
    for len in [0usize, 1, 7, 8, 9, 1024] {
        let blob = seal_small(&vec![0xabu8; len], &k);
        assert_eq!(blob.len() as u64, sealed_len(len as u64, SMALL).unwrap(), "明文 {len} 字节");
    }
    // 真实块大小下的大文件同样要准。
    let big = 1_852_290u64;
    assert_eq!(
        sealed_len(big, 1024 * 1024).unwrap(),
        HEADER_LEN as u64 + 2 * 20 + big
    );
}

/// 空文件也算一块，块数换算不许返回 0——0 块的对象没有任何东西被认证。
#[test]
fn an_empty_attachment_still_has_one_chunk() {
    assert_eq!(chunk_count_for(0, SMALL).unwrap(), 1);
    let k = key(1);
    let blob = seal_small(b"", &k);
    assert_eq!(AttachmentHeader::parse(&blob).unwrap().chunk_count, 1);
    assert_eq!(decrypt_attachment(&blob, &k).unwrap(), b"");
}

/// 解不开必须报错，绝不回落成明文——那会把密文当图片写进缓存，
/// UI 显示坏图，真实错误被藏起来。
#[test]
fn a_failed_decrypt_never_falls_back_to_plaintext() {
    let blob = seal_small(b"x", &key(1));
    assert!(decrypt_downloaded_attachment_bytes(&key(2), &blob).is_err());
}

/// 格式版本必须在头里，且不认识的版本要拒绝而不是硬解。
#[test]
fn an_unknown_format_version_is_refused() {
    let k = key(1);
    let mut blob = seal_small(b"x", &k);
    blob[2] = 0xfe; // 冒充一个未来版本
    assert!(decrypt_attachment(&blob, &k).is_err());
    assert_eq!(key_id_of(&blob), None, "版本不认识就不该报 key_id");
}

/// 分块大小由服务端冻结，客户端不得自选：同一份明文按不同块大小加密会得到不同长度的
/// 密文，token 里签的 sealed_size 就对不上。这里验的是 sealer 不接受短块。
#[test]
fn a_short_middle_chunk_is_refused_by_the_sealer() {
    let mut sealer = AttachmentSealer::new(&key(1), 1, 32, SMALL).expect("sealer");
    let err = sealer.seal_chunk(&[0u8; 3]).expect_err("must refuse");
    assert!(err.contains("exactly"), "{err}");
}

/// 多封一块也要拒——头里声明几块就是几块。
#[test]
fn sealing_more_chunks_than_declared_is_refused() {
    let mut sealer = AttachmentSealer::new(&key(1), 1, 8, SMALL).expect("sealer");
    sealer.seal_chunk(&[0u8; 8]).expect("the only chunk");
    assert!(sealer.is_complete());
    assert!(sealer.seal_chunk(&[]).is_err());
}

/// 密钥长度不对是配置错误，必须显式失败而不是凑合。
#[test]
fn a_key_of_the_wrong_length_is_refused() {
    assert!(AttachmentSealer::new(&[0u8; 16], 1, 8, SMALL).is_err());
    let blob = seal_small(b"x", &key(1));
    assert!(decrypt_attachment(&blob, &[0u8; 16]).is_err());
}

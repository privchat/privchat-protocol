// Copyright 2024 Shanghai Boyu Information Technology Co., Ltd.
// https://privchat.dev
//
// Licensed under the Apache License, Version 2.0 (the "License").

//! 附件加密：分块 AES-256-GCM，密钥由服务端配置、随上传 token 下发。
//!
//! 要防的是**对象存储服务商拿我们存的图片视频去跑训练**——对方对桶里的字节有完整
//! 访问权。所以明文永远不出客户端，桶里只有密文。
//!
//! 冻结设计见 `ATTACHMENT_ENCRYPTION_SPEC`。三条要点：
//!
//! - **分块**：整文件单个 tag 意味着服务端校验一份 500MB 视频得先缓冲整份。
//!   分块之后可以边读边验。
//! - **每块 AAD 绑 `SHA256(header) || chunk_index || plaintext_len`**：
//!   🔴 独立 tag 只能证明某块没被改，证明不了它属于这个文件、在这个位置。
//!   绑上之后乱序、跨对象替换、截断全部变成认证失败。
//! - **按对象派生密钥**：全站密钥直接用会让所有对象共享一个 nonce 空间，规模上来后
//!   碰撞风险是全站级的。HKDF 到每个对象之后，nonce 收敛到单对象内，可以安全地
//!   由块序号派生。
//!
//! **密钥绝不进日志。**

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub const KEY_LEN: usize = 32;
pub const TAG_LEN: usize = 16;
pub const SALT_LEN: usize = 16;
pub const NONCE_PREFIX_LEN: usize = 8;

/// `magic(2) || format_version(1) || key_id(1) || salt(16) || nonce_prefix(8)
///  || chunk_plain_size(4) || chunk_count(4) || plaintext_size(8)`
pub const HEADER_LEN: usize = 44;

/// 每块的定长开销：`plaintext_len(4) || ... || tag(16)`。
const CHUNK_OVERHEAD: usize = 4 + TAG_LEN;

const MAGIC: [u8; 2] = *b"PC";

/// blob 格式版本。
///
/// 🔴 **删掉旧实现 ≠ 删掉格式版本。** 现在只有一种格式，但换算法、改分块、扩头部时，
/// 没有这个字节就无法可靠区分新旧对象——只能靠猜长度或全量重写。
/// 不认识的版本一律拒绝，不得硬解。
pub const FORMAT_VERSION: u8 = 1;

/// 默认块大小 1 MiB。
///
/// 服务端会把实际值冻进上传 token，客户端不得自选——否则同一份明文因分块不同产出
/// 不同长度的密文，token 里签的 `sealed_size` 就对不上。
pub const DEFAULT_CHUNK_PLAIN_SIZE: u32 = 1024 * 1024;

const HKDF_INFO: &[u8] = b"privchat-attachment-object-v1";

/// 空明文也算一块（长度为 0），这样块数与偏移的换算不必到处特判。
pub fn chunk_count_for(plaintext_len: u64, chunk_plain_size: u32) -> Result<u32, String> {
    if chunk_plain_size == 0 {
        return Err("chunk_plain_size must not be zero".to_string());
    }
    let per = chunk_plain_size as u64;
    let count = plaintext_len.div_ceil(per).max(1);
    u32::try_from(count).map_err(|_| "attachment has too many chunks".to_string())
}

/// 给定明文长度，密文的确切字节数。
///
/// 🔴 上传 token 要签下**密文**的字节数，而密钥要等 token 回来才拿得到。
/// 这个函数不碰密钥，所以客户端可以在申请 token 之前把大小算准。
pub fn sealed_len(plaintext_len: u64, chunk_plain_size: u32) -> Result<u64, String> {
    let chunks = chunk_count_for(plaintext_len, chunk_plain_size)? as u64;
    Ok(HEADER_LEN as u64 + chunks * CHUNK_OVERHEAD as u64 + plaintext_len)
}

/// 受认证的文件头。它的所有字段都进每一块的 AAD，改任何一个字节都会让全部块解不开。
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AttachmentHeader {
    pub encryption_key_id: u8,
    pub object_salt: [u8; SALT_LEN],
    pub nonce_prefix: [u8; NONCE_PREFIX_LEN],
    pub chunk_plain_size: u32,
    pub chunk_count: u32,
    pub plaintext_size: u64,
}

/// 🔴 手写 Debug：盐和 nonce 前缀不是秘密，但结构体将来可能加字段，
/// derive 会把新字段自动带进日志。这里只渲染排障真正需要的东西。
impl std::fmt::Debug for AttachmentHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentHeader")
            .field("encryption_key_id", &self.encryption_key_id)
            .field("chunk_plain_size", &self.chunk_plain_size)
            .field("chunk_count", &self.chunk_count)
            .field("plaintext_size", &self.plaintext_size)
            .finish()
    }
}

impl AttachmentHeader {
    pub fn to_bytes(self) -> [u8; HEADER_LEN] {
        let mut out = [0u8; HEADER_LEN];
        out[0..2].copy_from_slice(&MAGIC);
        out[2] = FORMAT_VERSION;
        out[3] = self.encryption_key_id;
        out[4..20].copy_from_slice(&self.object_salt);
        out[20..28].copy_from_slice(&self.nonce_prefix);
        out[28..32].copy_from_slice(&self.chunk_plain_size.to_be_bytes());
        out[32..36].copy_from_slice(&self.chunk_count.to_be_bytes());
        out[36..44].copy_from_slice(&self.plaintext_size.to_be_bytes());
        out
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_LEN {
            return Err(format!(
                "attachment header too short: {} < {HEADER_LEN}",
                bytes.len()
            ));
        }
        if bytes[0..2] != MAGIC {
            return Err("not an encrypted attachment blob".to_string());
        }
        if bytes[2] != FORMAT_VERSION {
            return Err(format!("unsupported attachment format version: {}", bytes[2]));
        }
        let header = Self {
            encryption_key_id: bytes[3],
            object_salt: bytes[4..20].try_into().expect("16 bytes"),
            nonce_prefix: bytes[20..28].try_into().expect("8 bytes"),
            chunk_plain_size: u32::from_be_bytes(bytes[28..32].try_into().expect("4 bytes")),
            chunk_count: u32::from_be_bytes(bytes[32..36].try_into().expect("4 bytes")),
            plaintext_size: u64::from_be_bytes(bytes[36..44].try_into().expect("8 bytes")),
        };
        // 头自洽性先于解密检查：块数和总长必须互相印证，否则截断攻击要等到
        // 最后一块缺失才暴露，而那时调用方可能已经把前面的明文交出去了。
        let expected = chunk_count_for(header.plaintext_size, header.chunk_plain_size)?;
        if expected != header.chunk_count {
            return Err(format!(
                "attachment header is inconsistent: {} chunks declared, {expected} implied",
                header.chunk_count
            ));
        }
        Ok(header)
    }

    fn digest(self) -> [u8; 32] {
        Sha256::digest(self.to_bytes()).into()
    }
}

/// 读出 blob 声明的 key_id，用于挑密钥（轮换期两代对象并存）。
pub fn key_id_of(blob: &[u8]) -> Option<u8> {
    AttachmentHeader::parse(blob).ok().map(|h| h.encryption_key_id)
}

/// 按对象派生密钥。同一把站点密钥下，不同 salt 得到互不相关的对象密钥。
fn derive_object_key(site_key: &[u8], salt: &[u8; SALT_LEN]) -> Result<[u8; KEY_LEN], String> {
    if site_key.len() != KEY_LEN {
        return Err(format!(
            "key must be {KEY_LEN} bytes, got {}",
            site_key.len()
        ));
    }
    let mut out = [0u8; KEY_LEN];
    Hkdf::<Sha256>::new(Some(salt), site_key)
        .expand(HKDF_INFO, &mut out)
        .map_err(|_| "attachment key derivation failed".to_string())?;
    Ok(out)
}

fn chunk_nonce(prefix: &[u8; NONCE_PREFIX_LEN], index: u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[0..8].copy_from_slice(prefix);
    nonce[8..12].copy_from_slice(&index.to_be_bytes());
    nonce
}

fn chunk_aad(header_digest: &[u8; 32], index: u32, plaintext_len: u32) -> Vec<u8> {
    let mut aad = Vec::with_capacity(40);
    aad.extend_from_slice(header_digest);
    aad.extend_from_slice(&index.to_be_bytes());
    aad.extend_from_slice(&plaintext_len.to_be_bytes());
    aad
}

/// 逐块加密器。分块存在的意义就是不必把整份文件握在手里，所以对外是流式的。
pub struct AttachmentSealer {
    header: AttachmentHeader,
    header_digest: [u8; 32],
    cipher: Aes256Gcm,
    next_index: u32,
}

impl AttachmentSealer {
    /// `site_key` / `key_id` 来自上传 token；`chunk_plain_size` 也由服务端冻结。
    pub fn new(
        site_key: &[u8],
        key_id: u8,
        plaintext_size: u64,
        chunk_plain_size: u32,
    ) -> Result<Self, String> {
        let mut object_salt = [0u8; SALT_LEN];
        let mut nonce_prefix = [0u8; NONCE_PREFIX_LEN];
        rand::thread_rng().fill_bytes(&mut object_salt);
        rand::thread_rng().fill_bytes(&mut nonce_prefix);

        let header = AttachmentHeader {
            encryption_key_id: key_id,
            object_salt,
            nonce_prefix,
            chunk_plain_size,
            chunk_count: chunk_count_for(plaintext_size, chunk_plain_size)?,
            plaintext_size,
        };
        let object_key = derive_object_key(site_key, &object_salt)?;
        Ok(Self {
            header,
            header_digest: header.digest(),
            cipher: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&object_key)),
            next_index: 0,
        })
    }

    pub fn header(&self) -> AttachmentHeader {
        self.header
    }

    pub fn header_bytes(&self) -> [u8; HEADER_LEN] {
        self.header.to_bytes()
    }

    /// 封一块。除最后一块外，`plaintext` 必须正好是 `chunk_plain_size` 字节——
    /// 短块会让后续块的明文偏移算错，而 AAD 里没有偏移可以兜住这个错误。
    pub fn seal_chunk(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let index = self.next_index;
        if index >= self.header.chunk_count {
            return Err("attachment already has all declared chunks".to_string());
        }
        let is_last = index + 1 == self.header.chunk_count;
        if !is_last && plaintext.len() != self.header.chunk_plain_size as usize {
            return Err(format!(
                "chunk {index} must be exactly {} bytes, got {}",
                self.header.chunk_plain_size,
                plaintext.len()
            ));
        }
        if plaintext.len() > self.header.chunk_plain_size as usize {
            return Err("chunk exceeds the declared chunk size".to_string());
        }
        let plaintext_len = plaintext.len() as u32;
        let aad = chunk_aad(&self.header_digest, index, plaintext_len);
        let sealed = self
            .cipher
            .encrypt(
                Nonce::from_slice(&chunk_nonce(&self.header.nonce_prefix, index)),
                Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|_| "attachment encrypt failed".to_string())?;

        self.next_index += 1;
        let mut out = Vec::with_capacity(4 + sealed.len());
        out.extend_from_slice(&plaintext_len.to_be_bytes());
        out.extend_from_slice(&sealed);
        Ok(out)
    }

    /// 所有声明的块都封完了吗。少一块就上传等于故意截断，必须自己先拦住。
    pub fn is_complete(&self) -> bool {
        self.next_index == self.header.chunk_count
    }
}

/// 逐块解密器。服务端首传校验和客户端下载都走它。
pub struct AttachmentOpener {
    header: AttachmentHeader,
    header_digest: [u8; 32],
    cipher: Aes256Gcm,
    next_index: u32,
}

impl AttachmentOpener {
    pub fn new(header_bytes: &[u8], site_key: &[u8]) -> Result<Self, String> {
        let header = AttachmentHeader::parse(header_bytes)?;
        let object_key = derive_object_key(site_key, &header.object_salt)?;
        Ok(Self {
            header,
            header_digest: header.digest(),
            cipher: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&object_key)),
            next_index: 0,
        })
    }

    pub fn header(&self) -> AttachmentHeader {
        self.header
    }

    /// 解开下一块。`sealed` 是 `plaintext_len(4) || ct || tag`。
    ///
    /// 序号由解密器自己数，不从数据里读——读来的序号可以被改，数出来的不行。
    pub fn open_chunk(&mut self, sealed: &[u8]) -> Result<Vec<u8>, String> {
        let index = self.next_index;
        if index >= self.header.chunk_count {
            return Err("attachment has more chunks than its header declares".to_string());
        }
        if sealed.len() < CHUNK_OVERHEAD {
            return Err(format!("attachment chunk {index} is truncated"));
        }
        let plaintext_len = u32::from_be_bytes(sealed[0..4].try_into().expect("4 bytes"));
        if plaintext_len > self.header.chunk_plain_size {
            return Err(format!("attachment chunk {index} declares an oversized length"));
        }
        if sealed.len() != CHUNK_OVERHEAD + plaintext_len as usize {
            return Err(format!("attachment chunk {index} has a mismatched length"));
        }
        let aad = chunk_aad(&self.header_digest, index, plaintext_len);
        let plaintext = self
            .cipher
            .decrypt(
                Nonce::from_slice(&chunk_nonce(&self.header.nonce_prefix, index)),
                Payload {
                    msg: &sealed[4..],
                    aad: &aad,
                },
            )
            .map_err(|_| "attachment decrypt/auth failed".to_string())?;

        self.next_index += 1;
        Ok(plaintext)
    }

    /// 🔴 收尾必须显式调用。少了它，截断（丢掉末尾若干块）会一路"成功"，
    /// 调用方拿到的是一份短了的文件却没有任何错误。
    pub fn finish(self) -> Result<(), String> {
        if self.next_index != self.header.chunk_count {
            return Err(format!(
                "attachment is truncated: {} of {} chunks",
                self.next_index, self.header.chunk_count
            ));
        }
        Ok(())
    }
}

/// 整份加密（缩略图、小图这类一次性就装得下的内容）。
pub fn encrypt_attachment(plaintext: &[u8], site_key: &[u8], key_id: u8) -> Result<Vec<u8>, String> {
    encrypt_attachment_with_chunk_size(plaintext, site_key, key_id, DEFAULT_CHUNK_PLAIN_SIZE)
}

pub fn encrypt_attachment_with_chunk_size(
    plaintext: &[u8],
    site_key: &[u8],
    key_id: u8,
    chunk_plain_size: u32,
) -> Result<Vec<u8>, String> {
    let mut sealer = AttachmentSealer::new(
        site_key,
        key_id,
        plaintext.len() as u64,
        chunk_plain_size,
    )?;
    let mut out = Vec::with_capacity(
        sealed_len(plaintext.len() as u64, chunk_plain_size)? as usize,
    );
    out.extend_from_slice(&sealer.header_bytes());
    if plaintext.is_empty() {
        out.extend_from_slice(&sealer.seal_chunk(&[])?);
    } else {
        for part in plaintext.chunks(chunk_plain_size as usize) {
            out.extend_from_slice(&sealer.seal_chunk(part)?);
        }
    }
    debug_assert!(sealer.is_complete());
    Ok(out)
}

/// 整份解密。任一块认证失败、头被改、块缺失都返回 Err。
pub fn decrypt_attachment(blob: &[u8], site_key: &[u8]) -> Result<Vec<u8>, String> {
    let mut opener = AttachmentOpener::new(blob, site_key)?;
    let header = opener.header();
    let mut out = Vec::with_capacity(header.plaintext_size as usize);
    let mut rest = &blob[HEADER_LEN..];
    for _ in 0..header.chunk_count {
        if rest.len() < 4 {
            return Err("attachment is truncated".to_string());
        }
        let plaintext_len = u32::from_be_bytes(rest[0..4].try_into().expect("4 bytes")) as usize;
        let end = CHUNK_OVERHEAD
            .checked_add(plaintext_len)
            .filter(|end| *end <= rest.len())
            .ok_or_else(|| "attachment is truncated".to_string())?;
        out.extend_from_slice(&opener.open_chunk(&rest[..end])?);
        rest = &rest[end..];
    }
    if !rest.is_empty() {
        return Err("attachment has trailing bytes after its last chunk".to_string());
    }
    opener.finish()?;
    if out.len() as u64 != header.plaintext_size {
        return Err("attachment length does not match its header".to_string());
    }
    Ok(out)
}

/// 下载完成后把字节还原成明文（正文与缩略图统一走这里）。
///
/// 🔴 **缺密钥是错误，不是「那就当明文」。** 用「没有密钥」暗示明文是 fail-open：
/// 一次配置遗漏就会让新附件静默地以明文上传，而且没人会发现——桶里的东西看起来
/// 一切正常。公开资源（头像等）要走显式的 `business_type=PUBLIC` 分流，
/// 不能靠「没有密钥」来表达。
pub fn decrypt_downloaded_attachment_bytes(
    site_key: &[u8],
    blob: &[u8],
) -> Result<Vec<u8>, String> {
    decrypt_attachment(blob, site_key)
}

/// 服务端下发的密钥是 base64url(no-pad) 的 32 字节。
pub fn decode_site_key(encoded: &str) -> Result<[u8; KEY_LEN], String> {
    use base64::Engine as _;
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded.trim())
        .map_err(|_| "attachment key is not valid base64url(no-pad)".to_string())?;
    <[u8; KEY_LEN]>::try_from(raw.as_slice())
        .map_err(|_| format!("attachment key must be {KEY_LEN} bytes"))
}

/// 这段字节是不是本格式的密文。
///
/// 判据是自描述的文件头，不依赖任何服务端字段——票据里少一个 flag 就把密文当图片
/// 写进缓存，是这条路径上最容易犯的错。
pub fn looks_like_attachment(bytes: &[u8]) -> bool {
    AttachmentHeader::parse(bytes).is_ok()
}

/// 下载到的字节 → 明文。
///
/// 明文对象（`business_type=PUBLIC` 的头像等）原样返回；密文对象必须有密钥，
/// 🔴 **没有就报错，绝不把密文当内容交出去**——那会写进缓存、UI 渲染坏图，
/// 真正的错误反而被藏起来。
pub fn open_downloaded_bytes(bytes: Vec<u8>, site_key: Option<&str>) -> Result<Vec<u8>, String> {
    if !looks_like_attachment(&bytes) {
        return Ok(bytes);
    }
    let encoded = site_key.ok_or_else(|| {
        "attachment is encrypted but the download ticket carries no key".to_string()
    })?;
    decrypt_attachment(&bytes, &decode_site_key(encoded)?)
}

// 单测见 tests/attachment_crypto_test.rs（集成测试，绕开 lib 内不相关的 test fixture）。

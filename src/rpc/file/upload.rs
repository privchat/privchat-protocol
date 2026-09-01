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

/// 文件上传相关 RPC
use serde::{Deserialize, Serialize};

/// 请求上传令牌请求
///
/// RPC路由: `file/request_upload_token`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRequestUploadTokenRequest {
    /// 用户ID
    pub user_id: u64,
    /// 文件名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// **最终待上传 blob** 的字节数（加密后的长度，不是明文长度）。
    ///
    /// 与 `sha256` 同一口径：服务端按收到的字节数入库并比对。
    pub file_size: i64,
    /// 文件MIME类型
    pub mime_type: String,
    /// 文件类型 (image/video/audio/file/other)
    pub file_type: String,
    /// 业务类型 (message/avatar/group_avatar等)
    pub business_type: String,
    /// **最终待上传 blob** 的 SHA-256（十六进制，64 字符）。
    ///
    /// 「最终待上传 blob」= 压缩/转码之后、**并且已经加密之后**，真正要发给服务端的
    /// 那串字节。去重的单位就是它：服务端不理解加密，只比对收到的字节。
    ///
    /// 由此推出两条客户端硬约束：
    ///   · 预检之后**不能重新加密**——随机 CEK/nonce 会产出另一串字节，
    ///     本来也不该命中；必须上传当初参与哈希的那个 blob。
    ///   · 重试同样要复用同一个 blob（连同它的 CEK 和 nonce），否则每次重试
    ///     都变成一个新的物理文件。
    ///
    /// 所以「同一明文加密两次」是**两个**物理文件，这是预期行为，不是缺陷。
    ///
    /// 不带这个字段 = 老客户端，照常走完整上传。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// 产出这份字节的客户端处理版本；0 / 缺省 = 原始未处理。
    ///
    /// 仅作元数据，**不参与**秒传判定：身份只看内容摘要。字节不同摘要自然不同，
    /// 字节相同就该复用——因为压缩器版本号不同而重复存一份，是白占存储。
    #[serde(default)]
    pub transform_version: i32,
}

/// 上传回调请求
///
/// RPC路由: `file/upload_callback`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadCallbackRequest {
    /// 文件ID
    pub file_id: String,
    /// 用户ID
    pub user_id: u64,
    /// 上传状态
    pub status: String,
}

/// 请求上传令牌响应
///
/// RPC路由: `file/request_upload_token`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRequestUploadTokenResponse {
    /// 标准字段
    pub token: String,
    pub upload_url: String,
    /// 历史兼容字段：部分服务端未返回 file_id，默认空字符串
    #[serde(default)]
    pub file_id: String,
    /// 服务端已经有这份内容：**不必上传字节**。
    ///
    /// 🔴 这只是**告知**，本次调用不产生任何句柄。要拿到自己的 `file_id`，
    /// 带这个 token 和 sha256 去调 `file/claim_existing`。
    ///
    /// 探测与取得所有权必须分开：探测会被重试，合在一起的话每重试一次
    /// 就多给调用方一份文件记录，攒出一堆没有任何消息使用的孤儿句柄。
    #[serde(default)]
    pub already_exists: bool,
    /// 可选：token 过期时间（Unix 秒）
    #[serde(default)]
    pub expires_at: Option<i64>,
    /// 可选：允许上传的最大大小（bytes）
    #[serde(default)]
    pub max_size: Option<i64>,
    /// 服务端下发的分片方案。
    ///
    /// 🔴 **有就分片、没有就整包，客户端不自己判断。** 阈值与网格只活在服务端一处，
    /// 调整不用发版；关停分片 = 恒不下发。同一份方案也被签进 token，服务端按它校验
    /// 每一个分片请求——客户端拿到的这份只是**同一件事的可读副本**，不是另一个真源。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_plan: Option<UploadPlanDto>,
    /// 本次上传该用的全站加密密钥（v2）。
    ///
    /// 客户端用它加密后再上传：服务端和对象存储自始至终只见到密文。
    /// `None` = 服务端没有配置附件密钥，这个对象以明文存储。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment_key: Option<AttachmentKey>,
}

/// 分片方案（与服务端 `UploadPlan` 同构）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadPlanDto {
    /// 区间寻址网格：offset 必须是它的整数倍，非末段长度也必须整格。
    pub base_unit: u32,
    /// 首个探测请求的大小——先用它测一次真实吞吐，再决定后面发多大。
    pub initial_request_size: u32,
    /// 单次请求上限。
    pub max_request_size: u32,
    /// 小于等于此值不值得建会话，直接整包传。
    pub session_threshold: u64,
    /// 并发上限。
    pub max_parallel_parts: u8,
}

/// 请求**分片**上传令牌（RESUMABLE_UPLOAD_SPEC §2）。
///
/// RPC 路由: `file/request_chunked_upload_token`
///
/// 🔴 与 `file/request_upload_token` 是两个接口，不靠响应里"有没有某个字段"暗示走哪条路：
/// 调了分片接口就是要分片，调不通直接报错，不会静默退化成整包。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRequestChunkedUploadTokenRequest {
    /// 文件类型 (image/video/audio/file/other)
    pub file_type: String,
    /// 业务类型 (message/avatar/...)。整包路径里它来自 token，这里同样在申请时冻结。
    pub business_type: String,
    /// **封装后**字节数。
    pub file_size: i64,
    /// **封装后** SHA-256（十六进制 64 字符）。分片路径**必带**：complete 靠它核验。
    pub file_hash: String,
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default)]
    pub transform_version: i32,
    /// `true` = 跳过秒传预检直接建会话。只在「claim 失败且判定可退回」时置一次，
    /// 否则预检命中→claim 失败→重新申请→又命中，永远进不了实体上传。
    #[serde(default)]
    pub force_upload: bool,
    /// 可选：客户端支持的上传数据面（RESUMABLE_UPLOAD_SPEC §8.2，纯加法）。
    /// 取值如 `["proxy_offset_v1", "s3_multipart_v1"]`。🔴 旧客户端不带该字段 →
    /// 服务端行为与响应逐字节不变；不带或不含 `s3_multipart_v1` 时恒走现有
    /// `proxy_offset_v1`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_upload_transports: Option<Vec<String>>,
}

/// 分片上传令牌响应。
///
/// 两种形态互斥：`already_exists=true` 时只有 `claim_token`（拿去调
/// `file/claim_existing`），否则只有 `upload_token` / `upload_url` / `base_unit` /
/// `expires_at`。协商字段（`transport` / `part_size` / `total_parts`，RESUMABLE §8.2）
/// 只在客户端声明了 `supported_upload_transports` 时才可能出现；旧客户端的响应
/// 逐字节不变。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileRequestChunkedUploadTokenResponse {
    #[serde(default)]
    pub already_exists: bool,
    /// 秒传命中时的取用凭据。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_token: Option<String>,
    /// 分片会话凭据 `{upload_id}.{secret}`；chunk/status/complete/abort 只认它。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_token: Option<String>,
    /// 分片端点的基址（`.../files`）。客户端拼 `/chunk`、`/status`、`/complete`、`/abort`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_url: Option<String>,
    /// 寻址网格（字节）：非末段 offset 与 length 必须对齐它。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_unit: Option<u32>,
    /// token 过期时间（Unix 秒）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// 协商结果：本次会话的数据面（RESUMABLE_UPLOAD_SPEC §8.2）。当前只有
    /// `proxy_offset_v1`；选为 `s3_multipart_v1` 时另带 `part_size` / `total_parts`。
    /// 仅当客户端声明了 `supported_upload_transports` 时下发。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
    /// 仅 `s3_multipart_v1`：固定分片大小（字节）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_size: Option<u64>,
    /// 仅 `s3_multipart_v1`：总分片数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_parts: Option<u32>,
    /// 本次上传该用的全站加密密钥（v2）。
    ///
    /// 客户端用它加密后再上传：服务端和对象存储自始至终只见到密文。
    /// `None` = 服务端没有配置附件密钥，这个对象以明文存储。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment_key: Option<AttachmentKey>,
}

/// 获取文件 URL 请求
///
/// RPC路由: `file/get_url`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileGetUrlRequest {
    pub file_id: u64,
    /// 用户ID（服务器端填充，客户端不可设置）
    #[serde(skip_deserializing, default)]
    pub user_id: u64,
}

/// 附件加密密钥（v2 全站统一密钥，ATTACHMENT_ENCRYPTION_SPEC §0.1）。
///
/// 🔴 只在**已鉴权**的响应里出现，绝不进 URL、不进日志。
/// 威胁模型是对象存储服务商：明文和密钥都不能到服务端或桶里，所以密钥经由我们
/// 自己的接口下发给客户端，加解密只在客户端做。
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttachmentKey {
    /// 与密文 blob 头部的 key_id 对应；解密方按它挑密钥。
    pub key_id: u8,
    /// base64url(no-pad) 的 32 字节密钥。
    pub key: String,
}

/// 手写 `Debug`，密钥渲染成 `[REDACTED]`。
///
/// 「目前没有主动打印」不构成保证——一次 `{:?}` 的响应 dump、一条 panic 消息，
/// 密钥就进日志了。同一个坑在 `QuicServerConfig` 上已经踩过一次。
impl std::fmt::Debug for AttachmentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentKey")
            .field("key_id", &self.key_id)
            .field("key", &"[REDACTED]")
            .finish()
    }
}

/// 获取文件 URL 响应
///
/// RPC路由: `file/get_url`
#[derive(Debug, Clone, Serialize, Deserialize)]


pub struct FileGetUrlResponse {
    pub file_url: String,
    pub expires_at: i64,
    pub file_size: u64,
    pub mime_type: String,
    /// 原始文件名（file 表数据，Scheme B：filename/size/mime 均由 get_url 下发，
    /// 不进消息 typed metadata）。缺省=空串。
    #[serde(default)]
    pub original_filename: String,
    /// v2：解开**这一个**附件所需的密钥。
    ///
    /// 🔴 只给这个文件用的那把，不给全量密钥表——服务端按文件行上记录的
    /// `encryption_key_id` 取出对应密钥。下发全量意味着任何拿到一个附件的人
    /// 就获得了全部历史对象的解密能力。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment_key: Option<AttachmentKey>,
    /// 服务端对**已存储字节**算出的 SHA-256。
    ///
    /// 转发一份已有附件时用它：客户端直接拿这个摘要去 prepare + claim，
    /// 不必把文件下下来重算，也不必重新加密（重新加密会产出另一串字节，
    /// 那本来就是另一个物理文件）。老记录没有可用摘要时为空。
    #[serde(default)]
    pub sha256: Option<String>,
    /// 服务端记录的真实文件类型：`image` / `video` / `voice` / `file` / `other`。
    ///
    /// 复用一份已有附件时按它申请 token。**不要靠 mime 推**：`audio/mp3` 可能是
    /// 用户当普通文件发的一首歌而不是语音条，`video/mp4` 同理；推导表还会在
    /// 每个客户端各存一份，迟早分叉。老服务端不下发时为空，客户端才回退推导。
    #[serde(default)]
    pub file_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 🔴 旧客户端兼容门禁（RESUMABLE_UPLOAD_SPEC §8.2）：旧请求 JSON 不带
    /// `supported_upload_transports` 必须能照常反序列化；旧响应的序列化结果与
    /// 新增字段前的 wire shape **逐字节一致**（fixture 直接断言完整字符串）。
    #[test]
    fn legacy_chunked_token_wire_shape_is_unchanged() {
        let legacy_request = serde_json::json!({
            "file_type": "image",
            "business_type": "message",
            "file_size": 1048576,
            "file_hash": "ab".repeat(32),
            "mime_type": "image/jpeg"
        });
        let req: FileRequestChunkedUploadTokenRequest =
            serde_json::from_value(legacy_request).unwrap();
        assert!(req.supported_upload_transports.is_none());

        // 未命中秒传的旧响应：逐字节 fixture。
        let response = FileRequestChunkedUploadTokenResponse {
            already_exists: false,
            upload_token: Some("u.s".to_string()),
            upload_url: Some("http://x/files".to_string()),
            base_unit: Some(65536),
            expires_at: Some(100),
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_string(&response).unwrap(),
            r#"{"already_exists":false,"upload_token":"u.s","upload_url":"http://x/files","base_unit":65536,"expires_at":100}"#
        );

        // 不下发密钥时字段整个不出现；但一旦有密钥就必须落到线上——
        // `skip_serializing_if` 写错的话客户端拿不到密钥，而上面那条断言照样过。
        let with_key = FileRequestChunkedUploadTokenResponse {
            already_exists: false,
            upload_token: Some("u.s".to_string()),
            attachment_key: Some(AttachmentKey {
                key_id: 3,
                key: "k".to_string(),
            }),
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_string(&with_key).unwrap(),
            r#"{"already_exists":false,"upload_token":"u.s","attachment_key":{"key_id":3,"key":"k"}}"#
        );

        // 秒传命中的旧响应：逐字节 fixture。
        let claim = FileRequestChunkedUploadTokenResponse {
            already_exists: true,
            claim_token: Some("claim-1".to_string()),
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_string(&claim).unwrap(),
            r#"{"already_exists":true,"claim_token":"claim-1"}"#
        );

        // 新客户端声明能力 → 响应才带 transport；part_size/total_parts 仍不出现。
        let new_response = FileRequestChunkedUploadTokenResponse {
            transport: Some("proxy_offset_v1".to_string()),
            ..response
        };
        assert_eq!(
            serde_json::to_string(&new_response).unwrap(),
            r#"{"already_exists":false,"upload_token":"u.s","upload_url":"http://x/files","base_unit":65536,"expires_at":100,"transport":"proxy_offset_v1"}"#
        );
    }
}

/// 上传回调响应
///
/// RPC路由: `file/upload_callback`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type FileUploadCallbackResponse = bool;

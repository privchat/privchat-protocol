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
    /// **明文**字节数（压缩/转码之后、加密之前的那份内容的长度）。
    ///
    /// 🔴 不是密文长度。密文长度由服务端按
    /// `sealed_len(plaintext_size, chunk_plain_size)` 算出来并签进 token——客户端
    /// 算不出它，因为分块几何是服务端冻结的。
    pub plaintext_size: i64,
    /// 文件MIME类型
    pub mime_type: String,
    /// 文件类型 (image/video/audio/file/other)
    pub file_type: String,
    /// 业务类型 (message/avatar/group_avatar等)
    pub business_type: String,
    /// **明文**的 SHA-256（十六进制，64 字符）：压缩/转码之后、**加密之前**。
    ///
    /// 🔴 这是跨用户秒传的**唯一**判重键，而且必须是明文摘要。
    ///
    /// 密文摘要不行：全站密钥 + 每块随机 nonce，同一份明文每次封装都产出不同的
    /// 密文——按密文判重等于秒传只对"自己重发自己"生效，而秒传的收益几乎全在
    /// "别人已经传过"。
    ///
    /// 🔴 申请 token 时**不提交任何密文摘要**。此刻客户端还没拿到服务端下发的
    /// 全站密钥，封装不出最终字节，也就无从算起；服务端会在 complete 时流式回读、
    /// 解密重算，那才是权威判据。
    ///
    /// 不带这个字段 = 不参与秒传预检，照常走完整上传。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plaintext_sha256: Option<String>,
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
    /// 带这个 token 和明文摘要去调 `file/claim_existing`。
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
    /// 本次上传该用的全站加密密钥（`attachment_crypto::FORMAT_VERSION` 当前口径）。
    ///
    /// 客户端用它加密后再上传：**桶里只存密文**。
    ///
    /// 🔴 但服务端不是"自始至终只见密文"：complete 时它会回读并解密，重算明文摘要
    /// 来核对 token 冻结的身份（跨用户秒传的判重键就是明文摘要）。解密只在校验那一趟，
    /// 明文不落盘。
    ///
    /// 🔴 这把是**全站**密钥，不是 per-file key：同 key_id 的对象共用它。文件级的
    /// 访问隔离来自私有桶、短期 URL 与 `file/get_url` 的鉴权，不来自密钥本身。
    ///
    /// 🔴 **不存在"没有密钥就明文存储"这条路。** 服务端没有配置附件密钥时，
    /// 签发直接失败（`freeze_crypto` fail-closed）——"没配就当明文"是 fail-open：
    /// 一次配置遗漏会让全部新附件明文进桶，而桶里看起来一切正常。
    /// 所以走到客户端手上的响应必然带着密钥，`None` 只出现在不需要加密的旧路径上。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment_key: Option<AttachmentKey>,
    /// 服务端冻结的**分块几何**：客户端必须**原样**用它封装。
    ///
    /// 🔴 客户端不能自选。同一份明文按不同块大小封装，密文长度不同、每块边界也不同——
    /// 而 token 里冻结的密文长度是按服务端这个值算的，用别的值封出来的对象在
    /// complete 的长度核对上必然被拒。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_plain_size: Option<u32>,
    /// 封装之后的**密文**总字节数 = `sealed_len(plaintext_size, chunk_plain_size)`。
    ///
    /// 🔴 这是真正要传的字节数，也是分片几何/区间网格的基准。客户端自己算不出它
    /// （块大小是服务端定的），所以由服务端下发；上传前用它核对一下自己封出来的
    /// 长度，对不上就是两边几何不一致，早失败好过传完再被拒。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_size: Option<u64>,
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
    /// **明文**字节数。口径同整包，见 [`FileRequestUploadTokenRequest::plaintext_size`]。
    ///
    /// 🔴 分片几何（`part_size` / `total_parts` / 区间网格）全部按服务端算出的
    /// **密文**长度来定，不按这个值——响应里的 `total_size` 才是要传的字节数。
    pub plaintext_size: i64,
    /// **明文** SHA-256（十六进制 64 字符）。分片路径必带：它是秒传判重键，
    /// 也是 complete 解密重算之后要核对的身份。
    pub plaintext_sha256: String,
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
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
    /// 本次上传该用的全站加密密钥。口径同整包，见
    /// [`FileRequestUploadTokenResponse::attachment_key`]——**不存在"没密钥就明文"**。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment_key: Option<AttachmentKey>,
    /// 服务端冻结的**分块几何**：客户端必须**原样**用它封装。
    ///
    /// 🔴 客户端不能自选。同一份明文按不同块大小封装，密文长度不同、每块边界也不同——
    /// 而 token 里冻结的密文长度是按服务端这个值算的，用别的值封出来的对象在
    /// complete 的长度核对上必然被拒。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_plain_size: Option<u32>,
    /// 封装之后的**密文**总字节数 = `sealed_len(plaintext_size, chunk_plain_size)`。
    ///
    /// 🔴 这是真正要传的字节数，也是分片几何/区间网格的基准。客户端自己算不出它
    /// （块大小是服务端定的），所以由服务端下发；上传前用它核对一下自己封出来的
    /// 长度，对不上就是两边几何不一致，早失败好过传完再被拒。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_size: Option<u64>,
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
    /// 解开这个附件要用的密钥：对象记录的 `encryption_key_id` 对应的那把。
    ///
    /// 🔴 **这是全站密钥，不是 per-file key。** 同一个 key_id 下的所有对象共用它，
    /// 所以"拿到这把钥匙"在密码学上等于"能解开那一代的全部对象"。文件级的访问
    /// 隔离来自私有桶、短期 URL 和 `get_url` 的鉴权，不来自密钥本身——把它当成
    /// 文件专属密钥去设计上层逻辑会得出错误的安全结论。
    ///
    /// 服务端只下发这一把、不下发全量密钥表：暴露面因此限制在**一代**密钥上，
    /// 轮换之后旧对象不会跟着泄。这是纵深防御的一层，不是隔离本身。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment_key: Option<AttachmentKey>,
    /// 这份附件的**明文** SHA-256。
    ///
    /// 转发一份已有附件时用它：客户端直接拿这个摘要去 prepare + claim，
    /// 不必把文件下下来重算，也不必重新加密。
    ///
    /// 🔴 是明文摘要，**不是密文摘要**。判重键就是明文摘要：同一份内容由不同人
    /// （或同一个人两次）封装会得到不同的密文，密文摘要按定义无法跨用户命中。
    /// 这里给密文摘要的话，转发会稳定地秒传不中——每转发一次就重传一份。
    ///
    /// 老记录没有可用摘要时为空。
    #[serde(default)]
    pub plaintext_sha256: Option<String>,
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
            "plaintext_size": 1048576,
            "plaintext_sha256": "ab".repeat(32),
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

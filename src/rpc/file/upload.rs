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
    /// 附件加密版本：0=明文 legacy；1=AES-256-GCM（客户端解密）。缺省=0。
    #[serde(default)]
    pub encryption_version: i32,
    /// CEK（base64url 32B）；nonce 在密文 blob 头部。version=0 时 None。绝不进 URL/日志。
    #[serde(default)]
    pub cek: Option<String>,
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

/// 上传回调响应
///
/// RPC路由: `file/upload_callback`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type FileUploadCallbackResponse = bool;

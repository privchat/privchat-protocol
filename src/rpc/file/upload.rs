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
    /// 文件大小（字节）
    pub file_size: i64,
    /// 文件MIME类型
    pub mime_type: String,
    /// 文件类型 (image/video/audio/file/other)
    pub file_type: String,
    /// 业务类型 (message/avatar/group_avatar等)
    pub business_type: String,
    /// 最终内容的 SHA-256（十六进制，64 字符）。
    ///
    /// 「最终」= **压缩/转码之后、加密之前**。客户端第二次发同一份媒体时若再压一遍，
    /// 字节会变、摘要会变，秒传就永远命中不了——所以首次处理完就要把这份字节
    /// 连同摘要一起留住，之后任何再发都直接用它。
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
}

/// 上传回调响应
///
/// RPC路由: `file/upload_callback`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type FileUploadCallbackResponse = bool;

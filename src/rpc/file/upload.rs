// Copyright 2025 Shanghai Boyu Information Technology Co., Ltd.
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
    pub token: String,
    pub upload_url: String,
    pub file_id: String,
}

/// 上传回调响应
///
/// RPC路由: `file/upload_callback`
/// 简单操作，返回 true（成功/失败由协议层 code 处理）
pub type FileUploadCallbackResponse = bool;

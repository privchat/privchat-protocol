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

/// 表情包相关 RPC
use serde::{Deserialize, Serialize};

/// 获取表情包列表请求
///
/// RPC路由: `sticker/package/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerPackageListRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerInfo {
    pub sticker_id: String,
    pub package_id: String,
    pub image_url: String,
    pub alt_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerPackageInfo {
    pub package_id: String,
    pub name: String,
    pub thumbnail_url: String,
    pub author: String,
    pub description: String,
    pub sticker_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stickers: Option<Vec<StickerInfo>>,
}

/// 获取表情包列表响应
///
/// RPC路由: `sticker/package/list`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerPackageListResponse {
    pub packages: Vec<StickerPackageInfo>,
}

/// 获取表情包详情请求
///
/// RPC路由: `sticker/package/detail`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerPackageDetailRequest {
    pub package_id: String,
}

/// 获取表情包详情响应
///
/// RPC路由: `sticker/package/detail`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerPackageDetailResponse {
    pub package: StickerPackageInfo,
}

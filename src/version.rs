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

//! 协议版本
//!
//! 从 Cargo.toml 的 package.version 获取，供连接认证等使用。

/// 协议库版本（与 Cargo.toml 中 package.version 一致）
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 协议版本号（用于兼容性判断的数值，当前为 1）
pub const PROTOCOL_VERSION: u8 = 1;

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

//! 线上 u64 的字符串编码。
//!
//! 🔴 为什么必须字符串：JavaScript 的 `number` 只有 2^53 精度，雪花 ID 超过它。
//! 走 JSON 数字的话，`608993815990284288` 到了 TS 客户端会变成
//! `608993815990284300` —— **一条不存在的消息**，而且不报错。
//!
//! 编码严格（永远输出字符串），解码宽松（数字也认）：老客户端发数字仍能工作，
//! 新客户端拿到的一律是字符串。

use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberOrString {
        Number(u64),
        Text(String),
    }

    match NumberOrString::deserialize(deserializer)? {
        NumberOrString::Number(value) => Ok(value),
        NumberOrString::Text(text) => text.trim().parse::<u64>().map_err(|_| {
            serde::de::Error::custom(format!(
                "expected a u64 encoded as a decimal string: {text}"
            ))
        }),
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Wire {
        #[serde(with = "super")]
        id: u64,
    }

    /// 编码永远是字符串——这正是 TS 端不丢精度的前提。
    #[test]
    fn a_snowflake_id_goes_out_as_a_string() {
        let json = serde_json::to_string(&Wire {
            id: 608_993_815_990_284_288,
        })
        .expect("encode");
        assert_eq!(json, r#"{"id":"608993815990284288"}"#);
    }

    /// 解码认字符串，也认老客户端发的数字。
    #[test]
    fn decoding_accepts_both_shapes_without_losing_precision() {
        let from_string: Wire =
            serde_json::from_str(r#"{"id":"608993815990284288"}"#).expect("decode string");
        let from_number: Wire =
            serde_json::from_str(r#"{"id":608993815990284288}"#).expect("decode number");
        assert_eq!(from_string.id, 608_993_815_990_284_288);
        assert_eq!(from_number.id, 608_993_815_990_284_288);
    }

    #[test]
    fn a_non_numeric_string_is_an_error_rather_than_a_silent_zero() {
        assert!(serde_json::from_str::<Wire>(r#"{"id":"not-an-id"}"#).is_err());
    }
}

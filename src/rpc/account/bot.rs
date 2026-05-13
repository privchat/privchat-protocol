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

//! Bot follow / unfollow RPC bodies.
//!
//! Spec: `02-server/SERVICE_ACCOUNT_FOLLOW_SPEC.md`
//!
//! Bot follow uses the existing `RpcRequest` (biz_type=17) wire packet —
//! no new wire types are introduced. The bodies below are serialized as
//! JSON and placed in `RpcRequest.body`.

use serde::{Deserialize, Serialize};

/// Request body for `account/bot/follow`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotFollowRequest {
    /// Target Bot user id (must be `user_type = 2`).
    pub bot_user_id: u64,
}

/// Response body for `account/bot/follow`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotFollowResponse {
    /// Echo of the request target.
    pub bot_user_id: u64,
    /// Direct channel id between (caller, bot). Stable across follow/unfollow cycles.
    pub channel_id: u64,
    /// Reserved for future System / Official extension; v1.0 always `2` (Bot).
    pub account_user_type: i16,
    /// Current follow status — always `true` on the success path.
    pub followed: bool,
    /// `true` = newly created relation (or revived from unfollowed);
    /// `false` = already-followed, idempotent reuse.
    pub created: bool,
}

/// Request body for `account/bot/unfollow`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotUnfollowRequest {
    /// Target Bot user id.
    pub bot_user_id: u64,
}

/// Response body for `account/bot/unfollow`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotUnfollowResponse {
    /// Echo of the request target.
    pub bot_user_id: u64,
    /// Existing direct channel id (preserved — never deleted by unfollow).
    /// `0` if there was never a follow relation.
    pub channel_id: u64,
    /// `true` = relation flipped to unfollowed;
    /// `false` = caller was not following; no-op.
    pub unfollowed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::routes;

    #[test]
    fn routes_constants_match_spec() {
        assert_eq!(routes::account_bot::FOLLOW, "account/bot/follow");
        assert_eq!(routes::account_bot::UNFOLLOW, "account/bot/unfollow");
    }

    #[test]
    fn bot_follow_request_serde_roundtrip() {
        let req = BotFollowRequest {
            bot_user_id: 100002888,
        };
        let json = serde_json::to_string(&req).expect("encode");
        assert_eq!(json, r#"{"bot_user_id":100002888}"#);
        let back: BotFollowRequest = serde_json::from_str(&json).expect("decode");
        assert_eq!(back.bot_user_id, 100002888);
    }

    #[test]
    fn bot_follow_response_serde_roundtrip() {
        let resp = BotFollowResponse {
            bot_user_id: 100002888,
            channel_id: 2817,
            account_user_type: 2,
            followed: true,
            created: true,
        };
        let json = serde_json::to_string(&resp).expect("encode");
        let back: BotFollowResponse = serde_json::from_str(&json).expect("decode");
        assert_eq!(back.bot_user_id, 100002888);
        assert_eq!(back.channel_id, 2817);
        assert_eq!(back.account_user_type, 2);
        assert!(back.followed);
        assert!(back.created);
    }

    #[test]
    fn bot_unfollow_response_zero_channel_when_never_followed() {
        let resp = BotUnfollowResponse {
            bot_user_id: 100002888,
            channel_id: 0,
            unfollowed: false,
        };
        let json = serde_json::to_string(&resp).expect("encode");
        let back: BotUnfollowResponse = serde_json::from_str(&json).expect("decode");
        assert_eq!(back.channel_id, 0);
        assert!(!back.unfollowed);
    }
}

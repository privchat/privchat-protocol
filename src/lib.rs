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

//! PrivChat protocol library.
//!
//! `protocol::*` exposes owned Rust structs (`SendMessageRequest`,
//! `PushMessageRequest`, etc.). Wire codec is FlatBuffers — see
//! `protocol/*.fbs` and `codec::FlatBufferMessage`.

// FlatBuffers-generated modules. Each `xxx_generated.rs` declares
// `pub mod privchat::protocol { ... }` and may reference siblings via
// `use crate::xxx_generated::*`. They MUST live at the crate root for
// these cross-references to resolve.
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod common_generated {
    include!(concat!(env!("OUT_DIR"), "/common_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod auth_generated {
    include!(concat!(env!("OUT_DIR"), "/auth_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod disconnect_generated {
    include!(concat!(env!("OUT_DIR"), "/disconnect_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod send_generated {
    include!(concat!(env!("OUT_DIR"), "/send_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod push_generated {
    include!(concat!(env!("OUT_DIR"), "/push_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod ping_generated {
    include!(concat!(env!("OUT_DIR"), "/ping_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod subscribe_generated {
    include!(concat!(env!("OUT_DIR"), "/subscribe_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod publish_generated {
    include!(concat!(env!("OUT_DIR"), "/publish_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod rpc_generated {
    include!(concat!(env!("OUT_DIR"), "/rpc_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod transfer_generated {
    include!(concat!(env!("OUT_DIR"), "/transfer_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod content_generated {
    include!(concat!(env!("OUT_DIR"), "/content_generated.rs"));
}
#[allow(unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes)]
pub mod timeline_generated {
    include!(concat!(env!("OUT_DIR"), "/timeline_generated.rs"));
}

/// Aggregated re-export of all generated FlatBuffers view types.
/// Internal use; application layer should prefer owned types in `protocol`.
pub mod fb {
    pub use crate::auth_generated::privchat::protocol::*;
    pub use crate::common_generated::privchat::protocol::*;
    pub use crate::content_generated::privchat::protocol::*;
    pub use crate::disconnect_generated::privchat::protocol::*;
    pub use crate::ping_generated::privchat::protocol::*;
    pub use crate::publish_generated::privchat::protocol::*;
    pub use crate::push_generated::privchat::protocol::*;
    pub use crate::rpc_generated::privchat::protocol::*;
    pub use crate::send_generated::privchat::protocol::*;
    pub use crate::subscribe_generated::privchat::protocol::*;
    pub use crate::timeline_generated::privchat::protocol::*;
    pub use crate::transfer_generated::privchat::protocol::*;
}

pub mod codec;
pub mod error;
pub mod error_code;
pub mod inbox_event;
pub mod message;
pub mod notification;
pub mod presence;
pub mod protocol;
pub mod rpc;
pub mod version;

pub use codec::{decode_message, encode_message, FlatBufferMessage};
pub use error::ProtocolError;
pub use error_code::ErrorCode;
pub use inbox_event::{
    payloads as inbox_event_payloads, topics as inbox_event_topics, UserInboxEventEnvelope,
    USER_INBOX_EVENT_SCHEMA_VERSION_V1,
};
pub use message::*;
pub use notification::*;
pub use presence::*;
pub use protocol::*;
pub use version::{PROTOCOL_VERSION, VERSION};

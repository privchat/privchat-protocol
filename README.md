# PrivChat Protocol

[简体中文](README.zh-Hans.md)

A Rust protocol library for instant messaging (IM) applications, providing a complete message type system, RPC route definitions, error codes, presence management, and notification types.

## Overview

PrivChat Protocol defines the full communication contract between PrivChat clients and servers. It covers:

- **Transport-layer messages** -- connection auth, heartbeat, message send/receive, pub/sub, batch push, RPC
- **RPC API routes** -- 70+ route constants organized by domain (account, contact, group, message, channel, file, presence, device, sync, etc.)
- **Content message types** -- text, image, file, voice, video, location, sticker, contact card, forward
- **Error codes** -- structured error code system (system 1-999, common 10000-19999, business 20000+)
- **Presence** -- online status, typing indicators, privacy rules
- **Notifications** -- friend, group, message, red packet, and system notifications

## Message Types

The protocol defines 18 transport-layer message types in request/response pairs:

| Type ID | Name | Direction |
|---------|------|-----------|
| 1 / 2 | `AuthorizationRequest` / `Response` | Client -> Server |
| 3 / 4 | `DisconnectRequest` / `Response` | Client -> Server |
| 5 / 6 | `SendMessageRequest` / `Response` | Client -> Server |
| 7 / 8 | `PushMessageRequest` / `Response` | Server -> Client |
| 9 / 10 | `PushBatchRequest` / `Response` | Server -> Client |
| 11 / 12 | `PingRequest` / `PongResponse` | Client -> Server |
| 13 / 14 | `SubscribeRequest` / `Response` | Client -> Server |
| 15 / 16 | `PublishRequest` / `Response` | Server -> Client |
| 17 / 18 | `RpcRequest` / `RpcResponse` | Client -> Server |

## RPC Routes

All RPC operations are routed via `RpcRequest` / `RpcResponse` messages. Route constants are organized by domain:

| Domain | Module | Routes |
|--------|--------|--------|
| Auth | `rpc::routes::auth` | login, logout, refresh |
| Account | `rpc::routes::account_user` | register, detail, share_card, update |
| Profile | `rpc::routes::account_profile` | get, update |
| Search | `rpc::routes::account_search` | query, by_qrcode |
| Privacy | `rpc::routes::privacy` | get, update |
| Friend | `rpc::routes::friend` | apply, accept, reject, remove, pending, check |
| Blacklist | `rpc::routes::blacklist` | add, remove, list, check |
| Group | `rpc::routes::group` | create, info |
| Group Member | `rpc::routes::group_member` | add, remove, list, leave, mute, unmute |
| Group Role | `rpc::routes::group_role` | transfer_owner, set |
| Group Settings | `rpc::routes::group_settings` | get, update, mute_all |
| Group Approval | `rpc::routes::group_approval` | list, handle |
| Message | `rpc::routes::message` | revoke |
| Message History | `rpc::routes::message_history` | get |
| Message Status | `rpc::routes::message_status` | read, count, read_list, read_stats |
| Message Reaction | `rpc::routes::message_reaction` | add, remove, list, stats |
| Channel | `rpc::routes::channel` | direct_get_or_create, pin, hide, mute |
| Broadcast | `rpc::routes::channel_broadcast` | create, subscribe, list |
| File | `rpc::routes::file` | request_upload_token, upload_callback |
| Presence | `rpc::routes::presence` | subscribe, unsubscribe, typing, status_get |
| Device | `rpc::routes::device` | push_update, push_status |
| Sticker | `rpc::routes::sticker` | package_list, package_detail |
| QR Code | `rpc::routes::qrcode` | generate, resolve, refresh, revoke, list |
| Sync | `rpc::routes::sync` | submit, get_difference, get_channel_pts, batch_get_channel_pts, session_ready |
| Entity | `rpc::routes::entity` | sync_entities |

## Error Codes

Structured error code system with `ErrorCode` enum (`#[repr(u32)]`):

| Range | Category | Examples |
|-------|----------|----------|
| 0 | Success | `Ok` |
| 1 -- 999 | System errors | `SystemError`, `Timeout`, `ProtocolError`, `ClientVersionTooOld` |
| 10000 -- 19999 | Common errors | `AuthRequired`, `InvalidToken`, `PermissionDenied`, `RateLimitExceeded` |
| 20000+ | Business errors | `MessageNotFound`, `GroupFull`, `NotGroupMember`, `FileTooLarge` |

## Content Message Types

The `ContentMessageType` enum defines payload types carried inside send/push messages:

| Value | Type | Description |
|-------|------|-------------|
| 0 | Text | Plain text message |
| 1 | Image | Image with file_id, dimensions |
| 2 | File | Generic file attachment |
| 3 | Voice | Voice recording |
| 4 | Video | Video clip |
| 5 | System | System notification |
| 6 | Audio | Audio file |
| 7 | Location | GPS coordinates |
| 8 | ContactCard | User contact card |
| 9 | Sticker | Sticker with package |
| 10 | Forward | Forwarded message(s) |

## Presence & Typing

- **OnlineStatus**: `Online`, `Recently`, `LastWeek`, `LastMonth`, `LongTimeAgo`, `Offline`
- **TypingActionType**: `Typing`, `Recording`, `UploadingPhoto`, `UploadingVideo`, `UploadingFile`, `ChoosingSticker`
- **PrivacyRule**: `Everyone`, `Contacts`, `Nobody`, `Custom { allow_users, deny_users }`

## Notification Types

Covers friend, group, message, red packet, and system events:

- **Friend**: request sent/accepted/rejected, friend deleted
- **Group**: created, member joined/left/kicked, name/avatar/announcement changed, owner transferred, admin added/removed, member muted/unmuted, group dismissed
- **Message**: revoked, pinned/unpinned, read, edited
- **Red Packet**: sent, received, empty, expired
- **System**: maintenance, announcement, version update

## Sync Protocol

Dual sync system for reliable message delivery:

- **PTS-based message sync** -- per-channel monotonic sequence numbers, gap detection, `GetDifference` for catch-up
- **Entity version sync** -- incremental sync for contacts, groups, channels, settings with tombstone support

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
privchat-protocol = "0.1"
```

```rust
use privchat_protocol::protocol::*;
use privchat_protocol::rpc::routes;

// Create an RPC request
let req = RpcRequest {
    route: routes::auth::LOGIN.to_string(),
    body: serde_json::json!({
        "phone": "+1234567890",
        "code": "123456"
    }),
};

// Access message types
let msg_type = MessageType::RpcRequest;
assert_eq!(msg_type as u8, 17);
```

## Build

```bash
cargo build
cargo test
```

## License

Apache-2.0. See [LICENSE](LICENSE) for details.

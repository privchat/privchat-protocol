# PrivChat Protocol — FlatBuffers Schema

This directory holds the canonical FlatBuffers schemas for the PrivChat
application protocol. Generated Rust / TypeScript / Swift / Kotlin code is
produced from these `.fbs` files; no parallel hand-maintained schemas exist.

> **Wire-incompatible with the legacy JSON protocol.** This is a clean cutover
> while the project is pre-public. There is no v1/v2 dual codec.

## File layout

| File              | Contents |
|-------------------|----------|
| `common.fbs`      | `MessageType`, `AuthType`, `DeviceType`, `DisconnectReason`, `MessageSetting`, `Property` |
| `auth.fbs`        | `AuthorizationRequest/Response`, `ClientInfo`, `DeviceInfo`, `ServerInfo` |
| `disconnect.fbs`  | `DisconnectRequest/Response` |
| `send.fbs`        | `SendMessageRequest/Response` |
| `push.fbs`        | `PushMessageRequest/Response`, `PushBatchRequest/Response` |
| `ping.fbs`        | `PingRequest`, `PongResponse` |
| `subscribe.fbs`   | `SubscribeRequest/Response` |
| `publish.fbs`     | `PublishRequest/Response` |
| `rpc.fbs`         | `RpcRequest/Response` |

All files share `namespace privchat.protocol`.

## Tooling

- **Required:** `flatc` 24.x (verified with `flatc 24.3.25`).
- Install: `brew install flatbuffers` (macOS), or download from
  <https://github.com/google/flatbuffers/releases>.
- CI must pin a single `flatc` version — generated code differs across
  major versions.

Rust integration is via `build.rs` invoking `flatc`; output lands in
`OUT_DIR` and is `include!`-ed by `src/generated/mod.rs`.

## Hard rules (do not break)

### 1. Enum 0 is reserved

Every enum's value 0 is `Unknown` / `Unspecified`. FlatBuffers initialises
absent fields to 0; reserving 0 means "value not set" can never be confused
with a valid business value.

```fbs
enum AuthType : ubyte {
  Unspecified  = 0,    // ← reserved, do not assign business meaning
  JWT          = 1,
  ...
}
```

### 2. Enum numeric values are PERMANENT

Once a variant has a value, that value is locked forever. New variants are
added at the end with the next free value. **Never** renumber, **never**
remove. Use renaming only when semantics are preserved.

### 3. `0 = absent` for `ulong` / `uint` IDs

Business-ID fields (`user_id`, `channel_id`, `local_message_id`,
`server_message_id`, `from_uid`, etc.) treat `0` as "not set". Application
counters MUST start from 1.

### 4. Strings: empty ≠ absent

If a string field is absent on the wire, the decoder returns `None` /
`Option::None`. Empty string `""` is a valid present value.

### 5. Schema evolution

- Add new fields **at the end** of a `table`. Never insert.
- Never change a field's type.
- Never remove fields. Use `(deprecated)` to retire one.
- `struct` (fixed layout) is forbidden in this schema set — everything is
  `table` to keep evolvability. Tiny perf cost, big future-proofing.

### 6. `[ubyte]` for opaque payloads

`SendMessageRequest.payload`, `PushMessageRequest.payload`,
`PublishRequest.payload`, `RpcRequest.body`, `RpcResponse.data` are
`[ubyte]` — the application layer decides their encoding (current default:
JSON UTF-8 for RPC body, raw bytes for media).

## Wire format integration with msgtrans

PrivChat is a payload protocol on top of `msgtrans`:

```
msgtrans Packet
├── header
│   ├── version    = 1   (msgtrans wire format, not PrivChat protocol version)
│   ├── biz_type   = MessageType (1..18)
│   └── ...
└── payload                ← single FlatBuffer encoding the matching message
```

`msgtrans.version` is **never** used to signal PrivChat protocol changes.
PrivChat-level versioning, if ever needed, lives inside the FlatBuffer
schemas (e.g. a `protocol_version` field on `AuthorizationRequest`).

## Type mapping reference

| Rust source                | FlatBuffers              | Notes                        |
|----------------------------|--------------------------|------------------------------|
| `u8`                       | `ubyte`                  | |
| `u32`                      | `uint`                   | |
| `u64`                      | `ulong`                  | TS exposes as `bigint` internally, `string` at SDK boundary |
| `i64`                      | `long`                   | timestamps in PingRequest |
| `bool`                     | `bool`                   | |
| `String`                   | `string`                 | optional in fbs, present-or-absent in Rust |
| `Option<u32>`              | `uint`                   | 0 = absent |
| `Option<u64>`              | `ulong`                  | 0 = absent |
| `Option<String>`           | `string`                 | absent ≠ empty |
| `Vec<u8>`                  | `[ubyte]`                | |
| `Vec<T>`                   | `[T]`                    | |
| `HashMap<String, String>`  | `[Property]`             | array of key/value tables |
| `serde_json::Value`        | `[ubyte]`                | opaque, layer above decides codec |
| `enum`                     | `enum X : ubyte`         | 0 = Unknown/Unspecified |
| nested struct              | `table`                  | always `table`, never `struct` |

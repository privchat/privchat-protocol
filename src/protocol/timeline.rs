//! Canonical timeline events.
//!
//! The legacy sync RPC keeps `message_type + content` JSON for rolling
//! compatibility. New endpoints additionally carry this FlatBuffers value so
//! IDs and structured payloads never pass through JavaScript numbers.

use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use crate::message::{ContentMessageType, LocalMessagePayloadEnvelope};
use crate::protocol::content::{decode_payload_envelope, encode_payload_envelope};
use crate::MessagePayloadEnvelope;
use flatbuffers::FlatBufferBuilder;
use serde::Serialize;

pub const CANONICAL_TIMELINE_EVENT_SCHEMA_V1: u16 = 1;
/// Push topic whose payload is a FlatBuffers `CanonicalTimelineEvent`.
pub const CANONICAL_TIMELINE_PUSH_TOPIC_V1: &str = "timeline.canonical.v1";

#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalTimelineEvent {
    NewMessage(NewMessageEvent),
    Revoke(RevokeEvent),
    ReactionChange(ReactionChangeEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewMessageEvent {
    pub message_type: ContentMessageType,
    pub payload: MessagePayloadEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevokeEvent {
    pub target_server_message_id: u64,
    pub revoked_by: u64,
    pub revoked_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionOperation {
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionChangeEvent {
    pub target_server_message_id: u64,
    pub actor_id: u64,
    pub emoji: String,
    pub operation: ReactionOperation,
}

impl CanonicalTimelineEvent {
    /// Produce the additive legacy `message_type + content` projection used by
    /// old clients. IDs are strings so JSON consumers cannot lose u64 bits.
    pub fn to_legacy_commit(
        &self,
        channel_id: u64,
        channel_type: u8,
    ) -> Result<(String, serde_json::Value), ProtocolError> {
        #[derive(Serialize)]
        struct LegacyRevoke {
            message_id: String,
            channel_id: String,
            channel_type: u8,
            revoke: bool,
            revoked_by: String,
            revoked_at: i64,
        }
        #[derive(Serialize)]
        struct LegacyReaction<'a> {
            message_id: String,
            channel_id: String,
            channel_type: u8,
            uid: String,
            emoji: &'a str,
            deleted: bool,
        }

        let (message_type, value) = match self {
            Self::NewMessage(event) => (
                event.message_type.as_str().to_string(),
                serde_json::to_value(event.payload.to_legacy()),
            ),
            Self::Revoke(event) => (
                "message.revoke".to_string(),
                serde_json::to_value(LegacyRevoke {
                    message_id: event.target_server_message_id.to_string(),
                    channel_id: channel_id.to_string(),
                    channel_type,
                    revoke: true,
                    revoked_by: event.revoked_by.to_string(),
                    revoked_at: event.revoked_at,
                }),
            ),
            Self::ReactionChange(event) => (
                "message_reaction".to_string(),
                serde_json::to_value(LegacyReaction {
                    message_id: event.target_server_message_id.to_string(),
                    channel_id: channel_id.to_string(),
                    channel_type,
                    uid: event.actor_id.to_string(),
                    emoji: &event.emoji,
                    deleted: matches!(event.operation, ReactionOperation::Remove),
                }),
            ),
        };
        value
            .map(|value| (message_type, value))
            .map_err(|e| ProtocolError::InvalidValue(format!("legacy timeline projection: {e}")))
    }

    /// Convert an existing legacy commit payload into the canonical event.
    /// Unknown command types remain legacy-only during the additive rollout.
    pub fn from_legacy(
        message_type: &str,
        content: &serde_json::Value,
        server_msg_id: u64,
        sender_id: u64,
        server_timestamp: i64,
    ) -> Result<Option<Self>, ProtocolError> {
        match message_type {
            "message.revoke" | "message_extra" | "message_ext" => {
                let target = json_u64(content, "message_id").unwrap_or(server_msg_id);
                let revoked_by = json_u64(content, "revoked_by").unwrap_or(sender_id);
                let revoked_at = content
                    .get("revoked_at")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(server_timestamp);
                Ok(Some(Self::Revoke(RevokeEvent {
                    target_server_message_id: target,
                    revoked_by,
                    revoked_at,
                })))
            }
            "message_reaction" | "reaction" | "message.reaction" => {
                let target = json_u64(content, "message_id").ok_or_else(|| {
                    ProtocolError::InvalidValue(
                        "reaction legacy payload is missing message_id".to_string(),
                    )
                })?;
                let actor_id = json_u64(content, "uid").unwrap_or(sender_id);
                let emoji = content
                    .get("emoji")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        ProtocolError::InvalidValue(
                            "reaction legacy payload is missing emoji".to_string(),
                        )
                    })?
                    .to_string();
                let operation = if content
                    .get("deleted")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                {
                    ReactionOperation::Remove
                } else {
                    ReactionOperation::Add
                };
                Ok(Some(Self::ReactionChange(ReactionChangeEvent {
                    target_server_message_id: target,
                    actor_id,
                    emoji,
                    operation,
                })))
            }
            value => {
                let Some(content_type) = ContentMessageType::from_str(value) else {
                    return Ok(None);
                };
                let payload = payload_from_legacy_commit(content_type, content)?;
                Ok(Some(Self::NewMessage(NewMessageEvent {
                    message_type: content_type,
                    payload,
                })))
            }
        }
    }
}

fn json_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    let field = value.get(key)?;
    field
        .as_u64()
        .or_else(|| field.as_str().and_then(|raw| raw.parse().ok()))
}

fn payload_from_legacy_commit(
    content_type: ContentMessageType,
    value: &serde_json::Value,
) -> Result<MessagePayloadEnvelope, ProtocolError> {
    let is_envelope = value.as_object().is_some_and(|object| {
        [
            "content",
            "metadata",
            "reply_to_message_id",
            "mentioned_user_ids",
            "message_source",
        ]
        .iter()
        .any(|key| object.contains_key(*key))
    });
    if is_envelope {
        if let Ok(legacy) = serde_json::from_value::<LocalMessagePayloadEnvelope>(value.clone()) {
            return Ok(MessagePayloadEnvelope::from_legacy(&legacy, content_type));
        }
    }

    let display_content = value
        .as_str()
        .map(str::to_string)
        .or_else(|| {
            value
                .get("text")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            value
                .get("content")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| value.to_string());

    let metadata_value = value.get("metadata").unwrap_or(value);
    let metadata = crate::MessageMetadata::from_json_value(content_type, metadata_value);
    let requires_metadata = !matches!(
        content_type,
        ContentMessageType::Text
            | ContentMessageType::System
            | ContentMessageType::RedPacket
            | ContentMessageType::MoneyTransfer
    );
    if requires_metadata && metadata.is_none() {
        return Err(ProtocolError::InvalidValue(format!(
            "legacy {} payload cannot be mapped without metadata",
            content_type.as_str()
        )));
    }

    Ok(MessagePayloadEnvelope {
        content: display_content,
        metadata,
        reply_to_message_id: None,
        mentioned_user_ids: Vec::new(),
        message_source: None,
        // legacy 投影没有这个字段；转发副本的来源走 JSON 投影下发（§6.2）。
        forward_origin: None,
    })
}

impl FlatBufferMessage for CanonicalTimelineEvent {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let (payload_type, payload) = match self {
            Self::NewMessage(event) => {
                let payload = encode_payload_envelope(builder, &event.payload);
                let offset = fb::NewMessageEvent::create(
                    builder,
                    &fb::NewMessageEventArgs {
                        message_type: event.message_type.as_u32(),
                        payload: Some(payload),
                    },
                );
                (
                    fb::TimelineEventPayload::NewMessageEvent,
                    offset.as_union_value(),
                )
            }
            Self::Revoke(event) => {
                let offset = fb::RevokeEvent::create(
                    builder,
                    &fb::RevokeEventArgs {
                        target_server_message_id: event.target_server_message_id,
                        revoked_by: event.revoked_by,
                        revoked_at: event.revoked_at,
                    },
                );
                (
                    fb::TimelineEventPayload::RevokeEvent,
                    offset.as_union_value(),
                )
            }
            Self::ReactionChange(event) => {
                let emoji = builder.create_string(&event.emoji);
                let operation = match event.operation {
                    ReactionOperation::Add => fb::ReactionOperation::Add,
                    ReactionOperation::Remove => fb::ReactionOperation::Remove,
                };
                let offset = fb::ReactionChangeEvent::create(
                    builder,
                    &fb::ReactionChangeEventArgs {
                        target_server_message_id: event.target_server_message_id,
                        actor_id: event.actor_id,
                        emoji: Some(emoji),
                        operation,
                    },
                );
                (
                    fb::TimelineEventPayload::ReactionChangeEvent,
                    offset.as_union_value(),
                )
            }
        };
        let offset = fb::CanonicalTimelineEvent::create(
            builder,
            &fb::CanonicalTimelineEventArgs {
                payload_type,
                payload: Some(payload),
            },
        );
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::CanonicalTimelineEvent>(bytes)?;
        match view.payload_type() {
            fb::TimelineEventPayload::NewMessageEvent => {
                let event = view
                    .payload_as_new_message_event()
                    .ok_or(ProtocolError::MissingField("timeline.new_message"))?;
                let message_type =
                    ContentMessageType::from_u32(event.message_type()).ok_or_else(|| {
                        ProtocolError::InvalidValue(format!(
                            "unknown content message type {}",
                            event.message_type()
                        ))
                    })?;
                let payload = event
                    .payload()
                    .ok_or(ProtocolError::MissingField("timeline.new_message.payload"))?;
                Ok(Self::NewMessage(NewMessageEvent {
                    message_type,
                    payload: decode_payload_envelope(payload),
                }))
            }
            fb::TimelineEventPayload::RevokeEvent => {
                let event = view
                    .payload_as_revoke_event()
                    .ok_or(ProtocolError::MissingField("timeline.revoke"))?;
                Ok(Self::Revoke(RevokeEvent {
                    target_server_message_id: event.target_server_message_id(),
                    revoked_by: event.revoked_by(),
                    revoked_at: event.revoked_at(),
                }))
            }
            fb::TimelineEventPayload::ReactionChangeEvent => {
                let event = view
                    .payload_as_reaction_change_event()
                    .ok_or(ProtocolError::MissingField("timeline.reaction_change"))?;
                let operation = match event.operation() {
                    fb::ReactionOperation::Add => ReactionOperation::Add,
                    fb::ReactionOperation::Remove => ReactionOperation::Remove,
                    _ => {
                        return Err(ProtocolError::InvalidValue(
                            "unknown reaction operation".to_string(),
                        ))
                    }
                };
                Ok(Self::ReactionChange(ReactionChangeEvent {
                    target_server_message_id: event.target_server_message_id(),
                    actor_id: event.actor_id(),
                    emoji: event.emoji().unwrap_or("").to_string(),
                    operation,
                }))
            }
            _ => Err(ProtocolError::InvalidValue(
                "unknown canonical timeline event payload".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_round_trip_flatbuffers() {
        let events = vec![
            CanonicalTimelineEvent::NewMessage(NewMessageEvent {
                message_type: ContentMessageType::Text,
                payload: MessagePayloadEnvelope {
                    content: "hello".to_string(),
                    ..Default::default()
                },
            }),
            CanonicalTimelineEvent::Revoke(RevokeEvent {
                target_server_message_id: 9_007_199_254_740_993,
                revoked_by: 42,
                revoked_at: 123,
            }),
            CanonicalTimelineEvent::ReactionChange(ReactionChangeEvent {
                target_server_message_id: 9_007_199_254_740_995,
                actor_id: 43,
                emoji: "thumbs-up".to_string(),
                operation: ReactionOperation::Remove,
            }),
        ];
        for event in events {
            let bytes = event.encode_fb().expect("encode canonical event");
            let decoded =
                CanonicalTimelineEvent::decode_fb(&bytes).expect("decode canonical event");
            assert_eq!(decoded, event);
        }
    }

    #[test]
    fn maps_legacy_media_without_dropping_metadata() {
        let legacy = serde_json::to_value(LocalMessagePayloadEnvelope {
            content: String::new(),
            metadata: Some(
                serde_json::to_value(crate::ImageMetadata {
                    file_id: 77,
                    file_name: Some("photo.jpg".to_string()),
                    width: 10,
                    height: 20,
                    ..Default::default()
                })
                .expect("serialize image metadata"),
            ),
            ..Default::default()
        })
        .expect("serialize legacy envelope");
        let event = CanonicalTimelineEvent::from_legacy("image", &legacy, 1, 2, 3)
            .expect("map legacy")
            .expect("known event");
        let CanonicalTimelineEvent::NewMessage(event) = event else {
            panic!("expected new message")
        };
        assert_eq!(event.message_type, ContentMessageType::Image);
        let Some(crate::MessageMetadata::Image(image)) = event.payload.metadata else {
            panic!("expected image metadata")
        };
        assert_eq!(image.file_id, 77);
        assert_eq!(image.file_name.as_deref(), Some("photo.jpg"));
    }

    #[test]
    fn maps_opaque_money_snapshot_without_dropping_fields() {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct MoneySnapshot<'a> {
            red_packet_id: &'a str,
            title: &'a str,
            amount_text: &'a str,
            sender_user_id: u64,
        }

        let legacy = serde_json::to_value(MoneySnapshot {
            red_packet_id: "9007199254740993",
            title: "gift",
            amount_text: "CNY 8.88",
            sender_user_id: 42,
        })
        .expect("serialize money snapshot");
        let event = CanonicalTimelineEvent::from_legacy("red_packet", &legacy, 1, 42, 3)
            .expect("map legacy")
            .expect("known event");
        let CanonicalTimelineEvent::NewMessage(event) = event else {
            panic!("expected new message")
        };
        assert_eq!(event.message_type, ContentMessageType::RedPacket);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&event.payload.content)
                .expect("canonical opaque snapshot"),
            legacy
        );
    }

    #[test]
    fn legacy_mutation_projection_keeps_u64_ids_as_strings() {
        let event = CanonicalTimelineEvent::ReactionChange(ReactionChangeEvent {
            target_server_message_id: 9_007_199_254_740_993,
            actor_id: 9_007_199_254_740_995,
            emoji: "ok".to_string(),
            operation: ReactionOperation::Add,
        });
        let (message_type, value) = event
            .to_legacy_commit(9_007_199_254_740_997, 2)
            .expect("legacy projection");
        assert_eq!(message_type, "message_reaction");
        assert_eq!(value["message_id"], "9007199254740993");
        assert_eq!(value["uid"], "9007199254740995");
        assert_eq!(value["channel_id"], "9007199254740997");
    }
}

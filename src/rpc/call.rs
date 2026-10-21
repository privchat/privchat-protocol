//! Server-authoritative call signalling; media credentials never contain SFU secrets.
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallTransport {
    #[default]
    Livekit,
    Webrtc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallMediaKind {
    Audio,
    Video,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallState {
    Ringing,
    Active,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallEndReason {
    Normal,
    Rejected,
    Timeout,
    Busy,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallParticipantState {
    Invited,
    Joined,
    Left,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallParticipant {
    pub user_id: u64,
    pub state: CallParticipantState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallSnapshot {
    #[serde(default)]
    pub transport: CallTransport,
    pub call_id: u64,
    pub channel_id: u64,
    pub initiator_id: u64,
    pub media_kind: CallMediaKind,
    pub state: CallState,
    pub end_reason: Option<CallEndReason>,
    pub version: u64,
    pub created_at_ms: i64,
    pub answered_at_ms: Option<i64>,
    pub ended_at_ms: Option<i64>,
    pub expires_at_ms: i64,
    pub participants: Vec<CallParticipant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInviteRequest {
    #[serde(default)]
    pub supports_peer: bool,
    pub command_id: String,
    pub channel_id: u64,
    pub media_kind: CallMediaKind,
    /// Empty selects the eligible channel peers, subject to the configured participant limit.
    #[serde(default)]
    pub invitee_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallMutationRequest {
    #[serde(default)]
    pub supports_peer: bool,
    pub command_id: String,
    pub call_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGetRequest {
    pub call_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResponse {
    pub call: CallSnapshot,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CallJoinResponse {
    pub call: CallSnapshot,
    #[serde(default)]
    pub peer: Option<CallPeerConfig>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub expires_at_ms: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CallIceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CallPeerConfig {
    pub self_user_id: u64,
    pub peer_user_id: u64,
    pub offerer: bool,
    pub ice_servers: Vec<CallIceServer>,
    /// Zero when TURN is disabled and no credentials need renewal.
    pub credential_expires_at_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallPeerSignalKind {
    Offer,
    Answer,
    Ice,
}

// SDP, candidates and TURN credentials deliberately do not implement Debug.
#[derive(Clone, Serialize, Deserialize)]
pub struct CallPeerSignalRequest {
    pub command_id: String,
    pub call_id: u64,
    pub revision: u32,
    pub kind: CallPeerSignalKind,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallPeerEventsRequest {
    pub call_id: u64,
    pub after_seq: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CallPeerEvent {
    pub seq: u64,
    pub revision: u32,
    pub kind: CallPeerSignalKind,
    pub payload: String,
    pub from_user_id: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CallPeerEventsResponse {
    pub events: Vec<CallPeerEvent>,
    pub next_seq: u64,
}

impl std::fmt::Debug for CallJoinResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallJoinResponse")
            .field("call", &self.call)
            .field("url", &self.url)
            .field("token", &"[REDACTED]")
            .field("expires_at_ms", &self.expires_at_ms)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallActiveResponse {
    pub calls: Vec<CallSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallCapabilitiesResponse {
    pub enabled: bool,
    #[serde(default)]
    pub peer_enabled: bool,
    #[serde(default)]
    pub livekit_enabled: bool,
    pub audio_participant_limit: u32,
    pub video_participant_limit: u32,
    pub ringing_timeout_seconds: u32,
    #[serde(default = "default_heartbeat_lease_seconds")]
    pub heartbeat_lease_seconds: u32,
}

fn default_heartbeat_lease_seconds() -> u32 {
    60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn additive_peer_fields_default_for_legacy_wire() {
        let invite: CallInviteRequest =
            serde_json::from_str(r#"{"command_id":"c","channel_id":1,"media_kind":"audio"}"#)
                .unwrap();
        assert!(!invite.supports_peer);
        let mutation: CallMutationRequest =
            serde_json::from_str(r#"{"command_id":"c","call_id":1}"#).unwrap();
        assert!(!mutation.supports_peer);
        let snapshot = serde_json::json!({"call_id":1,"channel_id":2,"initiator_id":3,"media_kind":"audio","state":"ringing","version":1,"created_at_ms":1,"expires_at_ms":2,"participants":[]});
        let join: CallJoinResponse =
            serde_json::from_value(serde_json::json!({"call":snapshot})).unwrap();
        assert_eq!(join.call.transport, CallTransport::Livekit);
        assert!(join.peer.is_none());
        assert!(join.token.is_empty());
        assert!(join.url.is_empty());
        assert_eq!(join.expires_at_ms, 0);
    }

    #[test]
    fn join_debug_does_not_disclose_credentials() {
        let response = CallJoinResponse {
            peer: None,
            call: CallSnapshot {
                transport: CallTransport::Livekit,
                call_id: 1,
                channel_id: 2,
                initiator_id: 3,
                media_kind: CallMediaKind::Audio,
                state: CallState::Active,
                end_reason: None,
                version: 1,
                created_at_ms: 1,
                answered_at_ms: Some(2),
                ended_at_ms: None,
                expires_at_ms: 3000,
                participants: vec![],
            },
            url: "wss://media.example.test".into(),
            token: "private-test-token".into(),
            expires_at_ms: 3000,
        };
        assert!(!format!("{response:?}").contains("private-test-token"));
        assert!(serde_json::to_string(&response)
            .unwrap()
            .contains("private-test-token"));
    }

    #[test]
    fn control_enums_and_large_ids_round_trip_without_float_conversion() {
        let value = CallInviteRequest {
            supports_peer: false,
            command_id: "test-command".into(),
            channel_id: 9_007_199_254_740_993,
            media_kind: CallMediaKind::Video,
            invitee_ids: vec![9_007_199_254_740_995],
        };
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("\"media_kind\":\"video\""));
        let decoded: CallInviteRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.channel_id, value.channel_id);
        assert_eq!(decoded.invitee_ids, value.invitee_ids);
        assert!(serde_json::from_str::<CallMediaKind>("\"unknown\"").is_err());
    }
}

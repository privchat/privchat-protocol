//! Weey node business protocol v1. JSON carried by Transfer, never native Paseo wire.
use serde::{Deserialize, Serialize};

pub const MAX_BODY_BYTES: usize = 64 * 1024;
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopeKind {
    Private,
    Workspace,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentScope {
    pub kind: ScopeKind,
    pub r#ref: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Command<P> {
    pub protocol_version: u8,
    pub command_id: String,
    pub execution_id: String,
    pub scope: AgentScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_before: Option<u64>,
    pub payload: P,
}
macro_rules! dto {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }
    };
}
dto!(StartPayload { capability: String, agent_id: Option<String>, prompt: String, model: String, workdir_binding_id: String, parent_execution_id: Option<String>, input_refs: Option<Vec<String>> });
dto!(SendPayload {
    text: String,
    message_id: String,
    continue_from_execution_id: String
});
dto!(CancelPayload { reason: Option<String> });
dto!(EmptyPayload {});
dto!(HistoryPayload { cursor: Option<String>, limit: u32 });
dto!(WatchPayload { after_seq: Option<u64>, lease_ms: u32 });
dto!(UnwatchPayload {
    subscription_id: String
});
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    AllowOnce,
    Deny,
}
dto!(PermissionPayload {
    permission_id: String,
    request_digest: String,
    decision: PermissionDecision
});

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum NodeCommand {
    #[serde(rename = "execution.start")]
    Start(Command<StartPayload>),
    #[serde(rename = "execution.cancel")]
    Cancel(Command<CancelPayload>),
    #[serde(rename = "session.send")]
    Send(Command<SendPayload>),
    #[serde(rename = "session.inspect")]
    Inspect(Command<EmptyPayload>),
    #[serde(rename = "session.history")]
    History(Command<HistoryPayload>),
    #[serde(rename = "session.watch")]
    Watch(Command<WatchPayload>),
    #[serde(rename = "session.unwatch")]
    Unwatch(Command<UnwatchPayload>),
    #[serde(rename = "permission.resolve")]
    Permission(Command<PermissionPayload>),
}
impl NodeCommand {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Start(_) => "execution.start",
            Self::Cancel(_) => "execution.cancel",
            Self::Send(_) => "session.send",
            Self::Inspect(_) => "session.inspect",
            Self::History(_) => "session.history",
            Self::Watch(_) => "session.watch",
            Self::Unwatch(_) => "session.unwatch",
            Self::Permission(_) => "permission.resolve",
        }
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        macro_rules! common {
            ($c:expr) => {{
                let c = $c;
                if c.protocol_version != 1 {
                    return Err("AGENT_PROTOCOL_UNSUPPORTED");
                }
                if c.command_id.is_empty()
                    || c.execution_id.is_empty()
                    || (c.scope.kind == ScopeKind::Workspace && c.scope.r#ref.is_empty())
                {
                    return Err("AGENT_EXECUTION_INVALID");
                }
                if c.start_before.is_some_and(|n| n > MAX_SAFE_INTEGER) {
                    return Err("AGENT_EXECUTION_INVALID");
                }
                if c.start_before.is_some() && !matches!(self, Self::Start(_) | Self::Send(_)) {
                    return Err("AGENT_EXECUTION_INVALID");
                }
            }};
        }
        match self {
            Self::Start(c) => {
                common!(c);
                if [
                    &c.payload.capability,
                    &c.payload.prompt,
                    &c.payload.model,
                    &c.payload.workdir_binding_id,
                ]
                .iter()
                .any(|s| s.is_empty())
                {
                    return Err("AGENT_EXECUTION_INVALID");
                }
            }
            Self::Send(c) => {
                common!(c);
                if c.payload.text.is_empty()
                    || c.payload.message_id.is_empty()
                    || c.payload.continue_from_execution_id.is_empty()
                    || c.payload.continue_from_execution_id == c.execution_id
                {
                    return Err("AGENT_EXECUTION_INVALID");
                }
            }
            Self::Cancel(c) => common!(c),
            Self::Inspect(c) => common!(c),
            Self::History(c) => {
                common!(c);
                if c.payload.limit == 0 || c.payload.limit > 100 {
                    return Err("AGENT_EXECUTION_INVALID");
                }
            }
            Self::Watch(c) => {
                common!(c);
                if c.payload.lease_ms == 0
                    || c.payload.lease_ms > 60000
                    || c.payload.after_seq.is_some_and(|n| n > MAX_SAFE_INTEGER)
                {
                    return Err("AGENT_EXECUTION_INVALID");
                }
            }
            Self::Unwatch(c) => {
                common!(c);
                if c.payload.subscription_id.is_empty() {
                    return Err("AGENT_EXECUTION_INVALID");
                }
            }
            Self::Permission(c) => {
                common!(c);
                if c.payload.permission_id.is_empty()
                    || c.payload.request_digest.len() != 64
                    || !c
                        .payload
                        .request_digest
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                {
                    return Err("AGENT_EXECUTION_INVALID");
                }
            }
        }
        Ok(())
    }
}
pub fn decode_command(route: &str, body: &[u8]) -> Result<NodeCommand, &'static str> {
    if body.len() > MAX_BODY_BYTES {
        return Err("AGENT_PAYLOAD_TOO_LARGE");
    }
    let c: NodeCommand = serde_json::from_slice(body).map_err(|_| "AGENT_EXECUTION_INVALID")?;
    c.validate()?;
    if route != format!("agent/{}", c.kind().replace('.', "/")) {
        return Err("AGENT_OPERATION_UNSUPPORTED");
    }
    Ok(c)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Applied,
    Rejected,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResultKind {
    #[serde(rename = "command.result")]
    CommandResult,
}
dto!(NodeError {
    message: String,
    retryable: bool
});
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeResult<R> {
    pub protocol_version: u8,
    pub kind: ResultKind,
    pub command_id: String,
    pub execution_id: String,
    pub outcome: Outcome,
    pub data: Option<R>,
    pub error: Option<NodeError>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    Accepted,
    Dispatched,
    Running,
    WaitingUser,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    UserText,
    AssistantText,
    ToolSummary,
    Artifact,
    Notice,
}
dto!(HistoryItem { id: String, kind: ItemKind, text: Option<String>, tool_name: Option<String>, artifact_ref: Option<String>, native_kind: Option<String> });
dto!(InspectResult { session_state: String, native_session_id: String, active_turn_id: Option<String>, pending_permission_ids: Vec<String> });
dto!(HistoryResult { items: Vec<HistoryItem>, next_cursor: Option<String>, snapshot_version: String });
dto!(WatchResult {
    subscription_id: String,
    expires_at: u64,
    base_seq: u64
});
dto!(StatePayload { state: ExecutionState, state_version: u64, native_session_id: Option<String>, turn_outcome: Option<String> });
dto!(OutputPayload { items: Vec<HistoryItem> });
dto!(PermissionRequested {
    permission_id: String,
    request_digest: String,
    summary: String,
    expires_at: u64
});
dto!(PermissionResolved {
    permission_id: String,
    decision: PermissionDecision,
    resolved_by: String
});
dto!(HistoryInvalidated {
    snapshot_version: String
});
dto!(GapPayload {
    after_seq: u64,
    snapshot_required: bool
});
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEvent {
    pub protocol_version: u8,
    pub execution_id: String,
    pub event_id: String,
    pub seq: u64,
    #[serde(flatten)]
    pub event: EventBody,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum EventBody {
    State(StatePayload),
    Output(OutputPayload),
    PermissionRequested(PermissionRequested),
    PermissionResolved(PermissionResolved),
    HistoryInvalidated(HistoryInvalidated),
    Gap(GapPayload),
}
dto!(Capability { name: String, version: Option<String>, available: bool, reason: Option<String> });
dto!(NodeHello { protocol_version: u8, kind: String, runtime_version: String, capabilities: Vec<Capability>, protocol_versions: Vec<u8>, operations: Vec<String>, adapter_version: String, paseo_version: String, features: Vec<String> });
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloReply {
    pub node_id: String,
    pub control_ticket: String,
    pub control_ticket_expires_at: u64,
    pub selected_protocol_version: Option<u8>,
    #[serde(default)]
    pub commands: Vec<NodeCommand>,
    pub next_cursor: Option<String>,
}
dto!(ReconcileRequest { protocol_version: u8, kind: String, cursor: Option<String>, limit: u32 });
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileReply {
    #[serde(default)]
    pub commands: Vec<NodeCommand>,
    #[serde(default)]
    pub cancelled_execution_ids: Vec<String>,
    #[serde(default)]
    pub invalidated_permission_ids: Vec<String>,
    pub next_cursor: Option<String>,
}
dto!(ChildEvent { protocol_version: u8, kind: String, parent_execution_id: String, child_execution_id: String, event_id: String, state_version: u64, state: ExecutionState, result_refs: Vec<String> });
dto!(ChildrenRequest { protocol_version: u8, kind: String, parent_execution_id: String, cursor: Option<String>, limit: u32 });
dto!(ChildView { parent_execution_id: String, child_execution_id: String, state_version: u64, state: ExecutionState, result_refs: Vec<String> });
dto!(ChildrenReply { items: Vec<ChildView>, next_cursor: Option<String> });
dto!(DelegateInput {
    prompt: String,
    model: String,
    workdir_binding_id: String
});
dto!(DelegateRequest { protocol_version: u8, kind: String, command_id: String, parent_execution_id: String, target_node_id: String, capability: String, input: DelegateInput, input_refs: Vec<String> });
dto!(DelegateReply {
    child_execution_id: String
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_valid_commands() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/agent-v1/valid.json")).unwrap();
        for case in cases.as_array().unwrap() {
            if case["category"] != "command" {
                continue;
            }
            let body = serde_json::to_vec(&case["body"]).unwrap();
            let c = decode_command(case["route"].as_str().unwrap(), &body).unwrap();
            decode_command(
                case["route"].as_str().unwrap(),
                &serde_json::to_vec(&c).unwrap(),
            )
            .unwrap();
        }
    }
    #[test]
    fn shared_invalid_commands() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/agent-v1/invalid.json")).unwrap();
        for case in cases.as_array().unwrap() {
            assert!(
                decode_command(
                    case["route"].as_str().unwrap(),
                    &serde_json::to_vec(&case["body"]).unwrap()
                )
                .is_err(),
                "{}",
                case["name"]
            );
        }
    }
    #[test]
    fn limits_apply_to_bytes_not_characters() {
        assert!(decode_command("agent/session/inspect", &vec![b' '; MAX_BODY_BYTES + 1]).is_err());
    }
}

#[cfg(test)]
mod kotlin_interop {
    use super::*;
    #[test]
    fn actual_kotlin_hello_retains_legacy_fields_without_breaking_v1() {
        let h: HelloReply =
            serde_json::from_str(include_str!("../fixtures/agent-v1/kotlin-hello.json")).unwrap();
        assert_eq!(h.selected_protocol_version, Some(1));
        assert!(h.commands.is_empty());
    }
    #[test]
    fn actual_kotlin_cancel_allows_empty_private_scope() {
        let c = decode_command(
            "agent/execution/cancel",
            include_bytes!("../fixtures/agent-v1/kotlin-cancel.json"),
        )
        .unwrap();
        assert!(matches!(c, NodeCommand::Cancel(_)));
    }
}

#[cfg(test)]
mod complete_fixtures {
    use super::*;
    #[test]
    fn non_command_goldens_are_typed_too() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/agent-v1/valid.json")).unwrap();
        for c in cases.as_array().unwrap() {
            let b = c["body"].clone();
            match c["category"].as_str().unwrap() {
                "result" => {
                    serde_json::from_value::<NodeResult<serde_json::Value>>(b).unwrap();
                }
                "event" => {
                    serde_json::from_value::<NodeEvent>(b).unwrap();
                }
                "hello" => {
                    serde_json::from_value::<NodeHello>(b).unwrap();
                }
                "reconcile" => {
                    serde_json::from_value::<ReconcileRequest>(b).unwrap();
                }
                "child" => {
                    serde_json::from_value::<ChildEvent>(b).unwrap();
                }
                "delegate" => {
                    let d = serde_json::from_value::<DelegateRequest>(b).unwrap();
                    assert_eq!(d.input.workdir_binding_id, "root:0");
                }
                "command" => {}
                x => panic!("uncovered fixture {x}"),
            }
        }
    }
}

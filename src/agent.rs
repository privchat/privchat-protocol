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
    ($name:ident { $($(#[$attr:meta])* $field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $($(#[$attr])* pub $field: $ty),* }
    };
}
/// What one turn may touch. Chosen per turn by the server (development vs independent review),
/// because one persistent employee session can serve both. Absent means `workspace_write`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnAccess {
    WorkspaceWrite,
    ReadOnly,
}
/// Read or write, per directory a turn is given besides its start directory (WEEY_AGENT_PROTOCOL_SPEC §7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectoryAccess {
    Write,
    Read,
}
/// A directory the node advertised, given to one turn with its access (§7.3).
dto!(TurnDirectory { path: String, access: DirectoryAccess });
dto!(StartPayload { capability: String, agent_id: Option<String>, prompt: String, model: String, workdir_binding_id: String, parent_execution_id: Option<String>, input_refs: Option<Vec<String>>, #[serde(default, skip_serializing_if = "Option::is_none")] role_description: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] title: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] access: Option<TurnAccess>, #[serde(default, skip_serializing_if = "Option::is_none")] work_item_id: Option<String>,
    /// The turn's other directories, the complete set; absent means none (§7.3).
    #[serde(default, skip_serializing_if = "Option::is_none")] directories: Option<Vec<TurnDirectory>> });
dto!(SendPayload {
    text: String,
    message_id: String,
    continue_from_execution_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] role_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] access: Option<TurnAccess>,
    /// The work item title shown on the node for this turn; display only.
    #[serde(default, skip_serializing_if = "Option::is_none")] title: Option<String>,
    /// The work item this turn belongs to, so the node can group turns; display only, absent for private work.
    #[serde(default, skip_serializing_if = "Option::is_none")] work_item_id: Option<String>,
    /// This turn's other directories, the complete set; absent means none (§7.3).
    #[serde(default, skip_serializing_if = "Option::is_none")] directories: Option<Vec<TurnDirectory>>
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
                if c.payload
                    .role_description
                    .as_ref()
                    .is_some_and(|s| s.chars().count() > 4096)
                    || c.payload
                        .title
                        .as_ref()
                        .is_some_and(|s| s.chars().count() > 128)
                {
                    return Err("AGENT_EXECUTION_INVALID");
                }
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
                if c.payload
                    .role_description
                    .as_ref()
                    .is_some_and(|s| s.chars().count() > 4096)
                {
                    return Err("AGENT_EXECUTION_INVALID");
                }
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
    retryable: bool,
    /// Machine-readable facts of the refusal, e.g. the paths of AGENT_DIRECTORY_UNSUPPORTED (§7.3).
    #[serde(default, skip_serializing_if = "Option::is_none")] detail: Option<serde_json::Value>
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
/// Progress of one tool call; its updates share `call_id`. Never a business verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    InProgress,
    Completed,
    Failed,
}
/// What a tool call does, in Weey's own terms (the executor's tool names never reach a person). With it the
/// app words the step in the reader's language; the item's `text` is then only the concrete thing --
/// the command, path, query or address -- or absent when the executor has not said yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAction {
    Command,
    FileChange,
    Read,
    Search,
    Fetch,
    Other,
}
// One public item of a turn (WEEY_AGENT_PROTOCOL_SPEC §0.1/§0.5). `id` is the node's own stable id,
// never a native turn/item id; `tool_name` is a readable tool name or absent, never a backend label.
dto!(HistoryItem { id: String, kind: ItemKind,
    #[serde(default, skip_serializing_if = "Option::is_none")] text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] artifact_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] status: Option<ToolStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")] action: Option<ToolAction> });
impl HistoryItem {
    pub fn validate(&self) -> Result<(), &'static str> {
        let bad = |s: &str, max: usize| s.is_empty() || s.chars().count() > max || s.chars().any(char::is_control);
        if bad(&self.id, 128)
            || self.tool_name.as_deref().is_some_and(|t| bad(t, 64) || t.contains(':'))
            || self.call_id.as_deref().is_some_and(|c| bad(c, 128))
            || ((self.call_id.is_some() || self.status.is_some() || self.action.is_some()) && self.kind != ItemKind::ToolSummary)
        {
            return Err("AGENT_EXECUTION_INVALID");
        }
        Ok(())
    }
}
dto!(InspectResult { session_state: String, native_session_id: String, active_turn_id: Option<String>, pending_permission_ids: Vec<String> });
dto!(HistoryResult { items: Vec<HistoryItem>, next_cursor: Option<String>, snapshot_version: String });
dto!(WatchResult {
    subscription_id: String,
    expires_at: u64,
    base_seq: u64
});
/// How a turn ended; present on every terminal state (and on `unknown`), never otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnOutcome {
    Completed,
    Cancelled,
    Failed,
    Interrupted,
    Unknown,
}
dto!(StatePayload { state: ExecutionState, state_version: u64, native_session_id: Option<String>, turn_outcome: Option<TurnOutcome>, failure: Option<ExecutionFailure>,
    /// The cancel was not confirmed: the executor did not end the turn, or processes the turn
    /// started did not exit (WEEY_AGENT_PROTOCOL_SPEC §0.4.3). Only on `cancelling`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")] stop_unconfirmed: bool });
// Why an execution failed, as the node classified it. Only on `failed`; never the raw native error,
// environment or credentials. `summary` is short, single-line and redacted by the node.
dto!(ExecutionFailure { code: FailureCode, retryable: bool, summary: Option<String> });
pub const MAX_FAILURE_SUMMARY_CHARS: usize = 200;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCode {
    ExecutorNotLoggedIn,
    ExecutorQuotaExhausted,
    ExecutorVersionUnsupported,
    ExecutorUnavailable,
    ExecutorError,
}
impl StatePayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.stop_unconfirmed && self.state != ExecutionState::Cancelling {
            return Err("AGENT_EXECUTION_INVALID");
        }
        use ExecutionState as S;
        use TurnOutcome as O;
        let outcome_ok = match (&self.state, self.turn_outcome) {
            (S::Succeeded, Some(O::Completed)) | (S::Cancelled, Some(O::Cancelled)) | (S::Failed, Some(O::Failed)) => true,
            (S::Unknown, None | Some(O::Unknown | O::Interrupted)) => true,
            (S::Succeeded | S::Cancelled | S::Failed | S::Unknown, _) => false,
            (_, outcome) => outcome.is_none(),
        };
        if !outcome_ok {
            return Err("AGENT_EXECUTION_INVALID");
        }
        match &self.failure {
            None => Ok(()),
            Some(_) if self.state != ExecutionState::Failed => Err("AGENT_EXECUTION_INVALID"),
            Some(f) => match &f.summary {
                Some(s)
                    if s.chars().count() > MAX_FAILURE_SUMMARY_CHARS
                        || s.chars().any(char::is_control) =>
                {
                    Err("AGENT_EXECUTION_INVALID")
                }
                _ => Ok(()),
            },
        }
    }
}
dto!(OutputPayload { items: Vec<HistoryItem> });
/// What an approval asks for, in Weey's own terms (WEEY_AGENT_PROTOCOL_SPEC §0.5). The node converts
/// the native request in its adapter; no ACP/Codex field travels further.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionAction {
    Command,
    FileChange,
    Tool,
}
pub const MAX_PERMISSION_TITLE_CHARS: usize = 200;
pub const MAX_PERMISSION_COMMAND_CHARS: usize = 4000;
pub const MAX_PERMISSION_PATHS: usize = 50;
pub const MAX_PERMISSION_PATH_CHARS: usize = 1024;
dto!(PermissionSummary { action: PermissionAction, title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] cwd: Option<String>,
    #[serde(default)] paths: Vec<String>,
    /// Decided by the node: whether the owner can see enough of the request to allow it remotely.
    remote_approvable: bool });
impl PermissionSummary {
    pub fn validate(&self) -> Result<(), &'static str> {
        let control = |s: &str| s.chars().any(char::is_control);
        let title = self.title.chars().count();
        if title == 0 || title > MAX_PERMISSION_TITLE_CHARS || control(&self.title)
            || self.command.as_deref().is_some_and(|c| c.is_empty() || c.chars().count() > MAX_PERMISSION_COMMAND_CHARS)
            || self.cwd.as_deref().is_some_and(|c| c.is_empty() || c.chars().count() > MAX_PERMISSION_PATH_CHARS || control(c))
            || self.paths.len() > MAX_PERMISSION_PATHS
            || self.paths.iter().any(|p| p.is_empty() || p.chars().count() > MAX_PERMISSION_PATH_CHARS || control(p))
        {
            return Err("AGENT_EXECUTION_INVALID");
        }
        Ok(())
    }
}
dto!(PermissionRequested {
    permission_id: String,
    request_digest: String,
    expires_at: u64,
    summary: PermissionSummary,
    answers: Vec<PermissionDecision>
});
impl PermissionRequested {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.summary.validate()?;
        if self.answers.is_empty() || self.answers.len() > 2 {
            return Err("AGENT_EXECUTION_INVALID");
        }
        Ok(())
    }
}
/// Who settled an approval. A remote decision names the server command the node applied; the
/// server keeps its own record of who decided and does not take the node's word for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedBy {
    RemoteUser,
    LocalUser,
    Expired,
}
dto!(PermissionResolved {
    permission_id: String,
    decision: PermissionDecision,
    resolved_by: ResolvedBy,
    #[serde(default, skip_serializing_if = "Option::is_none")] command_id: Option<String>
});
impl PermissionResolved {
    pub fn validate(&self) -> Result<(), &'static str> {
        let ok = match self.resolved_by {
            ResolvedBy::RemoteUser => self.command_id.as_deref().is_some_and(|c| !c.is_empty() && c.len() <= 128),
            ResolvedBy::Expired => self.command_id.is_none() && self.decision == PermissionDecision::Deny,
            ResolvedBy::LocalUser => self.command_id.is_none(),
        };
        if ok { Ok(()) } else { Err("AGENT_EXECUTION_INVALID") }
    }
}
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
impl NodeEvent {
    /// Every payload rule of the event (the shapes themselves are enforced by decoding).
    pub fn validate(&self) -> Result<(), &'static str> {
        match &self.event {
            EventBody::State(p) => p.validate(),
            EventBody::Output(p) if p.items.is_empty() => Err("AGENT_EXECUTION_INVALID"),
            EventBody::Output(p) => p.items.iter().try_for_each(HistoryItem::validate),
            EventBody::PermissionRequested(p) => p.validate(),
            EventBody::PermissionResolved(p) => p.validate(),
            EventBody::HistoryInvalidated(_) | EventBody::Gap(_) => Ok(()),
        }
    }
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
dto!(Capability { name: String, version: Option<String>, available: bool, reason: Option<String>,
    /// The executor takes writable directories besides the start directory (§7.3).
    #[serde(default)] additional_directories: bool,
    /// The node keeps read-only directories unwritten for this executor (§7.3).
    #[serde(default)] read_only_directories: bool });
dto!(NodeWorkdir {
    id: String,
    label: String
});
dto!(NodeHello {
    protocol_version: u8,
    kind: String,
    runtime_version: String,
    capabilities: Vec<Capability>,
    protocol_versions: Vec<u8>,
    operations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    workdirs: Option<Vec<NodeWorkdir>>,
    /// Device description shown on the phone; it never authorizes anything (MODULE_AGENT_SPEC §5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    os: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    os_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    arch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    device_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    device_name: Option<String>
});
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
/// What a node does with a non-zero answer to one of its uplinks (WEEY_AGENT_PROTOCOL_SPEC §0.3). The single
/// classification every node uses; decided by the exact machine code, never by a prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UplinkDisposition {
    /// Temporary: send the same bytes again later, in order.
    Retry,
    /// This record is refused for good: keep it aside with its reason; the records behind it still flow.
    Quarantine,
    /// The node's own access credential failed: refresh it, then send again.
    RefreshCredentials,
    /// The node's binding itself is gone: stop new work and become unpaired.
    Unpaired,
    /// The execution's archive is full: the node compacts that record (existing archive-limit handling).
    ArchiveLimit,
    /// Not in this table: retry with backoff and record a local diagnostic. Never quarantined, never unpaired.
    Unknown,
}

/// Machine codes module-agent answers on node uplinks, by disposition. A code here decides alone, whatever the
/// numeric status it rides on (a refused record may carry PERMISSION_DENIED).
const UPLINK_RETRY: &[&str] = &[
    "AGENT_IM_UNAVAILABLE",
    "AGENT_SESSION_BUSY",
    "AGENT_EXECUTOR_RESTARTING",
    // The session's protocol is renegotiated by the next hello; the record itself is fine.
    "AGENT_PROTOCOL_UNSUPPORTED",
];
const UPLINK_QUARANTINE: &[&str] = &[
    "AGENT_EXECUTION_INVALID",
    "AGENT_EXECUTION_CONFLICT",
    "AGENT_EXECUTION_NOT_FOUND",
    "AGENT_PERMISSION_STALE",
    "AGENT_PAYLOAD_TOO_LARGE",
    "AGENT_CAPABILITY_NOT_GRANTED",
    "AGENT_OPERATION_UNSUPPORTED",
    "AGENT_COMMAND_EXPIRED",
    "AGENT_NODE_QUERY_NOT_FOUND",
    "AGENT_NODE_QUERY_INVALID",
];
const UPLINK_UNPAIRED: &[&str] = &["AGENT_NODE_REVOKED", "AGENT_NODE_NOT_FOUND"];
/// SYSTEM_BUSY, SERVICE_UNAVAILABLE, TIMEOUT, NETWORK_ERROR, RATE_LIMIT_EXCEEDED.
const UPLINK_RETRY_STATUS: &[i32] = &[2, 3, 5, 9, 10300];
/// AUTH_REQUIRED, INVALID_TOKEN, TOKEN_EXPIRED, TOKEN_REVOKED, SESSION_EXPIRED, SESSION_NOT_FOUND,
/// REFRESH_TOKEN_EXPIRED, REFRESH_TOKEN_REVOKED (not PERMISSION_DENIED 10004: that is never the credential).
const UPLINK_CREDENTIAL_STATUS: &[i32] = &[10000, 10001, 10002, 10003, 10005, 10006, 10009, 10010];

pub fn uplink_disposition(code: i32, message: &str) -> UplinkDisposition {
    if message == "AGENT_ARCHIVE_LIMIT" {
        return UplinkDisposition::ArchiveLimit;
    }
    if UPLINK_UNPAIRED.contains(&message) {
        return UplinkDisposition::Unpaired;
    }
    if UPLINK_QUARANTINE.contains(&message) {
        return UplinkDisposition::Quarantine;
    }
    if UPLINK_RETRY.contains(&message) {
        return UplinkDisposition::Retry;
    }
    if UPLINK_CREDENTIAL_STATUS.contains(&code) {
        return UplinkDisposition::RefreshCredentials;
    }
    if UPLINK_RETRY_STATUS.contains(&code) {
        return UplinkDisposition::Retry;
    }
    UplinkDisposition::Unknown
}

#[cfg(test)]
mod uplink_tests {
    use super::*;

    #[test]
    fn every_listed_code_has_one_disposition() {
        use UplinkDisposition::*;
        assert_eq!(uplink_disposition(10004, "AGENT_CAPABILITY_NOT_GRANTED"), Quarantine);
        assert_eq!(uplink_disposition(10100, "AGENT_EXECUTION_INVALID"), Quarantine);
        assert_eq!(uplink_disposition(10205, "AGENT_EXECUTION_CONFLICT"), Quarantine);
        assert_eq!(uplink_disposition(10200, "AGENT_NODE_REVOKED"), Unpaired);
        assert_eq!(uplink_disposition(10100, "AGENT_ARCHIVE_LIMIT"), ArchiveLimit);
        assert_eq!(uplink_disposition(3, "AGENT_IM_UNAVAILABLE"), Retry);
        assert_eq!(uplink_disposition(3, "SERVICE_UNAVAILABLE"), Retry);
        assert_eq!(uplink_disposition(10300, "RATE_LIMIT_EXCEEDED"), Retry);
        assert_eq!(uplink_disposition(10002, "TOKEN_EXPIRED"), RefreshCredentials);
        assert_eq!(uplink_disposition(10001, ""), RefreshCredentials);
        for (lists, d) in [(UPLINK_RETRY, Retry), (UPLINK_QUARANTINE, Quarantine), (UPLINK_UNPAIRED, Unpaired)] {
            for m in lists { assert_eq!(uplink_disposition(10100, m), d, "{m}"); }
        }
    }

    #[test]
    fn anything_not_in_the_table_is_unknown_never_quarantined_or_unpaired() {
        use UplinkDisposition::*;
        // An unlisted business code, whatever its prefix or status, is not guessed at.
        for (code, message) in [(10004, "AGENT_SOMETHING_NEW"), (10100, "AGENT_FUTURE_CODE"), (10004, "PERMISSION_DENIED"),
            (10004, "AGENT_ACTOR_FORBIDDEN"), (21501, "TRANSFER_SERVICE_UNREGISTERED"), (1, "SYSTEM_ERROR"), (0, "")] {
            assert_eq!(uplink_disposition(code, message), Unknown, "{code} {message}");
        }
    }
}

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
    fn terminal_states_carry_a_matching_turn_outcome() {
        let st = |v: serde_json::Value| serde_json::from_value::<StatePayload>(v).unwrap().validate();
        assert!(st(serde_json::json!({"state":"succeeded","stateVersion":1,"turnOutcome":"completed"})).is_ok());
        assert!(st(serde_json::json!({"state":"succeeded","stateVersion":1})).is_err(), "terminal without outcome");
        assert!(st(serde_json::json!({"state":"succeeded","stateVersion":1,"turnOutcome":"cancelled"})).is_err());
        assert!(st(serde_json::json!({"state":"unknown","stateVersion":1,"turnOutcome":"interrupted"})).is_ok());
        assert!(st(serde_json::json!({"state":"unknown","stateVersion":1,"turnOutcome":"completed"})).is_err());
        assert!(st(serde_json::json!({"state":"running","stateVersion":1,"turnOutcome":"completed"})).is_err());
        assert!(serde_json::from_value::<StatePayload>(serde_json::json!({"state":"succeeded","stateVersion":1,"turnOutcome":"done"})).is_err());
    }
    #[test]
    fn permissions_are_weey_shapes_with_bounded_fields() {
        let ok = serde_json::json!({"permissionId":"p","requestDigest":"d","expiresAt":1,"answers":["allow_once","deny"],
            "summary":{"action":"command","title":"Run tests","command":"make test","cwd":"/w","paths":[],"remoteApprovable":true}});
        assert!(serde_json::from_value::<PermissionRequested>(ok.clone()).unwrap().validate().is_ok());
        let mut native = ok.clone();
        native["summary"]["rawInput"] = serde_json::json!({"command":"make test"});
        assert!(serde_json::from_value::<PermissionRequested>(native).is_err(), "no native fields");
        let mut long = ok.clone();
        long["summary"]["title"] = serde_json::json!("x".repeat(201));
        assert!(serde_json::from_value::<PermissionRequested>(long).unwrap().validate().is_err());
        let mut lines = ok;
        lines["summary"]["title"] = serde_json::json!("a\nb");
        assert!(serde_json::from_value::<PermissionRequested>(lines).unwrap().validate().is_err());
        let r = |v: serde_json::Value| serde_json::from_value::<PermissionResolved>(v).unwrap().validate();
        assert!(r(serde_json::json!({"permissionId":"p","decision":"allow_once","resolvedBy":"remote_user","commandId":"cmd-1"})).is_ok());
        assert!(r(serde_json::json!({"permissionId":"p","decision":"allow_once","resolvedBy":"remote_user"})).is_err(), "remote names its command");
        assert!(r(serde_json::json!({"permissionId":"p","decision":"allow_once","resolvedBy":"expired"})).is_err(), "expiry only denies");
        assert!(serde_json::from_value::<PermissionResolved>(serde_json::json!({"permissionId":"p","decision":"deny","resolvedBy":"native"})).is_err());
    }
    #[test]
    fn items_carry_no_backend_labels() {
        let item = |v: serde_json::Value| serde_json::from_value::<HistoryItem>(v);
        assert!(item(serde_json::json!({"id":"9:ab","kind":"assistant_text","text":"t"})).unwrap().validate().is_ok());
        assert!(item(serde_json::json!({"id":"9:ab","kind":"assistant_text","toolName":"acp:message"})).unwrap().validate().is_err());
        assert!(item(serde_json::json!({"id":"9:ab","kind":"assistant_text","nativeKind":"reasoning"})).is_err());
        assert!(item(serde_json::json!({"id":"9:ab","kind":"assistant_text","callId":"c","status":"completed"})).unwrap().validate().is_err());
        assert!(item(serde_json::json!({"id":"9:ab","kind":"tool_summary","callId":"c","status":"completed"})).unwrap().validate().is_ok());
        assert!(item(serde_json::json!({"id":"9:ac","kind":"tool_summary","callId":"c","status":"in_progress","action":"command"})).unwrap().validate().is_ok());
        assert!(item(serde_json::json!({"id":"9:ad","kind":"assistant_text","action":"command"})).unwrap().validate().is_err(), "an action belongs to a tool call");
        assert!(item(serde_json::json!({"id":"9:ae","kind":"tool_summary","action":"bash"})).is_err(), "closed set");
    }
    #[test]
    fn stop_unconfirmed_is_only_a_cancelling_state() {
        let ok: StatePayload = serde_json::from_value(serde_json::json!({"state":"cancelling","stateVersion":4,"stopUnconfirmed":true})).unwrap();
        assert!(ok.validate().is_ok() && ok.stop_unconfirmed);
        let bad: StatePayload = serde_json::from_value(serde_json::json!({"state":"running","stateVersion":4,"stopUnconfirmed":true})).unwrap();
        assert_eq!(bad.validate(), Err("AGENT_EXECUTION_INVALID"));
        let plain: StatePayload = serde_json::from_value(serde_json::json!({"state":"cancelled","stateVersion":5,"turnOutcome":"cancelled"})).unwrap();
        assert!(!plain.stop_unconfirmed);
        assert!(!serde_json::to_string(&plain).unwrap().contains("stopUnconfirmed"), "absent when false");
    }
    #[test]
    fn a_follow_up_carries_the_work_item_title() {
        let body = serde_json::json!({"text":"go on","messageId":"m-1","continueFromExecutionId":"26","title":"Add subtract"});
        let p: SendPayload = serde_json::from_value(body).unwrap();
        assert_eq!(p.title.as_deref(), Some("Add subtract"));
    }
    #[test]
    fn turns_carry_their_work_item_id() {
        let start: StartPayload = serde_json::from_value(serde_json::json!({"capability":"agent.codex","prompt":"p","model":"m","workdirBindingId":"/w","workItemId":"10"})).unwrap();
        assert_eq!(start.work_item_id.as_deref(), Some("10"));
        let send: SendPayload = serde_json::from_value(serde_json::json!({"text":"go on","messageId":"m-1","continueFromExecutionId":"26","workItemId":"11"})).unwrap();
        assert_eq!(send.work_item_id.as_deref(), Some("11"));
        let private: SendPayload = serde_json::from_value(serde_json::json!({"text":"t","messageId":"m-2","continueFromExecutionId":"27"})).unwrap();
        assert_eq!(private.work_item_id, None);
        assert!(!serde_json::to_string(&private).unwrap().contains("workItemId"));
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
                    let e = serde_json::from_value::<NodeEvent>(b).unwrap_or_else(|e| panic!("{}: {e}", c["name"]));
                    e.validate().unwrap_or_else(|e| panic!("{}: {e}", c["name"]));
                }
                "hello" => {
                    serde_json::from_value::<NodeHello>(b).unwrap();
                }
                "reconcile" => {
                    serde_json::from_value::<ReconcileRequest>(b).unwrap();
                }
                "command" => {}
                x => panic!("uncovered fixture {x}"),
            }
        }
    }
}

#[cfg(test)]
mod failure {
    use super::*;
    fn state(v: serde_json::Value) -> StatePayload {
        serde_json::from_value(v).unwrap()
    }
    #[test]
    fn failure_is_bounded_and_only_on_failed() {
        let ok = state(
            serde_json::json!({"state": "failed", "stateVersion": 2, "turnOutcome": "failed", "failure": {"code": "executor_quota_exhausted", "retryable": true, "summary": "usage limit reached"}}),
        );
        assert!(ok.validate().is_ok());
        let on_success = state(
            serde_json::json!({"state": "succeeded", "stateVersion": 2, "turnOutcome": "completed", "failure": {"code": "executor_error", "retryable": false}}),
        );
        assert!(on_success.validate().is_err());
        let long = state(
            serde_json::json!({"state": "failed", "stateVersion": 2, "failure": {"code": "executor_error", "retryable": false, "summary": "x".repeat(201)}}),
        );
        assert!(long.validate().is_err());
        let multiline = state(
            serde_json::json!({"state": "failed", "stateVersion": 2, "failure": {"code": "executor_error", "retryable": false, "summary": "a\nb"}}),
        );
        assert!(multiline.validate().is_err());
        assert!(serde_json::from_value::<StatePayload>(serde_json::json!({"state": "failed", "stateVersion": 2, "failure": {"code": "raw_native_text", "retryable": false}})).is_err());
        assert!(serde_json::from_value::<StatePayload>(serde_json::json!({"state": "failed", "stateVersion": 2, "failure": {"code": "executor_error", "retryable": false, "env": {}}})).is_err());
        assert!(
            state(serde_json::json!({"state": "running", "stateVersion": 1}))
                .validate()
                .is_ok()
        );
    }
}

#[cfg(test)]
mod turn_access {
    use super::*;
    fn start(access: serde_json::Value) -> Result<NodeCommand, &'static str> {
        let body = serde_json::json!({"protocolVersion":1,"kind":"execution.start","commandId":"cmd-1","executionId":"1","scope":{"kind":"private","ref":""},
            "payload":{"capability":"agent.codex","prompt":"p","model":"m","workdirBindingId":"root:0","access":access}});
        decode_command("agent/execution/start", &serde_json::to_vec(&body).unwrap()).map_err(|_| "invalid")
    }
    #[test]
    fn access_is_optional_and_only_two_values_decode() {
        assert!(start(serde_json::json!("read_only")).is_ok());
        assert!(start(serde_json::json!("workspace_write")).is_ok());
        assert!(start(serde_json::json!("full_access")).is_err());
        let send = serde_json::json!({"protocolVersion":1,"kind":"session.send","commandId":"cmd-2","executionId":"2","scope":{"kind":"private","ref":""},
            "payload":{"text":"t","messageId":"m","continueFromExecutionId":"1","access":"read_only"}});
        assert!(decode_command("agent/session/send", &serde_json::to_vec(&send).unwrap()).is_ok());
        // Absent stays absent on the wire: older nodes see no new field.
        let plain = serde_json::json!({"protocolVersion":1,"kind":"execution.start","commandId":"cmd-3","executionId":"3","scope":{"kind":"private","ref":""},
            "payload":{"capability":"agent.codex","prompt":"p","model":"m","workdirBindingId":"root:0"}});
        let c = decode_command("agent/execution/start", &serde_json::to_vec(&plain).unwrap()).unwrap();
        assert!(!serde_json::to_string(&c).unwrap().contains("access"));
    }
}


#[cfg(test)]
mod directories {
    use super::*;

    #[test]
    fn turns_carry_their_directories_and_absent_means_none() {
        let start: StartPayload = serde_json::from_value(serde_json::json!({"capability":"agent.claude","prompt":"p","model":"m",
            "workdirBindingId":"/w/app","directories":[{"path":"/w/docs","access":"read"},{"path":"/w/server","access":"write"}]})).unwrap();
        let dirs = start.directories.clone().unwrap();
        assert_eq!(dirs[0], TurnDirectory { path: "/w/docs".into(), access: DirectoryAccess::Read });
        assert_eq!(dirs[1].access, DirectoryAccess::Write);
        let send: SendPayload = serde_json::from_value(serde_json::json!({"text":"t","messageId":"m","continueFromExecutionId":"1",
            "directories":[{"path":"/w/docs","access":"read"}]})).unwrap();
        assert_eq!(send.directories.unwrap().len(), 1);
        let plain: SendPayload = serde_json::from_value(serde_json::json!({"text":"t","messageId":"m","continueFromExecutionId":"1"})).unwrap();
        assert!(plain.directories.is_none());
        assert!(!serde_json::to_value(&plain).unwrap().as_object().unwrap().contains_key("directories"));
        // Unknown access is refused, not read as write.
        assert!(serde_json::from_value::<TurnDirectory>(serde_json::json!({"path":"/w","access":"full"})).is_err());
    }

    #[test]
    fn capabilities_default_to_no_extra_directories_and_errors_may_carry_detail() {
        let cap: Capability = serde_json::from_value(serde_json::json!({"name":"agent.codex","available":true})).unwrap();
        assert!(!cap.additional_directories && !cap.read_only_directories);
        let cap: Capability = serde_json::from_value(serde_json::json!({"name":"agent.claude","available":true,
            "additionalDirectories":true,"readOnlyDirectories":true})).unwrap();
        assert!(cap.additional_directories && cap.read_only_directories);
        let err: NodeError = serde_json::from_value(serde_json::json!({"message":"AGENT_DIRECTORY_UNSUPPORTED","retryable":false,
            "detail":{"paths":["/w/docs"]}})).unwrap();
        assert_eq!(err.detail.unwrap()["paths"][0], "/w/docs");
    }
}

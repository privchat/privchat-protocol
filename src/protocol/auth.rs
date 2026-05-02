//! Authorization handshake messages.

use super::{Message, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ------------------------------------------------------------------
// Owned types
// ------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub auth_type: AuthType,
    pub auth_token: String,
    pub client_info: ClientInfo,
    pub device_info: DeviceInfo,
    pub protocol_version: String,
    pub properties: HashMap<String, String>,
}

impl AuthorizationRequest {
    pub fn new() -> Self {
        Self {
            auth_type: AuthType::JWT,
            auth_token: String::new(),
            client_info: ClientInfo::default(),
            device_info: DeviceInfo::default(),
            protocol_version: crate::version::VERSION.to_string(),
            properties: HashMap::new(),
        }
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::AuthorizationRequest, self)
    }
}

impl Default for AuthorizationRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    pub success: bool,
    pub error_code: Option<u32>,
    pub error_message: Option<String>,
    pub session_id: Option<String>,
    pub user_id: Option<u64>,
    pub connection_id: Option<String>,
    pub server_info: Option<ServerInfo>,
    pub heartbeat_interval: Option<u64>,
}

impl AuthorizationResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::AuthorizationResponse, self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AuthType {
    Unspecified = 0,
    JWT = 1,
    UserPassword = 2,
    OAuth = 3,
    Anonymous = 4,
}

impl Default for AuthType {
    fn default() -> Self {
        AuthType::Unspecified
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client_type: String,
    pub version: String,
    pub os: String,
    pub os_version: String,
    pub device_model: Option<String>,
    pub app_package: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_type: DeviceType,
    pub app_id: String,
    pub push_token: Option<String>,
    pub push_channel: Option<String>,
    pub device_name: String,
    pub device_model: Option<String>,
    pub os_version: Option<String>,
    pub app_version: Option<String>,
    pub manufacturer: Option<String>,
    pub device_fingerprint: Option<String>,
}

/// Device platform. Unknown is the default value (FlatBuffers tag 0).
///
/// Serde encoding uses lowercase strings ("ios", "android", ...) for backward
/// compatibility with existing JSON consumers (HTTP APIs, presence service).
/// FlatBuffers encoding uses the numeric tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum DeviceType {
    #[serde(rename = "unknown")]
    Unknown = 0,
    #[serde(rename = "ios")]
    iOS = 1,
    #[serde(rename = "android")]
    Android = 2,
    #[serde(rename = "web")]
    Web = 3,
    #[serde(rename = "macos")]
    MacOS = 4,
    #[serde(rename = "windows")]
    Windows = 5,
    #[serde(rename = "linux")]
    Linux = 6,
    #[serde(rename = "iot")]
    IoT = 7,
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Unknown
    }
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::iOS => "ios",
            DeviceType::Android => "android",
            DeviceType::Web => "web",
            DeviceType::MacOS => "macos",
            DeviceType::Windows => "windows",
            DeviceType::Linux => "linux",
            DeviceType::IoT => "iot",
            DeviceType::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ios" => DeviceType::iOS,
            "android" => DeviceType::Android,
            "web" => DeviceType::Web,
            "macos" => DeviceType::MacOS,
            "windows" => DeviceType::Windows,
            "linux" | "freebsd" | "unix" => DeviceType::Linux,
            "iot" => DeviceType::IoT,
            _ => DeviceType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub name: String,
    pub features: Vec<String>,
    pub max_message_size: u64,
    pub connection_timeout: u64,
}

// ------------------------------------------------------------------
// Message trait
// ------------------------------------------------------------------

impl Message for AuthorizationRequest {
    fn message_type(&self) -> MessageType {
        MessageType::AuthorizationRequest
    }
}
impl Message for AuthorizationResponse {
    fn message_type(&self) -> MessageType {
        MessageType::AuthorizationResponse
    }
}

// ------------------------------------------------------------------
// FlatBuffers codec
// ------------------------------------------------------------------

fn auth_to_fb(t: AuthType) -> fb::AuthType {
    fb::AuthType(t as u8)
}
fn auth_from_fb(t: fb::AuthType) -> AuthType {
    match t.0 {
        1 => AuthType::JWT,
        2 => AuthType::UserPassword,
        3 => AuthType::OAuth,
        4 => AuthType::Anonymous,
        _ => AuthType::Unspecified,
    }
}
fn dev_to_fb(t: DeviceType) -> fb::DeviceType {
    fb::DeviceType(t as u8)
}
fn dev_from_fb(t: fb::DeviceType) -> DeviceType {
    match t.0 {
        1 => DeviceType::iOS,
        2 => DeviceType::Android,
        3 => DeviceType::Web,
        4 => DeviceType::MacOS,
        5 => DeviceType::Windows,
        6 => DeviceType::Linux,
        7 => DeviceType::IoT,
        _ => DeviceType::Unknown,
    }
}

fn encode_client_info<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    info: &ClientInfo,
) -> flatbuffers::WIPOffset<fb::ClientInfo<'a>> {
    let client_type = builder.create_string(&info.client_type);
    let version = builder.create_string(&info.version);
    let os = builder.create_string(&info.os);
    let os_version = builder.create_string(&info.os_version);
    let device_model = info.device_model.as_ref().map(|s| builder.create_string(s));
    let app_package = info.app_package.as_ref().map(|s| builder.create_string(s));

    fb::ClientInfo::create(
        builder,
        &fb::ClientInfoArgs {
            client_type: Some(client_type),
            version: Some(version),
            os: Some(os),
            os_version: Some(os_version),
            device_model,
            app_package,
        },
    )
}

fn decode_client_info(view: Option<fb::ClientInfo<'_>>) -> ClientInfo {
    let v = match view {
        Some(v) => v,
        None => return ClientInfo::default(),
    };
    ClientInfo {
        client_type: v.client_type().unwrap_or("").to_string(),
        version: v.version().unwrap_or("").to_string(),
        os: v.os().unwrap_or("").to_string(),
        os_version: v.os_version().unwrap_or("").to_string(),
        device_model: v.device_model().map(|s| s.to_string()),
        app_package: v.app_package().map(|s| s.to_string()),
    }
}

fn encode_device_info<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    info: &DeviceInfo,
) -> flatbuffers::WIPOffset<fb::DeviceInfo<'a>> {
    let device_id = builder.create_string(&info.device_id);
    let app_id = builder.create_string(&info.app_id);
    let push_token = info.push_token.as_ref().map(|s| builder.create_string(s));
    let push_channel = info.push_channel.as_ref().map(|s| builder.create_string(s));
    let device_name = builder.create_string(&info.device_name);
    let device_model = info.device_model.as_ref().map(|s| builder.create_string(s));
    let os_version = info.os_version.as_ref().map(|s| builder.create_string(s));
    let app_version = info.app_version.as_ref().map(|s| builder.create_string(s));
    let manufacturer = info.manufacturer.as_ref().map(|s| builder.create_string(s));
    let device_fingerprint = info
        .device_fingerprint
        .as_ref()
        .map(|s| builder.create_string(s));

    fb::DeviceInfo::create(
        builder,
        &fb::DeviceInfoArgs {
            device_id: Some(device_id),
            device_type: dev_to_fb(info.device_type),
            app_id: Some(app_id),
            push_token,
            push_channel,
            device_name: Some(device_name),
            device_model,
            os_version,
            app_version,
            manufacturer,
            device_fingerprint,
        },
    )
}

fn decode_device_info(view: Option<fb::DeviceInfo<'_>>) -> DeviceInfo {
    let v = match view {
        Some(v) => v,
        None => return DeviceInfo::default(),
    };
    DeviceInfo {
        device_id: v.device_id().unwrap_or("").to_string(),
        device_type: dev_from_fb(v.device_type()),
        app_id: v.app_id().unwrap_or("").to_string(),
        push_token: v.push_token().map(|s| s.to_string()),
        push_channel: v.push_channel().map(|s| s.to_string()),
        device_name: v.device_name().unwrap_or("").to_string(),
        device_model: v.device_model().map(|s| s.to_string()),
        os_version: v.os_version().map(|s| s.to_string()),
        app_version: v.app_version().map(|s| s.to_string()),
        manufacturer: v.manufacturer().map(|s| s.to_string()),
        device_fingerprint: v.device_fingerprint().map(|s| s.to_string()),
    }
}

fn encode_server_info<'a>(
    builder: &mut FlatBufferBuilder<'a>,
    info: &ServerInfo,
) -> flatbuffers::WIPOffset<fb::ServerInfo<'a>> {
    let version = builder.create_string(&info.version);
    let name = builder.create_string(&info.name);
    let feature_offsets: Vec<_> = info
        .features
        .iter()
        .map(|f| builder.create_string(f))
        .collect();
    let features = builder.create_vector(&feature_offsets);

    fb::ServerInfo::create(
        builder,
        &fb::ServerInfoArgs {
            version: Some(version),
            name: Some(name),
            features: Some(features),
            max_message_size: info.max_message_size,
            connection_timeout: info.connection_timeout,
        },
    )
}

fn decode_server_info(view: Option<fb::ServerInfo<'_>>) -> Option<ServerInfo> {
    let v = view?;
    Some(ServerInfo {
        version: v.version().unwrap_or("").to_string(),
        name: v.name().unwrap_or("").to_string(),
        features: v
            .features()
            .map(|vec| vec.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        max_message_size: v.max_message_size(),
        connection_timeout: v.connection_timeout(),
    })
}

impl FlatBufferMessage for AuthorizationRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let auth_token = builder.create_string(&self.auth_token);
        let client_info = encode_client_info(builder, &self.client_info);
        let device_info = encode_device_info(builder, &self.device_info);
        let protocol_version = builder.create_string(&self.protocol_version);

        let property_offsets: Vec<_> = self
            .properties
            .iter()
            .map(|(k, v)| {
                let key = builder.create_string(k);
                let value = builder.create_string(v);
                fb::Property::create(
                    builder,
                    &fb::PropertyArgs {
                        key: Some(key),
                        value: Some(value),
                    },
                )
            })
            .collect();
        let properties = builder.create_vector(&property_offsets);

        let args = fb::AuthorizationRequestArgs {
            auth_type: auth_to_fb(self.auth_type),
            auth_token: Some(auth_token),
            client_info: Some(client_info),
            device_info: Some(device_info),
            protocol_version: Some(protocol_version),
            properties: Some(properties),
        };
        let offset = fb::AuthorizationRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::AuthorizationRequest>(bytes)?;
        let properties: HashMap<String, String> = view
            .properties()
            .map(|vec| {
                vec.iter()
                    .map(|p| {
                        (
                            p.key().unwrap_or("").to_string(),
                            p.value().unwrap_or("").to_string(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            auth_type: auth_from_fb(view.auth_type()),
            auth_token: view.auth_token().unwrap_or("").to_string(),
            client_info: decode_client_info(view.client_info()),
            device_info: decode_device_info(view.device_info()),
            protocol_version: view.protocol_version().unwrap_or("").to_string(),
            properties,
        })
    }
}

impl FlatBufferMessage for AuthorizationResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let error_message = self.error_message.as_ref().map(|s| builder.create_string(s));
        let session_id = self.session_id.as_ref().map(|s| builder.create_string(s));
        let connection_id = self.connection_id.as_ref().map(|s| builder.create_string(s));
        let server_info = self
            .server_info
            .as_ref()
            .map(|si| encode_server_info(builder, si));

        let args = fb::AuthorizationResponseArgs {
            success: self.success,
            error_code: self.error_code.unwrap_or(0),
            error_message,
            session_id,
            user_id: self.user_id.unwrap_or(0),
            connection_id,
            server_info,
            heartbeat_interval: self.heartbeat_interval.unwrap_or(0),
        };
        let offset = fb::AuthorizationResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::AuthorizationResponse>(bytes)?;
        let error_code = match view.error_code() {
            0 => None,
            n => Some(n),
        };
        let user_id = match view.user_id() {
            0 => None,
            n => Some(n),
        };
        let heartbeat_interval = match view.heartbeat_interval() {
            0 => None,
            n => Some(n),
        };
        Ok(Self {
            success: view.success(),
            error_code,
            error_message: view.error_message().map(|s| s.to_string()),
            session_id: view.session_id().map(|s| s.to_string()),
            user_id,
            connection_id: view.connection_id().map(|s| s.to_string()),
            server_info: decode_server_info(view.server_info()),
            heartbeat_interval,
        })
    }
}

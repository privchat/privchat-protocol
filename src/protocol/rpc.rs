//! Generic RPC envelope. `body` and `data` are opaque application bytes —
//! per-route encoding is decided by the RPC layer.

use super::{Message, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpcRequest {
    pub route: String,
    pub body: Vec<u8>,
}

impl RpcRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::RpcRequest, self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpcResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<Vec<u8>>,
}

impl RpcResponse {
    pub fn new() -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: None,
        }
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::RpcResponse, self)
    }

    pub fn success(data: Vec<u8>) -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: Some(data),
        }
    }

    pub fn success_empty() -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: None,
        }
    }

    pub fn error(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    #[inline]
    pub fn is_ok(&self) -> bool {
        self.code == 0
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        self.code != 0
    }
}

impl Message for RpcRequest {
    fn message_type(&self) -> MessageType {
        MessageType::RpcRequest
    }
}
impl Message for RpcResponse {
    fn message_type(&self) -> MessageType {
        MessageType::RpcResponse
    }
}

impl FlatBufferMessage for RpcRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let route = builder.create_string(&self.route);
        let body = builder.create_vector(&self.body);
        let args = fb::RpcRequestArgs {
            route: Some(route),
            body: Some(body),
        };
        let offset = fb::RpcRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::RpcRequest>(bytes)?;
        Ok(Self {
            route: view.route().unwrap_or("").to_string(),
            body: view.body().map(|v| v.bytes().to_vec()).unwrap_or_default(),
        })
    }
}

impl FlatBufferMessage for RpcResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let message = builder.create_string(&self.message);
        // None encodes as empty [ubyte]; decoder distinguishes by length.
        let data = match &self.data {
            Some(d) => builder.create_vector(d),
            None => builder.create_vector::<u8>(&[]),
        };
        let args = fb::RpcResponseArgs {
            code: self.code,
            message: Some(message),
            data: Some(data),
        };
        let offset = fb::RpcResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::RpcResponse>(bytes)?;
        let data = view.data().map(|v| v.bytes().to_vec()).and_then(|v| {
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        });
        Ok(Self {
            code: view.code(),
            message: view.message().unwrap_or("").to_string(),
            data,
        })
    }
}

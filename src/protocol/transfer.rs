//! Channel Transfer envelope. Bidirectional channel-scoped RPC; see
//! `protocol/transfer.fbs` and `02-server/CHANNEL_TRANSFER_SPEC.md` v2.0.

use super::{Message, MessageType, Packet};
use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferRequest {
    pub request_id: String,
    pub channel_id: u64,
    pub route: String,
    pub body: Vec<u8>,
}

impl TransferRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::TransferRequest, self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferResponse {
    pub request_id: String,
    pub channel_id: u64,
    pub code: i32,
    pub message: String,
    pub data: Option<Vec<u8>>,
}

impl TransferResponse {
    pub fn new() -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            ..Self::default()
        }
    }

    pub fn create_packet(self) -> Packet<Self> {
        Packet::new(MessageType::TransferResponse, self)
    }

    pub fn success(request_id: String, channel_id: u64, data: Vec<u8>) -> Self {
        Self {
            request_id,
            channel_id,
            code: 0,
            message: "OK".to_string(),
            data: Some(data),
        }
    }

    pub fn success_empty(request_id: String, channel_id: u64) -> Self {
        Self {
            request_id,
            channel_id,
            code: 0,
            message: "OK".to_string(),
            data: None,
        }
    }

    pub fn error(request_id: String, channel_id: u64, code: i32, message: String) -> Self {
        Self {
            request_id,
            channel_id,
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

impl Message for TransferRequest {
    fn message_type(&self) -> MessageType {
        MessageType::TransferRequest
    }
}

impl Message for TransferResponse {
    fn message_type(&self) -> MessageType {
        MessageType::TransferResponse
    }
}

impl FlatBufferMessage for TransferRequest {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let request_id = builder.create_string(&self.request_id);
        let route = builder.create_string(&self.route);
        let body = builder.create_vector(&self.body);
        let args = fb::TransferRequestArgs {
            request_id: Some(request_id),
            channel_id: self.channel_id,
            route: Some(route),
            body: Some(body),
        };
        let offset = fb::TransferRequest::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::TransferRequest>(bytes)?;
        Ok(Self {
            request_id: view.request_id().unwrap_or("").to_string(),
            channel_id: view.channel_id(),
            route: view.route().unwrap_or("").to_string(),
            body: view.body().map(|v| v.bytes().to_vec()).unwrap_or_default(),
        })
    }
}

impl FlatBufferMessage for TransferResponse {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        let request_id = builder.create_string(&self.request_id);
        let message = builder.create_string(&self.message);
        // None encodes as empty [ubyte]; decoder distinguishes by length.
        let data = match &self.data {
            Some(d) => builder.create_vector(d),
            None => builder.create_vector::<u8>(&[]),
        };
        let args = fb::TransferResponseArgs {
            request_id: Some(request_id),
            channel_id: self.channel_id,
            code: self.code,
            message: Some(message),
            data: Some(data),
        };
        let offset = fb::TransferResponse::create(builder, &args);
        builder.finish(offset, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::TransferResponse>(bytes)?;
        let data = view.data().map(|v| v.bytes().to_vec()).and_then(|v| {
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        });
        Ok(Self {
            request_id: view.request_id().unwrap_or("").to_string(),
            channel_id: view.channel_id(),
            code: view.code(),
            message: view.message().unwrap_or("").to_string(),
            data,
        })
    }
}

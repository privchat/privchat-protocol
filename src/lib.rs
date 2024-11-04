pub mod encoder;
pub mod decoder;
pub mod message;
pub mod security;

use md5::{Digest, Md5};
use uuid::Uuid;
use num_bigint::BigUint;
use std::str::FromStr;
use num_traits::ToPrimitive;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use base64::{engine::general_purpose, Engine as _};

use crate::decoder::Decoder;
use crate::encoder::Encoder;
use crate::message::{
    ConnectAckMessage, ConnectMessage, DisconnectMessage, Packet, StreamFlag,
    RecvAckMessage, RecvMessage, SendAckMessage, SendMessage, Setting, SubscribeAckMessage, SubscribeMessage,
};

use crate::security::SecurityManager;

static mut PROTOCOL_VERSION: u8 = 0; // 服务端返回的协议版本

#[derive(Clone)]
pub struct Protocol {
    message_encode_map: HashMap<TypeId, fn(&dyn Any) -> Vec<u8>>,
    packet_decode_map: HashMap<TypeId, fn(&mut Decoder) -> Box<dyn Any>>,
}

impl Protocol {
    pub fn new() -> Self {
        let mut message_encode_map: HashMap<TypeId, fn(&dyn Any) -> Vec<u8>> = HashMap::new();
        message_encode_map.insert(TypeId::of::<ConnectMessage>(), Self::encode_connect_message);
        message_encode_map.insert(TypeId::of::<ConnectAckMessage>(), Self::encode_connect_ack_message);
        message_encode_map.insert(TypeId::of::<DisconnectMessage>(), Self::encode_disconnect_message);
        message_encode_map.insert(TypeId::of::<SendMessage>(), Self::encode_send_message);
        message_encode_map.insert(TypeId::of::<SendAckMessage>(), Self::encode_send_ack_message);
        message_encode_map.insert(TypeId::of::<RecvMessage>(), Self::encode_recv_message);
        message_encode_map.insert(TypeId::of::<RecvAckMessage>(), Self::encode_recv_ack_message);
        message_encode_map.insert(TypeId::of::<SubscribeMessage>(), Self::encode_subscribe_message);
        message_encode_map.insert(TypeId::of::<SubscribeAckMessage>(), Self::encode_subscribe_ack_message);

        let mut packet_decode_map: HashMap<TypeId, fn(&mut Decoder) -> Box<dyn Any>> = HashMap::new();
        packet_decode_map.insert(TypeId::of::<ConnectMessage>(), Self::decode_connect_message);
        packet_decode_map.insert(TypeId::of::<ConnectAckMessage>(), Self::decode_connect_ack_message);
        packet_decode_map.insert(TypeId::of::<SendMessage>(), Self::decode_send_message);
        packet_decode_map.insert(TypeId::of::<SendAckMessage>(), Self::decode_send_ack_message);
        packet_decode_map.insert(TypeId::of::<RecvMessage>(), Self::decode_recv_message);
        packet_decode_map.insert(TypeId::of::<RecvAckMessage>(), Self::decode_recv_ack_message);
        packet_decode_map.insert(TypeId::of::<DisconnectMessage>(), Self::decode_disconnect_message);
        packet_decode_map.insert(TypeId::of::<SubscribeMessage>(), Self::decode_subscribe_message);
        packet_decode_map.insert(TypeId::of::<SubscribeAckMessage>(), Self::decode_subscribe_ack_message);

        Self {
            message_encode_map,
            packet_decode_map,
        }
    }
    
    pub fn encode<T: Any>(&self, message: &T) -> Vec<u8> {
        let encode_func = self.message_encode_map.get(&TypeId::of::<T>()).unwrap();
        encode_func(message)
    }

    pub fn decode<T: Any>(&self, data: &[u8]) -> Option<T> {
        let decode_func = self.packet_decode_map.get(&TypeId::of::<T>()).unwrap();
        let boxed_any = decode_func(&mut Decoder::new(data.to_vec()));

        boxed_any.downcast::<T>().ok().map(|boxed| *boxed)
    }

    fn encode_connect_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let p = message.downcast_ref::<ConnectMessage>().unwrap();
        enc.write_uint8(p.version);
        enc.write_uint8(p.device_flag);
        enc.write_string(&p.device_id);
        enc.write_string(&p.uid);
        enc.write_string(&p.token);
        enc.write_int64(&BigUint::from(p.client_timestamp as u64));
        enc.write_string(&p.client_key);
        enc.to_uint8_array()
    }

    fn encode_send_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let mut p = message.downcast_ref::<SendMessage>().unwrap().clone(); // 克隆 SendMessage
    
        enc.write_uint8(p.setting.to_uint8());
        enc.write_int32(p.client_seq as i32);
        if p.client_msg_no.is_empty() {
            p.client_msg_no = Uuid::new_v4().to_string();
        }
        enc.write_string(&p.client_msg_no);
    
        if p.setting.stream_on() {
            enc.write_string(&p.stream_no);
        }
    
        enc.write_string(&p.channel_id);
        enc.write_uint8(p.channel_type);
        if unsafe { PROTOCOL_VERSION } >= 3 {
            enc.write_int32(p.expire.unwrap_or(0) as i32);
        }
    
        let mut hasher = Md5::new();
        hasher.update(&p.verify_string().as_bytes());
        enc.write_string(&format!("{:x}", hasher.finalize()));
    
        if p.setting.topic {
            enc.write_string(p.topic.as_deref().unwrap_or(""));
        }

        enc.write_string(std::str::from_utf8(&p.payload).unwrap());
    
        enc.to_uint8_array()
    }

    fn decode_send_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = SendMessage::new();
        
        // 读取并设置各字段的值
        p.setting = Setting::from_uint8(decoder.read_byte());
        p.client_seq = decoder.read_int32() as u32;
        p.client_msg_no = decoder.read_string();
    
        if p.setting.stream_on() {
            p.stream_no = decoder.read_string();
        }
    
        p.channel_id = decoder.read_string();
        p.channel_type = decoder.read_byte();
    
        if unsafe { PROTOCOL_VERSION } >= 3 {
            p.expire = Some(decoder.read_int32() as u32);
        }
    
        // 读取并处理 msg_key 的哈希值
        let _msg_key_hash = decoder.read_string();
        // `msg_key` 的值仅用于加密过程，不需要在 `decode_send_message` 中直接解密或处理。
    
        // 读取并处理 topic（如果存在）
        if p.setting.topic {
            p.topic = Some(decoder.read_string());
        }

        p.payload = decoder.read_string().into_bytes();
    
        Box::new(p)
    }

    fn encode_send_ack_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let p = message.downcast_ref::<SendAckMessage>().unwrap();
        
        enc.write_int32(p.client_seq as i32);
        enc.write_int64(&p.message_id.to_biguint().unwrap());
        enc.write_int32(p.message_seq as i32);
        enc.write_uint8(p.reason_code);
        
        enc.to_uint8_array()
    }

    fn encode_recv_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let p = message.downcast_ref::<RecvMessage>().unwrap();
    
        enc.write_uint8(p.setting.to_uint8());
        enc.write_string(&p.msg_key);
        enc.write_string(&p.from_uid);
        enc.write_string(&p.channel_id);
        enc.write_uint8(p.channel_type);
        
        if unsafe { PROTOCOL_VERSION } >= 3 {
            if let Some(expire) = p.expire {
                enc.write_int32(expire as i32);
            }
        }
        
        enc.write_string(&p.client_msg_no);
        
        if p.setting.stream_on() {
            enc.write_string(&p.stream_no);
            enc.write_int32(p.stream_seq as i32);
            enc.write_uint8(p.stream_flag as u8);
        }
        
        // Convert message_id from String to BigUint using `from_str_radix`
        let message_id_biguint = BigUint::from_str(&p.message_id).unwrap();
        enc.write_int64(&message_id_biguint);
        enc.write_int32(p.message_seq as i32);
        enc.write_int32(p.timestamp as i32);
        
        if p.setting.topic {
            enc.write_string(p.topic.as_deref().unwrap_or(""));
        }
        
        enc.write_bytes(&p.payload);
        
        enc.to_uint8_array()
    }

    fn encode_subscribe_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let p = message.downcast_ref::<SubscribeMessage>().unwrap();
        enc.write_uint8(p.setting);
        enc.write_string(&p.client_msg_no);
        enc.write_string(&p.channel_id);
        enc.write_uint8(p.channel_type);
        enc.write_uint8(p.action);
        enc.write_string(p.param.as_deref().unwrap_or(""));
        enc.to_uint8_array()
    }
    
    fn encode_subscribe_ack_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let p = message.downcast_ref::<SubscribeAckMessage>().unwrap();
        
        enc.write_string(&p.client_msg_no);
        enc.write_string(&p.channel_id);
        enc.write_uint8(p.channel_type);
        enc.write_uint8(p.action);
        enc.write_uint8(p.reason_code);
        
        enc.to_uint8_array()
    }

    fn encode_recv_ack_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let p = message.downcast_ref::<RecvAckMessage>().unwrap();
        let message_id_biguint = BigUint::parse_bytes(p.message_id.as_bytes(), 10).unwrap();
        enc.write_int64(&message_id_biguint);
        enc.write_int32(p.message_seq as i32);
        enc.to_uint8_array()
    }

    // 编码 DisconnectMessage
    fn encode_disconnect_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let p = message.downcast_ref::<DisconnectMessage>().unwrap();
        enc.write_uint8(p.reason_code);
        enc.write_string(&p.reason);
        enc.to_uint8_array()
    }

    // 编码 ConnectAckMessage
    fn encode_connect_ack_message(message: &dyn Any) -> Vec<u8> {
        let mut enc = Encoder::new();
        let p = message.downcast_ref::<ConnectAckMessage>().unwrap();
        enc.write_uint8(p.protocol_version);
        
        // 将 time_diff 从 BigInt 转换为 BigUint
        let time_diff = p.time_diff.to_biguint().unwrap_or_else(|| BigUint::from(0u64));
        enc.write_int64(&time_diff);
        
        enc.write_uint8(p.reason_code);
        enc.write_string(&p.server_key);
        enc.write_string(&p.salt);
        
        if p.protocol_version >= 4 {
            // 将 node_id 从 BigInt 转换为 BigUint
            let node_id = p.node_id.to_biguint().unwrap_or_else(|| BigUint::from(0u64));
            enc.write_int64(&node_id);
        }
        
        enc.to_uint8_array()
    }

    // 解码 SubscribeMessage
    fn decode_subscribe_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = SubscribeMessage::new();
        p.setting = decoder.read_byte();
        p.client_msg_no = decoder.read_string();
        p.channel_id = decoder.read_string();
        p.channel_type = decoder.read_byte();
        p.action = decoder.read_byte();
        p.param = Some(decoder.read_string());
        Box::new(p)
    }

    // 解码 RecvAckMessage
    fn decode_recv_ack_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = RecvAckMessage::new();
        p.message_id = decoder.read_int64().to_string();
        p.message_seq = decoder.read_int32();
        Box::new(p)
    }

    fn decode_connect_ack_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = ConnectAckMessage::new();

        p.protocol_version = decoder.read_byte();
        unsafe {
            PROTOCOL_VERSION = p.protocol_version;
        }

        p.time_diff = decoder.read_int64().into();
        p.reason_code = decoder.read_byte();
        p.server_key = decoder.read_string();
        p.salt = decoder.read_string();
        if p.protocol_version >= 4 {
            p.node_id = decoder.read_int64().into();
        }
        Box::new(p)
    }

    fn decode_connect_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = ConnectMessage::new();

        p.version = decoder.read_byte();
        p.device_flag = decoder.read_byte();
        p.device_id = decoder.read_string();
        p.uid = decoder.read_string();
        p.token = decoder.read_string();
        p.client_timestamp = decoder.read_int64().to_u64().unwrap() as i64;
        p.client_key = decoder.read_string();
        Box::new(p)
    }

    fn decode_disconnect_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = DisconnectMessage::new();
        p.reason_code = decoder.read_byte();
        p.reason = decoder.read_string();
        Box::new(p)
    }

    fn decode_recv_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = RecvMessage::new();
        p.setting = Setting::from_uint8(decoder.read_byte());
        p.msg_key = decoder.read_string();
        p.from_uid = decoder.read_string();
        p.channel_id = decoder.read_string();
        p.channel_type = decoder.read_byte();
        if unsafe { PROTOCOL_VERSION } >= 3 {
            p.expire = Some(decoder.read_int32() as u32);
        }
        p.client_msg_no = decoder.read_string();
        if p.setting.stream_on() {
            p.stream_no = decoder.read_string();
            p.stream_seq = decoder.read_int32() as u32;
            p.stream_flag = StreamFlag::from(decoder.read_byte());
        }
        p.message_id = decoder.read_int64().to_string();
        p.message_seq = decoder.read_int32() as u32;
        p.timestamp = decoder.read_int32() as u32;
        if p.setting.topic {
            p.topic = Some(decoder.read_string());
        }
        p.payload = decoder.read_remaining();
        Box::new(p)
    }

    fn decode_send_ack_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = SendAckMessage::new();
        p.message_id = decoder.read_int64().into();
        p.client_seq = decoder.read_int32();
        p.message_seq = decoder.read_int32();
        p.reason_code = decoder.read_byte();
        Box::new(p)
    }

    fn decode_subscribe_ack_message(decoder: &mut Decoder) -> Box<dyn Any> {
        let mut p = SubscribeAckMessage::new();

        p.client_msg_no = decoder.read_string();
        p.channel_id = decoder.read_string();
        p.channel_type = decoder.read_byte();
        p.action = decoder.read_byte();
        p.reason_code = decoder.read_byte();
        Box::new(p)
    }

    fn encode_bool(b: bool) -> u8 {
        if b {
            1
        } else {
            0
        }
    }

    fn encode_variable_length(mut len: usize) -> Vec<u8> {
        let mut ret = Vec::new();
        while len > 0 {
            let mut digit = len % 0x80;
            len /= 0x80;
            if len > 0 {
                digit |= 0x80;
            }
            ret.push(digit as u8);
        }
        ret
    }
}

impl<T: Any> Packet<T> {
    pub fn as_any(&self) -> &dyn Any {
        &self.message_object
    }
}

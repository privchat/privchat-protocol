/// PrivChat 协议库
/// 
/// PrivChat 协议消息定义和编解码工具
/// 
/// 使用示例：
/// ```
/// use privchat_protocol::protocol::*;
/// 
/// fn main() {
///     let mut connect_req = AuthorizationRequest::new();
///     connect_req.auth_token = "test_token".to_string();
///     let packet = connect_req.create_packet();
///     println!("消息类型: {:?}", packet.message_type);
/// }
/// ```

// 导出主要模块
pub mod protocol;
pub mod message;
pub mod rpc;
pub mod presence;
pub mod notification;
pub mod error_code;
pub mod version;

pub use protocol::*;
pub use message::*;
pub use presence::*;
pub use notification::*;
pub use error_code::ErrorCode;
pub use version::{VERSION, PROTOCOL_VERSION};

/// 解码消息的通用函数
pub fn decode_message<T>(payload: &[u8]) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let json_str = String::from_utf8(payload.to_vec())?;
    let message: T = serde_json::from_str(&json_str)?;
    Ok(message)
}

/// 编码消息的通用函数
pub fn encode_message<T>(message: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let json_str = serde_json::to_string(message)?;
    Ok(json_str.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_message_creation() {
        // 测试连接消息
        let mut connect_req = AuthorizationRequest::new();
        connect_req.auth_token = "test_token".to_string();
        
        let packet = connect_req.create_packet();
        assert_eq!(packet.message_type, MessageType::AuthorizationRequest);
    }
    
    #[test]
    fn test_message_types() {
        // 确保所有消息类型都有正确的值
        assert_eq!(MessageType::AuthorizationRequest as u8, 1);
        assert_eq!(MessageType::AuthorizationResponse as u8, 2);
        assert_eq!(MessageType::DisconnectRequest as u8, 3);
        assert_eq!(MessageType::DisconnectResponse as u8, 4);
        assert_eq!(MessageType::SendMessageRequest as u8, 5);
        assert_eq!(MessageType::SendMessageResponse as u8, 6);
        assert_eq!(MessageType::PushMessageRequest as u8, 7);
        assert_eq!(MessageType::PushMessageResponse as u8, 8);
        assert_eq!(MessageType::PushBatchRequest as u8, 9);        // 批量接收消息
        assert_eq!(MessageType::PushBatchResponse as u8, 10);     // 批量接收确认
        assert_eq!(MessageType::PingRequest as u8, 11);
        assert_eq!(MessageType::PongResponse as u8, 12);
        assert_eq!(MessageType::SubscribeRequest as u8, 13);
        assert_eq!(MessageType::SubscribeResponse as u8, 14);
        assert_eq!(MessageType::PublishRequest as u8, 15);          // 推送消息
        assert_eq!(MessageType::PublishResponse as u8, 16);       // 推送确认
    }
    
    #[test]
    fn test_send_message() {
        let mut send_req = SendMessageRequest::new();
        send_req.from_uid = 123;
        send_req.channel_id = 456;
        send_req.payload = "Hello World".as_bytes().to_vec();
        
        let packet = send_req.create_packet();
        assert_eq!(packet.message_type, MessageType::SendMessageRequest);
    }
    
    #[test]
    fn test_recv_message() {
        let mut recv_req = PushMessageRequest::new();
        recv_req.from_uid = 456;
        recv_req.channel_id = 789;
        recv_req.server_message_id = 100;
        recv_req.payload = "Hello Back".as_bytes().to_vec();
        
        let packet = recv_req.create_packet();
        assert_eq!(packet.message_type, MessageType::PushMessageRequest);
    }
}

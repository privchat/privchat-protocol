/// PrivChat 协议库
/// 
/// PrivChat 协议消息定义和编解码工具
/// 
/// 使用示例：
/// ```
/// use privchat_protocol::message::*;
/// 
/// fn main() {
///     // 创建连接消息
///     let mut connect_req = ConnectRequest::new();
///     connect_req.uid = "user123".to_string();
///     connect_req.token = "test_token".to_string();
///     
///     // 创建包
///     let packet = connect_req.create_packet();
///     println!("消息类型: {:?}", packet.message_type);
/// }
/// ```

// 导出主要模块
pub mod message;

pub use message::*;

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 支持的协议版本
pub const PROTOCOL_VERSION: u8 = 1;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_message_creation() {
        // 测试连接消息
        let mut connect_req = ConnectRequest::new();
        connect_req.uid = "test_user".to_string();
        connect_req.token = "test_token".to_string();
        
        let packet = connect_req.create_packet();
        assert_eq!(packet.message_type, MessageType::ConnectRequest);
    }
    
    #[test]
    fn test_message_types() {
        // 确保所有消息类型都有正确的值
        assert_eq!(MessageType::ConnectRequest as u8, 1);
        assert_eq!(MessageType::ConnectResponse as u8, 2);
        assert_eq!(MessageType::DisconnectRequest as u8, 3);
        assert_eq!(MessageType::DisconnectResponse as u8, 4);
        assert_eq!(MessageType::SendRequest as u8, 5);
        assert_eq!(MessageType::SendResponse as u8, 6);
        assert_eq!(MessageType::RecvRequest as u8, 7);
        assert_eq!(MessageType::RecvResponse as u8, 8);
        assert_eq!(MessageType::RecvBatchRequest as u8, 9);        // 批量接收消息
        assert_eq!(MessageType::RecvBatchResponse as u8, 10);     // 批量接收确认
        assert_eq!(MessageType::PingRequest as u8, 11);
        assert_eq!(MessageType::PongResponse as u8, 12);
        assert_eq!(MessageType::SubscribeRequest as u8, 13);
        assert_eq!(MessageType::SubscribeResponse as u8, 14);
        assert_eq!(MessageType::PublishRequest as u8, 15);          // 推送消息
        assert_eq!(MessageType::PublishResponse as u8, 16);       // 推送确认
    }
    
    #[test]
    fn test_send_message() {
        let mut send_req = SendRequest::new();
        send_req.from_uid = "sender123".to_string();
        send_req.channel_id = "channel456".to_string();
        send_req.payload = "Hello World".as_bytes().to_vec();
        
        let packet = send_req.create_packet();
        assert_eq!(packet.message_type, MessageType::SendRequest);
    }
    
    #[test]
    fn test_recv_message() {
        let mut recv_req = RecvRequest::new();
        recv_req.from_uid = "sender456".to_string();
        recv_req.channel_id = "channel789".to_string();
        recv_req.payload = "Hello Back".as_bytes().to_vec();
        
        let packet = recv_req.create_packet();
        assert_eq!(packet.message_type, MessageType::RecvRequest);
    }
}

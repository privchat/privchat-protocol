/// PrivChat IM Error Code v3.0
/// 
/// Error code design (priority-based):
/// - 0: Success
/// - 1-999: System errors (protocol, version, runtime stability) ⭐ First priority
/// - 10000-19999: Common errors (authentication, parameters, permissions, rate limiting)
/// - 20000-65535: Business errors (freely defined, allocated as needed)
/// 
/// Design principles: Simple and practical, flexible extension, within 65535 (u16 range)
/// 
/// **Important**: Error codes cannot be changed once published. New error codes must use new values.

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ErrorCode {
    // ==================== Success ====================
    /// Operation successful
    Ok = 0,
    
    // ==================== System Errors (1-999) ⭐ First Priority ====================
    
    // System Runtime (1-99)
    /// System error
    SystemError = 1,
    /// System busy, please retry later
    SystemBusy = 2,
    /// Service unavailable
    ServiceUnavailable = 3,
    /// Internal error
    InternalError = 4,
    /// Operation timeout
    Timeout = 5,
    /// System maintenance
    Maintenance = 6,
    /// Database error
    DatabaseError = 7,
    /// Cache error
    CacheError = 8,
    /// Network error
    NetworkError = 9,
    
    // Protocol Related (100-199)
    /// Protocol error
    ProtocolError = 100,
    /// Unsupported protocol version
    UnsupportedProtocol = 101,
    /// Invalid packet
    InvalidPacket = 102,
    /// Packet too large
    PacketTooLarge = 103,
    /// Encoding error
    EncodingError = 104,
    /// Decoding error
    DecodingError = 105,
    
    // Version Compatibility (200-299)
    /// Version error
    VersionError = 200,
    /// Client version too old
    ClientVersionTooOld = 201,
    /// Client version too new
    ClientVersionTooNew = 202,
    /// Incompatible version
    IncompatibleVersion = 203,
    /// API deprecated
    DeprecatedApi = 204,
    
    // ==================== Common Errors (10000-19999) ====================
    // Every 100 codes per segment
    
    // Authentication/Authorization (10000-10099)
    /// Authentication required
    AuthRequired = 10000,
    /// Invalid token
    InvalidToken = 10001,
    /// Token expired
    TokenExpired = 10002,
    /// Token revoked
    TokenRevoked = 10003,
    /// Permission denied
    PermissionDenied = 10004,
    /// Session expired
    SessionExpired = 10005,
    /// Session not found
    SessionNotFound = 10006,
    /// User banned
    UserBanned = 10007,
    /// IP address not allowed
    IpNotAllowed = 10008,
    
    // Request Parameters (10100-10199)
    /// Invalid parameters
    InvalidParams = 10100,
    /// Missing required parameter
    MissingRequiredParam = 10101,
    /// Invalid parameter type
    InvalidParamType = 10102,
    /// Parameter out of range
    ParamOutOfRange = 10103,
    /// Invalid format
    InvalidFormat = 10104,
    /// Invalid JSON format
    InvalidJson = 10105,
    /// Payload too large
    PayloadTooLarge = 10106,
    
    // Business Rules (10200-10299)
    /// Operation not allowed
    OperationNotAllowed = 10200,
    /// Resource not found
    ResourceNotFound = 10201,
    /// Resource already exists
    ResourceAlreadyExists = 10202,
    /// Resource deleted
    ResourceDeleted = 10203,
    /// Duplicate operation
    DuplicateOperation = 10204,
    /// Operation conflict
    OperationConflict = 10205,
    
    // Rate Limiting (10300-10399)
    /// Rate limit exceeded
    RateLimitExceeded = 10300,
    /// Daily quota exceeded
    DailyQuotaExceeded = 10301,
    /// Monthly quota exceeded
    MonthlyQuotaExceeded = 10302,
    /// Concurrent limit exceeded
    ConcurrentLimitExceeded = 10303,
    
    // ==================== Business Errors (20000-65535) ====================
    // Every 100 codes per segment, starting from 20000
    
    // Message Basics (20000-20099)
    /// Message not found
    MessageNotFound = 20000,
    /// Message deleted
    MessageDeleted = 20001,
    /// Message revoked
    MessageRevoked = 20002,
    /// Message send failed
    MessageSendFailed = 20003,
    /// Message too large
    MessageTooLarge = 20004,
    /// Invalid message type
    MessageTypeInvalid = 20005,
    /// Invalid message content
    MessageContentInvalid = 20006,
    /// Message cannot be revoked (timeout)
    MessageCannotRevoke = 20007,
    /// Message already read
    MessageAlreadyRead = 20008,
    /// Send message too fast
    SendMessageTooFast = 20009,
    
    // Offline Messages (20100-20199)
    /// Offline message queue full
    OfflineMessageFull = 20100,
    /// Offline message expired
    OfflineMessageExpired = 20101,
    
    // User Basics (20200-20299)
    /// User not found
    UserNotFound = 20200,
    /// User already exists
    UserAlreadyExists = 20201,
    /// User deleted
    UserDeleted = 20202,
    /// User banned
    UserBannedAlt = 20203,
    /// User not active
    UserNotActive = 20204,
    /// Invalid nickname
    NicknameInvalid = 20205,
    /// Invalid avatar
    AvatarInvalid = 20206,
    
    // Group Basics (20300-20399)
    /// Group not found
    GroupNotFound = 20300,
    /// Group deleted
    GroupDeleted = 20301,
    /// Group full
    GroupFull = 20302,
    /// Not a group member
    NotGroupMember = 20303,
    /// Not a group admin
    NotGroupAdmin = 20304,
    /// Not a group owner
    NotGroupOwner = 20305,
    /// Group muted
    GroupMuted = 20306,
    /// Member muted
    MemberMuted = 20307,
    /// Member already in group
    MemberAlreadyInGroup = 20308,
    /// Cannot remove owner
    CannotRemoveOwner = 20309,
    /// Join approval required
    JoinApprovalRequired = 20310,
    
    // Friend Basics (20400-20499)
    /// Friend not found
    FriendNotFound = 20400,
    /// Already friends
    AlreadyFriends = 20401,
    /// Blocked by user
    BlockedByUser = 20402,
    /// User in blacklist
    UserInBlacklist = 20403,
    
    // Channel Basics (20500-20599)
    /// Channel not found
    ChannelNotFound = 20500,
    /// Channel deleted
    ChannelDeleted = 20501,
    /// Channel muted
    ChannelMuted = 20502,
    
    // File Basics (20600-20699)
    /// File not found
    FileNotFound = 20600,
    /// File upload failed
    FileUploadFailed = 20601,
    /// File too large
    FileTooLarge = 20602,
    /// File type not allowed
    FileTypeNotAllowed = 20603,
    /// Upload token invalid
    UploadTokenInvalid = 20604,
    /// Upload token expired
    UploadTokenExpired = 20605,
    /// Storage quota exceeded
    StorageQuotaExceeded = 20606,
    
    // QR Code (20700-20799)
    /// QR code not found
    QRCodeNotFound = 20700,
    /// QR code expired
    QRCodeExpired = 20701,
    /// QR code used
    QRCodeUsed = 20702,
    /// QR code revoked
    QRCodeRevoked = 20703,
    /// QR code limit exceeded
    QRCodeLimitExceeded = 20704,
    
    // Device (20800-20899)
    /// Device not found
    DeviceNotFound = 20800,
    /// Device limit exceeded
    DeviceLimitExceeded = 20801,
    /// Device not verified
    DeviceNotVerified = 20802,
}

impl ErrorCode {
    /// Get the error code value
    pub fn code(&self) -> u32 {
        *self as u32
    }
    
    /// Get the error code message
    pub fn message(&self) -> &'static str {
        match self {
            // Success
            Self::Ok => "Operation successful",
            
            // System Errors (1-999)
            Self::SystemError => "System error",
            Self::SystemBusy => "System busy, please retry later",
            Self::ServiceUnavailable => "Service unavailable",
            Self::InternalError => "Internal error",
            Self::Timeout => "Operation timeout",
            Self::Maintenance => "System maintenance",
            Self::DatabaseError => "Database error",
            Self::CacheError => "Cache error",
            Self::NetworkError => "Network error",
            Self::ProtocolError => "Protocol error",
            Self::UnsupportedProtocol => "Unsupported protocol version",
            Self::InvalidPacket => "Invalid packet",
            Self::PacketTooLarge => "Packet too large",
            Self::EncodingError => "Encoding error",
            Self::DecodingError => "Decoding error",
            Self::VersionError => "Version error",
            Self::ClientVersionTooOld => "Client version too old",
            Self::ClientVersionTooNew => "Client version too new",
            Self::IncompatibleVersion => "Incompatible version",
            Self::DeprecatedApi => "API deprecated",
            
            // Common Errors (10000-19999)
            Self::AuthRequired => "Authentication required",
            Self::InvalidToken => "Invalid token",
            Self::TokenExpired => "Token expired",
            Self::TokenRevoked => "Token revoked",
            Self::PermissionDenied => "Permission denied",
            Self::SessionExpired => "Session expired",
            Self::SessionNotFound => "Session not found",
            Self::UserBanned => "User banned",
            Self::IpNotAllowed => "IP address not allowed",
            Self::InvalidParams => "Invalid parameters",
            Self::MissingRequiredParam => "Missing required parameter",
            Self::InvalidParamType => "Invalid parameter type",
            Self::ParamOutOfRange => "Parameter out of range",
            Self::InvalidFormat => "Invalid format",
            Self::InvalidJson => "Invalid JSON format",
            Self::PayloadTooLarge => "Payload too large",
            Self::OperationNotAllowed => "Operation not allowed",
            Self::ResourceNotFound => "Resource not found",
            Self::ResourceAlreadyExists => "Resource already exists",
            Self::ResourceDeleted => "Resource deleted",
            Self::DuplicateOperation => "Duplicate operation",
            Self::OperationConflict => "Operation conflict",
            Self::RateLimitExceeded => "Rate limit exceeded",
            Self::DailyQuotaExceeded => "Daily quota exceeded",
            Self::MonthlyQuotaExceeded => "Monthly quota exceeded",
            Self::ConcurrentLimitExceeded => "Concurrent limit exceeded",
            
            // Business Errors (20000-65535)
            Self::MessageNotFound => "Message not found",
            Self::MessageDeleted => "Message deleted",
            Self::MessageRevoked => "Message revoked",
            Self::MessageSendFailed => "Message send failed",
            Self::MessageTooLarge => "Message too large",
            Self::MessageTypeInvalid => "Invalid message type",
            Self::MessageContentInvalid => "Invalid message content",
            Self::MessageCannotRevoke => "Message cannot be revoked (timeout)",
            Self::MessageAlreadyRead => "Message already read",
            Self::SendMessageTooFast => "Send message too fast",
            Self::OfflineMessageFull => "Offline message queue full",
            Self::OfflineMessageExpired => "Offline message expired",
            Self::UserNotFound => "User not found",
            Self::UserAlreadyExists => "User already exists",
            Self::UserDeleted => "User deleted",
            Self::UserBannedAlt => "User banned",
            Self::UserNotActive => "User not active",
            Self::NicknameInvalid => "Invalid nickname",
            Self::AvatarInvalid => "Invalid avatar",
            Self::GroupNotFound => "Group not found",
            Self::GroupDeleted => "Group deleted",
            Self::GroupFull => "Group full",
            Self::NotGroupMember => "Not a group member",
            Self::NotGroupAdmin => "Not a group admin",
            Self::NotGroupOwner => "Not a group owner",
            Self::GroupMuted => "Group muted",
            Self::MemberMuted => "Member muted",
            Self::MemberAlreadyInGroup => "Member already in group",
            Self::CannotRemoveOwner => "Cannot remove owner",
            Self::JoinApprovalRequired => "Join approval required",
            Self::FriendNotFound => "Friend not found",
            Self::AlreadyFriends => "Already friends",
            Self::BlockedByUser => "Blocked by user",
            Self::UserInBlacklist => "User in blacklist",
            Self::ChannelNotFound => "Channel not found",
            Self::ChannelDeleted => "Channel deleted",
            Self::ChannelMuted => "Channel muted",
            Self::FileNotFound => "File not found",
            Self::FileUploadFailed => "File upload failed",
            Self::FileTooLarge => "File too large",
            Self::FileTypeNotAllowed => "File type not allowed",
            Self::UploadTokenInvalid => "Upload token invalid",
            Self::UploadTokenExpired => "Upload token expired",
            Self::StorageQuotaExceeded => "Storage quota exceeded",
            Self::QRCodeNotFound => "QR code not found",
            Self::QRCodeExpired => "QR code expired",
            Self::QRCodeUsed => "QR code used",
            Self::QRCodeRevoked => "QR code revoked",
            Self::QRCodeLimitExceeded => "QR code limit exceeded",
            Self::DeviceNotFound => "Device not found",
            Self::DeviceLimitExceeded => "Device limit exceeded",
            Self::DeviceNotVerified => "Device not verified",
        }
    }
    
    /// Convert from u32 to ErrorCode
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            // Success
            0 => Some(Self::Ok),
            
            // System errors (1-999)
            1 => Some(Self::SystemError),
            2 => Some(Self::SystemBusy),
            3 => Some(Self::ServiceUnavailable),
            4 => Some(Self::InternalError),
            5 => Some(Self::Timeout),
            6 => Some(Self::Maintenance),
            7 => Some(Self::DatabaseError),
            8 => Some(Self::CacheError),
            9 => Some(Self::NetworkError),
            100 => Some(Self::ProtocolError),
            101 => Some(Self::UnsupportedProtocol),
            102 => Some(Self::InvalidPacket),
            103 => Some(Self::PacketTooLarge),
            104 => Some(Self::EncodingError),
            105 => Some(Self::DecodingError),
            200 => Some(Self::VersionError),
            201 => Some(Self::ClientVersionTooOld),
            202 => Some(Self::ClientVersionTooNew),
            203 => Some(Self::IncompatibleVersion),
            204 => Some(Self::DeprecatedApi),
            
            // Common errors (10000-19999)
            10000 => Some(Self::AuthRequired),
            10001 => Some(Self::InvalidToken),
            10002 => Some(Self::TokenExpired),
            10003 => Some(Self::TokenRevoked),
            10004 => Some(Self::PermissionDenied),
            10005 => Some(Self::SessionExpired),
            10006 => Some(Self::SessionNotFound),
            10007 => Some(Self::UserBanned),
            10008 => Some(Self::IpNotAllowed),
            10100 => Some(Self::InvalidParams),
            10101 => Some(Self::MissingRequiredParam),
            10102 => Some(Self::InvalidParamType),
            10103 => Some(Self::ParamOutOfRange),
            10104 => Some(Self::InvalidFormat),
            10105 => Some(Self::InvalidJson),
            10106 => Some(Self::PayloadTooLarge),
            10200 => Some(Self::OperationNotAllowed),
            10201 => Some(Self::ResourceNotFound),
            10202 => Some(Self::ResourceAlreadyExists),
            10203 => Some(Self::ResourceDeleted),
            10204 => Some(Self::DuplicateOperation),
            10205 => Some(Self::OperationConflict),
            10300 => Some(Self::RateLimitExceeded),
            10301 => Some(Self::DailyQuotaExceeded),
            10302 => Some(Self::MonthlyQuotaExceeded),
            10303 => Some(Self::ConcurrentLimitExceeded),
            
            // Business errors (20000-65535)
            20000 => Some(Self::MessageNotFound),
            20001 => Some(Self::MessageDeleted),
            20002 => Some(Self::MessageRevoked),
            20003 => Some(Self::MessageSendFailed),
            20004 => Some(Self::MessageTooLarge),
            20005 => Some(Self::MessageTypeInvalid),
            20006 => Some(Self::MessageContentInvalid),
            20007 => Some(Self::MessageCannotRevoke),
            20008 => Some(Self::MessageAlreadyRead),
            20009 => Some(Self::SendMessageTooFast),
            20100 => Some(Self::OfflineMessageFull),
            20101 => Some(Self::OfflineMessageExpired),
            20200 => Some(Self::UserNotFound),
            20201 => Some(Self::UserAlreadyExists),
            20202 => Some(Self::UserDeleted),
            20203 => Some(Self::UserBannedAlt),
            20204 => Some(Self::UserNotActive),
            20205 => Some(Self::NicknameInvalid),
            20206 => Some(Self::AvatarInvalid),
            20300 => Some(Self::GroupNotFound),
            20301 => Some(Self::GroupDeleted),
            20302 => Some(Self::GroupFull),
            20303 => Some(Self::NotGroupMember),
            20304 => Some(Self::NotGroupAdmin),
            20305 => Some(Self::NotGroupOwner),
            20306 => Some(Self::GroupMuted),
            20307 => Some(Self::MemberMuted),
            20308 => Some(Self::MemberAlreadyInGroup),
            20309 => Some(Self::CannotRemoveOwner),
            20310 => Some(Self::JoinApprovalRequired),
            20400 => Some(Self::FriendNotFound),
            20401 => Some(Self::AlreadyFriends),
            20402 => Some(Self::BlockedByUser),
            20403 => Some(Self::UserInBlacklist),
            20500 => Some(Self::ChannelNotFound),
            20501 => Some(Self::ChannelDeleted),
            20502 => Some(Self::ChannelMuted),
            20600 => Some(Self::FileNotFound),
            20601 => Some(Self::FileUploadFailed),
            20602 => Some(Self::FileTooLarge),
            20603 => Some(Self::FileTypeNotAllowed),
            20604 => Some(Self::UploadTokenInvalid),
            20605 => Some(Self::UploadTokenExpired),
            20606 => Some(Self::StorageQuotaExceeded),
            20700 => Some(Self::QRCodeNotFound),
            20701 => Some(Self::QRCodeExpired),
            20702 => Some(Self::QRCodeUsed),
            20703 => Some(Self::QRCodeRevoked),
            20704 => Some(Self::QRCodeLimitExceeded),
            20800 => Some(Self::DeviceNotFound),
            20801 => Some(Self::DeviceLimitExceeded),
            20802 => Some(Self::DeviceNotVerified),
            
            _ => None,
        }
    }
    
    /// Check if this is a system error (first priority)
    pub fn is_system_error(&self) -> bool {
        let code = self.code();
        code >= 1 && code < 1000
    }
    
    /// Check if this is a common error
    pub fn is_common_error(&self) -> bool {
        let code = self.code();
        code >= 10000 && code < 20000
    }
    
    /// Check if this is a business error
    pub fn is_business_error(&self) -> bool {
        let code = self.code();
        code >= 20000
    }
}

impl Default for ErrorCode {
    fn default() -> Self {
        Self::Ok
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code(), self.message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_values() {
        assert_eq!(ErrorCode::Ok.code(), 0);
        
        // System errors (1-999)
        assert_eq!(ErrorCode::SystemBusy.code(), 2);
        assert_eq!(ErrorCode::ProtocolError.code(), 100);
        assert_eq!(ErrorCode::ClientVersionTooOld.code(), 201);
        
        // Common errors (10000-19999)
        assert_eq!(ErrorCode::AuthRequired.code(), 10000);
        assert_eq!(ErrorCode::InvalidParams.code(), 10100);
        assert_eq!(ErrorCode::RateLimitExceeded.code(), 10300);
        
        // Business errors (20000+)
        assert_eq!(ErrorCode::MessageNotFound.code(), 20000);
        assert_eq!(ErrorCode::OfflineMessageFull.code(), 20100);
        assert_eq!(ErrorCode::UserNotFound.code(), 20200);
        assert_eq!(ErrorCode::GroupNotFound.code(), 20300);
        assert_eq!(ErrorCode::FriendNotFound.code(), 20400);
        assert_eq!(ErrorCode::FileNotFound.code(), 20600);
    }

    #[test]
    fn test_error_code_messages() {
        assert_eq!(ErrorCode::Ok.message(), "Operation successful");
        assert_eq!(ErrorCode::SystemBusy.message(), "System busy, please retry later");
        assert_eq!(ErrorCode::AuthRequired.message(), "Authentication required");
        assert_eq!(ErrorCode::MessageNotFound.message(), "Message not found");
    }

    #[test]
    fn test_error_code_classification() {
        // System errors (first priority)
        assert!(ErrorCode::SystemBusy.is_system_error());
        assert!(!ErrorCode::SystemBusy.is_common_error());
        assert!(!ErrorCode::SystemBusy.is_business_error());
        
        // Common errors
        assert!(!ErrorCode::AuthRequired.is_system_error());
        assert!(ErrorCode::AuthRequired.is_common_error());
        assert!(!ErrorCode::AuthRequired.is_business_error());
        
        // Business errors
        assert!(!ErrorCode::MessageNotFound.is_system_error());
        assert!(!ErrorCode::MessageNotFound.is_common_error());
        assert!(ErrorCode::MessageNotFound.is_business_error());
    }

    #[test]
    fn test_error_code_from_code() {
        assert_eq!(ErrorCode::from_code(0), Some(ErrorCode::Ok));
        assert_eq!(ErrorCode::from_code(2), Some(ErrorCode::SystemBusy));
        assert_eq!(ErrorCode::from_code(10000), Some(ErrorCode::AuthRequired));
        assert_eq!(ErrorCode::from_code(10100), Some(ErrorCode::InvalidParams));
        assert_eq!(ErrorCode::from_code(20000), Some(ErrorCode::MessageNotFound));
        assert_eq!(ErrorCode::from_code(99999), None);
    }

    #[test]
    fn test_error_code_display() {
        let err = ErrorCode::AuthRequired;
        assert_eq!(err.to_string(), "[10000] Authentication required");
        
        let err = ErrorCode::SystemBusy;
        assert_eq!(err.to_string(), "[2] System busy, please retry later");
        
        let err = ErrorCode::GroupNotFound;
        assert_eq!(err.to_string(), "[20300] Group not found");
    }
}

/// RPC 请求和响应类型定义
/// 
/// 所有 RPC 接口的请求和响应结构体定义，使用 serde 自动序列化/反序列化
/// 
/// ## 设计原则
/// 1. 所有 ID 字段统一使用 u64 类型
/// 2. 每个 RPC 接口独立文件，便于维护和避免冲突
/// 3. 请求和响应类型命名规范：`{Module}{Action}Request/Response`
/// 4. 使用文档注释标注 RPC 路由路径
/// 
/// ## 目录结构
/// ```
/// rpc/
///   ├── group/          // 群组相关
///   ├── contact/        // 联系人相关
///   ├── message/        // 消息相关
///   ├── channel/   // 会话相关
///   └── ...
/// ```

pub mod group;
pub mod contact;
pub mod message;
pub mod channel;
pub mod account;
pub mod file;
pub mod qrcode;
pub mod auth;
pub mod channel_broadcast;
pub mod presence;
pub mod device;
pub mod sync;
pub mod routes;

// 重导出所有类型，方便使用
pub use group::*;
pub use contact::*;
pub use message::*;
pub use channel::*;
pub use account::*;
pub use file::*;
pub use qrcode::*;
pub use auth::*;
pub use channel_broadcast::*;
pub use presence::*;
pub use device::*;
pub use sync::*;
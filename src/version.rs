//! 协议版本
//!
//! 从 Cargo.toml 的 package.version 获取，供连接认证等使用。

/// 协议库版本（与 Cargo.toml 中 package.version 一致）
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 协议版本号（用于兼容性判断的数值，当前为 1）
pub const PROTOCOL_VERSION: u8 = 1;

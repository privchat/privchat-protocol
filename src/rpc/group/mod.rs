/// 群组相关 RPC 类型定义

pub mod role_set;
pub mod member_mute;
pub mod settings;
pub mod qrcode;
pub mod transfer;
pub mod member;
pub mod approval;
pub mod group;

pub use role_set::*;
pub use member_mute::{GroupMemberMuteResponse, GroupMemberUnmuteResponse};
pub use settings::*;
pub use qrcode::*;
pub use transfer::*;
pub use member::*;
pub use approval::*;
pub use group::*;
//! 频道类型的**线上(wire)编号**——所有端(server / Rust SDK / TS SDK / FFI / 客户端)
//! 对 `channel_type` 字段的唯一真源。
//!
//! 取值 1/2/3,**没有 0**:0 是"未填"。server 数据库内部另有一套存储编号
//! (Direct=0 / Group=1 / Room=2),那是 server 私有实现,**绝不上线**;
//! 历史上正是拿 wire 值当 DB 编号比对,让群聊(2)撞进 Room 分支。
//!
//! `SubscribeRequest` / `PushMessageRequest` 等 wire 结构体的 `channel_type` 字段
//! 仍是 `u8`(FlatBuffers `ubyte`),用 [`ChannelType::from_wire`] 解析、
//! [`ChannelType::as_wire`] 写入;解析失败(0 或 >3)由调用方按协议错误处理。

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChannelType {
    /// 一对一私聊(强语义 IM:pts / 离线 / 已读)。
    Direct = 1,
    /// 群聊(强语义 IM)。
    Group = 2,
    /// Room:有独立生命周期的广播通道,票据授权,无 pts / 离线(ROOM_CHANNEL_SPEC)。
    Room = 3,
}

impl ChannelType {
    pub const ALL: [ChannelType; 3] = [ChannelType::Direct, ChannelType::Group, ChannelType::Room];

    /// 解析 wire 值;0 与未知值返回 `None`。
    pub const fn from_wire(value: u8) -> Option<Self> {
        match value {
            1 => Some(ChannelType::Direct),
            2 => Some(ChannelType::Group),
            3 => Some(ChannelType::Room),
            _ => None,
        }
    }

    pub const fn as_wire(self) -> u8 {
        self as u8
    }

    pub const fn is_room(self) -> bool {
        matches!(self, ChannelType::Room)
    }

    /// 强语义 IM 频道(有 pts / 离线 / 已读):Direct 与 Group。
    pub const fn is_im(self) -> bool {
        !self.is_room()
    }

    pub const fn name(self) -> &'static str {
        match self {
            ChannelType::Direct => "direct",
            ChannelType::Group => "group",
            ChannelType::Room => "room",
        }
    }
}

impl TryFrom<u8> for ChannelType {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, u8> {
        Self::from_wire(value).ok_or(value)
    }
}

impl From<ChannelType> for u8 {
    fn from(value: ChannelType) -> u8 {
        value.as_wire()
    }
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

// JSON / Redis 里保持数字形态,与 wire 一致。
impl Serialize for ChannelType {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(self.as_wire())
    }
}

impl<'de> Deserialize<'de> for ChannelType {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = u8::deserialize(d)?;
        ChannelType::from_wire(v)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid wire channel_type {v} (expected 1=direct, 2=group, 3=room)")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_values_are_one_two_three_and_zero_is_not_a_channel() {
        assert_eq!(ChannelType::Direct.as_wire(), 1);
        assert_eq!(ChannelType::Group.as_wire(), 2);
        assert_eq!(ChannelType::Room.as_wire(), 3);
        assert_eq!(ChannelType::from_wire(0), None);
        assert_eq!(ChannelType::from_wire(4), None);
        for t in ChannelType::ALL {
            assert_eq!(ChannelType::from_wire(t.as_wire()), Some(t));
        }
    }

    #[test]
    fn serde_is_numeric() {
        assert_eq!(serde_json::to_string(&ChannelType::Room).unwrap(), "3");
        assert_eq!(serde_json::from_str::<ChannelType>("2").unwrap(), ChannelType::Group);
        assert!(serde_json::from_str::<ChannelType>("0").is_err());
    }
}

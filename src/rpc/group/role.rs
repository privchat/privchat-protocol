// Copyright 2024 Shanghai Boyu Information Technology Co., Ltd.
// https://privchat.dev
//
// Author: zoujiaqing <zoujiaqing@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! 群成员角色 —— **全系统唯一权威定义**。
//!
//! 在这个文件出现之前，"role" 在四个地方各自为政：server 内存枚举、server DB 列、
//! 客户端本地表、以及各端 UI 的 `isOwner` 判断。协议里只有一个裸 `i32` 和一个裸
//! `String`，谁都不是权威。结果是同一个人从不同路径进来会得到不同角色——
//! 已经因此出过两次事故（三端 `canManage` 恒 false、以及实体同步把群主写成普通成员）。
//!
//! 从这里起：**线上表示以本枚举为准**，任何一端的本地存储若使用别的编码，
//! MUST 在读写边界显式转换，且转换点要能被指着看。

use serde::{Deserialize, Serialize};

/// 群成员角色。
///
/// **数值（wire）编码冻结**：`Member = 0`、`Owner = 1`、`Admin = 2`。
///
/// `0` 必须是**权限最低**的那一个：字段缺失、`unwrap_or_default()`、未初始化的
/// 整数、老客户端不认识的新字段——所有"没拿到"的情形都会落在 0。让 0 代表群主
/// 等于把「读取失败」变成「提权」。
///
/// ⚠️ 这**不是** server `privchat_group_members.role` 列的编码（那里 owner=0），
/// 也不是客户端本地表的历史编码（那里 owner=2）。两边都 MUST 在读写边界转换成
/// 本枚举，转换点要能被指着看。
///
/// **字符串编码冻结**：小写 `"owner"` / `"admin"` / `"member"`。
/// `group/member/list` 与 `group/info.my_role` 用它。
/// 大写（Rust `Debug` 形态）曾让三端权限判定全部失效，**禁止**再出现在线上。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum GroupMemberRole {
    /// 0 —— 缺省值就该是权限最低的。
    Member = 0,
    Owner = 1,
    Admin = 2,
}

impl GroupMemberRole {
    /// 从线上数值解析。
    ///
    /// **未知值一律按 `Member`**：往上猜是提权。一个解析不出来的角色让人当成群主，
    /// 比让群主暂时看不到管理入口危险得多。
    pub fn from_wire_i32(value: i32) -> Self {
        match value {
            1 => Self::Owner,
            2 => Self::Admin,
            _ => Self::Member,
        }
    }

    /// 线上数值。
    pub fn to_wire_i32(self) -> i32 {
        self as i32
    }

    /// 线上字符串（恒小写）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }

    /// 从线上字符串解析。大小写不敏感——老服务端发过 `"Owner"`，
    /// 兼容它读，但**永远不要那样写**。未知一律 `Member`，理由同 [`from_wire_i32`]。
    pub fn from_wire_str(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "owner" => Self::Owner,
            "admin" => Self::Admin,
            _ => Self::Member,
        }
    }

    /// 是否具备群管理权限（群主或管理员）。
    ///
    /// 各端不要再各写一遍 `role == owner || role == admin`——那种表达式抄错一次，
    /// 就是一个群的管理功能整体消失。
    pub fn can_manage(self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }
}

impl Default for GroupMemberRole {
    /// 缺省是普通成员，绝不是群主。
    fn default() -> Self {
        Self::Member
    }
}

#[cfg(test)]
mod tests {
    use super::GroupMemberRole;

    #[test]
    fn wire_numbers_are_frozen() {
        assert_eq!(GroupMemberRole::Member.to_wire_i32(), 0);
        assert_eq!(GroupMemberRole::Owner.to_wire_i32(), 1);
        assert_eq!(GroupMemberRole::Admin.to_wire_i32(), 2);
    }

    /// 0 必须是权限最低的：所有"没拿到"的情形都落在 0，
    /// 让 0 代表群主等于把读取失败变成提权。
    #[test]
    fn zero_is_the_least_privileged_role() {
        assert_eq!(GroupMemberRole::from_wire_i32(0), GroupMemberRole::Member);
        assert!(!GroupMemberRole::from_wire_i32(0).can_manage());
    }

    #[test]
    fn wire_strings_are_lowercase() {
        // 大写形态曾让三端 canManage 恒 false，群主在所有端都看不到管理入口。
        assert_eq!(GroupMemberRole::Owner.as_str(), "owner");
        assert_eq!(GroupMemberRole::Admin.as_str(), "admin");
        assert_eq!(GroupMemberRole::Member.as_str(), "member");
    }

    #[test]
    fn unknown_values_never_resolve_to_owner() {
        // 往上猜是提权：解析不出来的角色绝不能变成群主。
        assert_eq!(GroupMemberRole::from_wire_i32(99), GroupMemberRole::Member);
        assert_eq!(GroupMemberRole::from_wire_i32(-1), GroupMemberRole::Member);
        assert_eq!(GroupMemberRole::from_wire_str(""), GroupMemberRole::Member);
        assert_eq!(
            GroupMemberRole::from_wire_str("god"),
            GroupMemberRole::Member
        );
        assert_eq!(GroupMemberRole::default(), GroupMemberRole::Member);
    }

    #[test]
    fn legacy_capitalised_strings_still_read() {
        // 老服务端发过 Debug 形态；能读，但不许再写。
        assert_eq!(
            GroupMemberRole::from_wire_str("Owner"),
            GroupMemberRole::Owner
        );
        assert_eq!(
            GroupMemberRole::from_wire_str(" ADMIN "),
            GroupMemberRole::Admin
        );
    }

    #[test]
    fn round_trips_both_ways() {
        for role in [
            GroupMemberRole::Owner,
            GroupMemberRole::Admin,
            GroupMemberRole::Member,
        ] {
            assert_eq!(GroupMemberRole::from_wire_i32(role.to_wire_i32()), role);
            assert_eq!(GroupMemberRole::from_wire_str(role.as_str()), role);
        }
    }

    #[test]
    fn only_owner_and_admin_manage() {
        assert!(GroupMemberRole::Owner.can_manage());
        assert!(GroupMemberRole::Admin.can_manage());
        assert!(!GroupMemberRole::Member.can_manage());
    }
}

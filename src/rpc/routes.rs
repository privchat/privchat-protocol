// Copyright 2025 Shanghai Boyu Information Technology Co., Ltd.
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

/// RPC 路由常量定义
///
/// 所有 RPC 接口的路由路径集中定义，避免硬编码字符串
///
/// ## 使用示例
/// ```rust
/// use privchat_protocol::rpc::routes;
///
/// // 使用路由常量
/// client.call_rpc(routes::FRIEND_APPLY, request).await?;
/// ```

/// 好友系统路由
pub mod friend {
    /// 申请添加好友
    pub const APPLY: &str = "contact/friend/apply";

    /// 接受好友申请
    pub const ACCEPT: &str = "contact/friend/accept";

    /// 拒绝好友申请
    pub const REJECT: &str = "contact/friend/reject";

    /// 删除好友
    pub const DELETE: &str = "contact/friend/remove";

    /// 待处理好友请求列表
    pub const PENDING: &str = "contact/friend/pending";

    /// 检查好友关系
    pub const CHECK: &str = "contact/friend/check";
}

/// 黑名单路由
pub mod blacklist {
    /// 添加黑名单
    pub const ADD: &str = "contact/blacklist/add";

    /// 移除黑名单
    pub const REMOVE: &str = "contact/blacklist/remove";

    /// 获取黑名单列表
    pub const LIST: &str = "contact/blacklist/list";

    /// 检查黑名单状态
    pub const CHECK: &str = "contact/blacklist/check";
}

/// 群组路由
pub mod group {
    /// 创建群组
    pub const CREATE: &str = "group/group/create";

    /// 获取群组信息
    pub const INFO: &str = "group/group/info";
}

/// 群组成员路由
pub mod group_member {
    /// 添加成员
    pub const ADD: &str = "group/member/add";

    /// 移除成员
    pub const REMOVE: &str = "group/member/remove";

    /// 获取成员列表
    pub const LIST: &str = "group/member/list";

    /// 退出群组
    pub const LEAVE: &str = "group/member/leave";

    /// 禁言成员
    pub const MUTE: &str = "group/member/mute";

    /// 取消禁言
    pub const UNMUTE: &str = "group/member/unmute";
}

/// 群组角色路由
pub mod group_role {
    /// 转移群主
    pub const TRANSFER_OWNER: &str = "group/role/transfer_owner";

    /// 设置角色
    pub const SET: &str = "group/role/set";
}

/// 群组设置路由
pub mod group_settings {
    /// 获取群组设置
    pub const GET: &str = "group/settings/get";

    /// 更新群组设置
    pub const UPDATE: &str = "group/settings/update";

    /// 全员禁言
    pub const MUTE_ALL: &str = "group/settings/mute_all";
}

/// 群组审批路由
pub mod group_approval {
    /// 获取审批列表
    pub const LIST: &str = "group/approval/list";

    /// 处理审批
    pub const HANDLE: &str = "group/approval/handle";
}

/// 群组二维码路由
pub mod group_qrcode {
    /// 生成群组二维码
    pub const GENERATE: &str = "group/qrcode/generate";

    /// 通过二维码加入群组
    pub const JOIN: &str = "group/join/qrcode";
}

/// 消息路由
pub mod message {
    /// 撤回消息
    pub const REVOKE: &str = "message/revoke";
}

/// 消息历史路由
pub mod message_history {
    /// 获取历史消息
    pub const GET: &str = "message/history/get";
}

/// 消息状态路由
pub mod message_status {
    /// 标记已读
    pub const READ: &str = "message/status/read";

    /// 获取未读数量
    pub const COUNT: &str = "message/status/count";

    /// 获取已读列表
    pub const READ_LIST: &str = "message/status/read_list";

    /// 获取已读统计
    pub const READ_STATS: &str = "message/status/read_stats";
}

/// 消息反应路由
pub mod message_reaction {
    /// 添加反应
    pub const ADD: &str = "message/reaction/add";

    /// 移除反应
    pub const REMOVE: &str = "message/reaction/remove";

    /// 获取反应列表
    pub const LIST: &str = "message/reaction/list";

    /// 获取反应统计
    pub const STATS: &str = "message/reaction/stats";
}

/// 频道路由（私聊、群聊等会话功能）
pub mod channel {
    /// 获取或创建私聊会话（有则返回已有 channel_id）
    pub const DIRECT_GET_OR_CREATE: &str = "channel/direct/get_or_create";

    /// 置顶频道
    pub const PIN: &str = "channel/pin";

    /// 隐藏频道
    pub const HIDE: &str = "channel/hide";

    /// 设置频道静音
    pub const MUTE: &str = "channel/mute";
}

/// 账号搜索路由
pub mod account_search {
    /// 搜索用户
    pub const QUERY: &str = "account/search/query";

    /// 通过二维码搜索
    pub const BY_QRCODE: &str = "account/search/by_qrcode";
}

/// 账号用户路由
pub mod account_user {
    /// 获取用户详情
    pub const DETAIL: &str = "account/user/detail";

    /// 分享用户名片
    pub const SHARE_CARD: &str = "account/user/share_card";

    /// 用户注册
    pub const REGISTER: &str = "account/user/register";

    /// 更新用户资料
    pub const UPDATE: &str = "account/user/update";
}

/// 在线状态路由
pub mod presence {
    /// 订阅在线状态（打开私聊会话时）
    pub const SUBSCRIBE: &str = "presence/subscribe";

    /// 取消订阅（关闭私聊会话时）
    pub const UNSUBSCRIBE: &str = "presence/unsubscribe";

    /// 发送输入状态通知
    pub const TYPING: &str = "presence/typing";

    /// 批量查询在线状态
    pub const STATUS_GET: &str = "presence/status/get";
}

/// 设备管理路由
pub mod device {
    /// 更新设备推送状态
    pub const PUSH_UPDATE: &str = "device/push/update";

    /// 获取设备推送状态
    pub const PUSH_STATUS: &str = "device/push/status";
}

/// 账号资料路由
pub mod account_profile {
    /// 获取个人资料
    pub const GET: &str = "account/profile/get";

    /// 更新个人资料
    pub const UPDATE: &str = "account/profile/update";
}

/// 认证路由
pub mod auth {
    /// 登录
    pub const LOGIN: &str = "account/auth/login";

    /// 登出
    pub const LOGOUT: &str = "account/auth/logout";

    /// 刷新令牌
    pub const REFRESH: &str = "account/auth/refresh";
}

/// 隐私设置路由
pub mod privacy {
    /// 获取隐私设置
    pub const GET: &str = "account/privacy/get";

    /// 更新隐私设置
    pub const UPDATE: &str = "account/privacy/update";
}

/// 文件路由
pub mod file {
    /// 请求上传令牌
    pub const REQUEST_UPLOAD_TOKEN: &str = "file/request_upload_token";

    /// 上传回调
    pub const UPLOAD_CALLBACK: &str = "file/upload_callback";
}

/// 广播频道路由（订阅号/频道功能）
pub mod channel_broadcast {
    /// 创建广播频道
    pub const CREATE: &str = "channel/broadcast/create";

    /// 订阅广播频道
    pub const SUBSCRIBE: &str = "channel/broadcast/subscribe";

    /// 获取广播频道列表
    pub const LIST: &str = "channel/broadcast/list";
}

/// 频道内容路由
pub mod channel_content {
    /// 发布内容
    pub const PUBLISH: &str = "channel/content/publish";

    /// 获取内容列表
    pub const LIST: &str = "channel/content/list";
}

/// 表情包路由
pub mod sticker {
    /// 获取表情包列表
    pub const PACKAGE_LIST: &str = "sticker/package/list";

    /// 获取表情包详情
    pub const PACKAGE_DETAIL: &str = "sticker/package/detail";
}

/// 二维码路由
pub mod qrcode {
    /// 生成二维码
    pub const GENERATE: &str = "qrcode/generate";

    /// 解析二维码
    pub const RESOLVE: &str = "qrcode/resolve";

    /// 刷新二维码
    pub const REFRESH: &str = "qrcode/refresh";

    /// 撤销二维码
    pub const REVOKE: &str = "qrcode/revoke";

    /// 获取二维码列表
    pub const LIST: &str = "qrcode/list";
}

/// 用户二维码路由
pub mod user_qrcode {
    /// 生成用户二维码
    pub const GENERATE: &str = "user/qrcode/generate";

    /// 刷新用户二维码
    pub const REFRESH: &str = "user/qrcode/refresh";

    /// 获取用户二维码
    pub const GET: &str = "user/qrcode/get";
}

/// 同步机制路由
pub mod sync {
    /// 客户端提交命令
    pub const SUBMIT: &str = "sync/submit";

    /// 获取差异（补齐间隙）
    pub const GET_DIFFERENCE: &str = "sync/get_difference";

    /// 获取频道 pts
    pub const GET_CHANNEL_PTS: &str = "sync/get_channel_pts";

    /// 批量获取频道 pts
    pub const BATCH_GET_CHANNEL_PTS: &str = "sync/batch_get_channel_pts";

    /// 会话准备完成（bootstrap sync 完成后调用，开始补差+实时推送）
    pub const SESSION_READY: &str = "sync/session_ready";
}

/// 实体状态同步（ENTITY_SYNC_V1，与 PTS 消息流正交）
pub mod entity {
    /// 通用实体同步 RPC
    pub const SYNC_ENTITIES: &str = "entity/sync_entities";
}

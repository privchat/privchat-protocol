// 负向 fixture:故意制造一个与 protocol 枚举碰撞的常量。
// check_error_codes.py --self-test 会把本目录纳入扫描,**必须**因此失败;
// 若它通过,说明检查器本身失效(历史上正是这种假阴性放过了两组真碰撞)。
object NegativeFixture {
    const val CODE_DELIBERATE_COLLISION: Int = 20900   // 撞 SyncChannelResyncRequired
}

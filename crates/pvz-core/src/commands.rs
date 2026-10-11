/// 空棋盘模拟阶段支持的输入命令。
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
    /// 增加阳光。
    AddSun { amount: u32 },
    /// 消耗阳光。
    SpendSun { amount: u32 },
}

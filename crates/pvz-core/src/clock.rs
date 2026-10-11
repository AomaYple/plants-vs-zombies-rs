use std::fmt;

/// 模拟 tick 的无墙钟计数类型。
pub type Tick = u64;

/// 尚未完成原版时基测量前使用的开发参数。
///
/// 这个值用于稳定的本地开发反馈，不代表原版规格；原版行为验收前必须以测量结果替换或确认。
pub const DEVELOPMENT_TICKS_PER_SECOND: u32 = 60;

/// 模拟时钟的 tick 频率无效。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InvalidTickRate;

impl fmt::Display for InvalidTickRate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("模拟时钟的每秒 tick 数必须大于零")
    }
}

impl std::error::Error for InvalidTickRate {}

/// 只由显式 tick 驱动的可替换模拟时钟。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SimulationClock {
    current_tick: Tick,
    ticks_per_second: u32,
}

impl SimulationClock {
    /// 使用开发阶段的明确 tick 参数创建时钟。
    pub const fn development() -> Self {
        Self {
            current_tick: 0,
            ticks_per_second: DEVELOPMENT_TICKS_PER_SECOND,
        }
    }

    /// 使用指定频率创建时钟。
    pub const fn new(ticks_per_second: u32) -> Result<Self, InvalidTickRate> {
        if ticks_per_second == 0 {
            Err(InvalidTickRate)
        } else {
            Ok(Self {
                current_tick: 0,
                ticks_per_second,
            })
        }
    }

    /// 返回当前 tick。
    pub const fn current_tick(self) -> Tick {
        self.current_tick
    }

    /// 返回每秒 tick 数。
    pub const fn ticks_per_second(self) -> u32 {
        self.ticks_per_second
    }

    /// 推进一个 tick 并返回推进后的计数。
    pub fn advance(&mut self) -> Tick {
        self.current_tick = self.current_tick.saturating_add(1);
        self.current_tick
    }
}

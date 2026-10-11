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
    ///
    /// # Panics
    ///
    /// 已达到 `u64::MAX` 时明确拒绝推进，时钟保持原状态。
    pub fn advance(&mut self) -> Tick {
        self.current_tick = self
            .current_tick
            .checked_add(1)
            .expect("模拟时钟已达到 tick 上限，无法继续推进");
        self.current_tick
    }
}

#[cfg(test)]
mod tests {
    use super::SimulationClock;
    use crate::{Command, Event, GameState, InitialState};
    use pvz_data::LevelId;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn 时钟上限明确拒绝推进且保持原状态() {
        let mut clock = SimulationClock {
            current_tick: u64::MAX,
            ticks_per_second: 60,
        };
        let before = clock;
        let error =
            catch_unwind(AssertUnwindSafe(|| clock.advance())).expect_err("上限必须拒绝推进");
        let message = error
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied())
            .expect("诊断必须是文本");
        assert!(message.contains("模拟时钟") && message.contains("上限"));
        assert_eq!(clock, before);
    }

    #[test]
    fn 世界最后一个tick成功随后拒绝命令且无部分修改() {
        let clock = SimulationClock {
            current_tick: u64::MAX - 1,
            ticks_per_second: 30,
        };
        let initial = InitialState::new(LevelId::new("test-tick-boundary").unwrap(), 0);
        let mut game = GameState::with_clock(7, initial, clock);
        let events = game.step(&[Command::AddSun { amount: 10 }]);
        assert_eq!(events.tick(), u64::MAX);
        assert_eq!(
            events.as_slice(),
            &[
                Event::TickStarted { tick: u64::MAX },
                Event::SunAdded {
                    amount: 10,
                    total: 10
                },
                Event::TickCompleted { tick: u64::MAX },
            ]
        );
        let before = game.state_digest();
        let error = catch_unwind(AssertUnwindSafe(|| {
            game.step(&[
                Command::SpendSun { amount: 10 },
                Command::AddSun { amount: 5 },
            ])
        }));
        assert!(error.is_err(), "达到上限后不能重复同一 tick");
        assert_eq!(game.state_digest(), before);
        assert_eq!(game.sun(), 10);
        assert_eq!(
            game.clock(),
            SimulationClock {
                current_tick: u64::MAX,
                ticks_per_second: 30
            }
        );
    }
}

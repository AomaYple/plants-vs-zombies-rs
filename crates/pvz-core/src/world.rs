use crate::{Command, DeterministicRng, Event, SimulationClock, StepEvents, Tick};
use pvz_data::{LevelId, RulesVersion};

/// 创建模拟世界时使用的初始纯数据。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InitialState {
    level: LevelId,
    sun: u32,
}

impl InitialState {
    /// 创建指定关卡和阳光数量的初始状态。
    pub fn new(level: LevelId, sun: u32) -> Self {
        Self { level, sun }
    }

    /// 返回关卡标识符。
    pub fn level(&self) -> &LevelId {
        &self.level
    }

    /// 返回初始阳光数量。
    pub const fn sun(&self) -> u32 {
        self.sun
    }
}

/// 用于确定性回放比较的完整状态及规则合同摘要。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StateDigest {
    level: LevelId,
    clock: SimulationClock,
    sun: u32,
    rng_state: (u128, u128),
    rules_version: RulesVersion,
}

impl StateDigest {
    /// 返回关卡标识符。
    pub fn level(&self) -> &LevelId {
        &self.level
    }

    /// 返回完整时钟合同，包含当前 tick 和频率。
    pub const fn clock(&self) -> SimulationClock {
        self.clock
    }

    /// 返回当前开发规则版本。
    pub const fn rules_version(&self) -> RulesVersion {
        self.rules_version
    }

    /// 返回 tick。
    pub const fn tick(&self) -> Tick {
        self.clock.current_tick()
    }

    /// 返回阳光数量。
    pub const fn sun(&self) -> u32 {
        self.sun
    }

    /// 返回 RNG 内部状态。
    pub const fn rng_state(&self) -> (u128, u128) {
        self.rng_state
    }
}

/// 不依赖引擎和平台的模拟世界。
///
/// 命令按提交顺序执行；同一个 tick 中先消费再增加不会用后续增加弥补余额。
///
/// ```
/// use pvz_core::{Command, Event, GameState, InitialState};
/// use pvz_data::LevelId;
///
/// let initial = InitialState::new(LevelId::new("example-order")?, 0);
/// let mut game = GameState::new(7, initial);
/// let events = game.step(&[
///     Command::SpendSun { amount: 10 },
///     Command::AddSun { amount: 10 },
/// ]);
/// assert_eq!(game.sun(), 10);
/// assert_eq!(events.as_slice()[1], Event::SunSpendRejected { amount: 10, available: 0 });
/// assert_eq!(game.state_digest().rng_state(), (
///     0x5557_6224_b31e_24e5_5fc5_b510_b8ce_3d5d,
///     0x5fc1_c2d3_ff7a_0dc6_b786_0f7a_9a5f_029f,
/// ));
/// # Ok::<(), pvz_data::IdentifierError>(())
/// ```
pub struct GameState {
    clock: SimulationClock,
    level: LevelId,
    sun: u32,
    rng: DeterministicRng,
}

impl GameState {
    /// 使用开发阶段时钟创建世界。
    pub fn new(seed: u64, initial_state: InitialState) -> Self {
        Self::with_clock(seed, initial_state, SimulationClock::development())
    }

    /// 使用调用方提供的时钟创建世界，便于测试和未来的参考时基接入。
    pub fn with_clock(seed: u64, initial_state: InitialState, clock: SimulationClock) -> Self {
        Self {
            clock,
            level: initial_state.level,
            sun: initial_state.sun,
            rng: DeterministicRng::new(seed),
        }
    }

    /// 按提交顺序执行命令并推进一个 tick。
    ///
    /// 当前空棋盘与阳光命令没有随机规则消费者，因此不消耗 RNG。
    ///
    /// # Panics
    ///
    /// 时钟已达到 `u64::MAX` 时拒绝推进；状态及命令均不发生部分修改。
    pub fn step(&mut self, commands: &[Command]) -> StepEvents {
        let tick = self.clock.advance();
        let mut events = vec![Event::TickStarted { tick }];
        for command in commands {
            match *command {
                Command::AddSun { amount } => match self.sun.checked_add(amount) {
                    Some(total) => {
                        self.sun = total;
                        events.push(Event::SunAdded { amount, total });
                    }
                    None => events.push(Event::SunAddRejected {
                        amount,
                        available: self.sun,
                    }),
                },
                Command::SpendSun { amount } if amount <= self.sun => {
                    self.sun -= amount;
                    events.push(Event::SunSpent {
                        amount,
                        total: self.sun,
                    });
                }
                Command::SpendSun { amount } => events.push(Event::SunSpendRejected {
                    amount,
                    available: self.sun,
                }),
            }
        }

        events.push(Event::TickCompleted { tick });
        StepEvents::new(tick, events)
    }

    /// 返回当前关卡标识符。
    pub fn level(&self) -> &LevelId {
        &self.level
    }

    /// 返回当前阳光数量。
    pub const fn sun(&self) -> u32 {
        self.sun
    }

    /// 返回当前 tick。
    pub const fn tick(&self) -> Tick {
        self.clock.current_tick()
    }

    /// 返回当前时钟合同。
    pub const fn clock(&self) -> SimulationClock {
        self.clock
    }

    /// 返回用于确定性回放比较的状态摘要。
    pub fn state_digest(&self) -> StateDigest {
        StateDigest {
            level: self.level.clone(),
            clock: self.clock,
            sun: self.sun,
            rng_state: self.rng.state(),
            rules_version: RulesVersion::current(),
        }
    }
}

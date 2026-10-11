use crate::{Command, DeterministicRng, Event, SimulationClock, StepEvents, Tick};
use pvz_data::LevelId;

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

/// 用于确定性回放比较的最小状态摘要。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StateDigest {
    tick: Tick,
    sun: u32,
    rng_state: u64,
}

impl StateDigest {
    /// 构造一个用于比较的初始摘要；其 RNG 状态为零仅用于测试夹具。
    pub const fn initial(sun: u32) -> Self {
        Self {
            tick: 0,
            sun,
            rng_state: 0,
        }
    }

    /// 返回 tick。
    pub const fn tick(self) -> Tick {
        self.tick
    }

    /// 返回阳光数量。
    pub const fn sun(self) -> u32 {
        self.sun
    }

    /// 返回 RNG 内部状态。
    pub const fn rng_state(self) -> u64 {
        self.rng_state
    }
}

/// 不依赖引擎和平台的模拟世界。
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

    /// 按规范顺序执行命令并推进一个 tick。
    pub fn step(&mut self, commands: &[Command]) -> StepEvents {
        let tick = self.clock.advance();
        let mut events = vec![Event::TickStarted { tick }];
        let _random_value = self.rng.next_u64();

        let mut ordered_commands = commands.to_vec();
        ordered_commands.sort_by_key(Command::sort_key);
        for command in ordered_commands {
            match command {
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
    pub const fn state_digest(&self) -> StateDigest {
        StateDigest {
            tick: self.clock.current_tick(),
            sun: self.sun,
            rng_state: self.rng.state(),
        }
    }
}

use crate::{
    Board, BoardGeometry, COLUMNS, Cell, Command, DeterministicRng, Event, GameStatus,
    InvalidSliceRules, MowerStatus, Pea, Plant, PlantKind, Point, ROWS, Rejection, SimulationClock,
    SliceRules, StepEvents, Sun, Tick, Zombie,
};
use pvz_data::{LevelId, RulesVersion};

impl Board {
    fn sun_spawn(&mut self, position: Point, tick: Tick, events: &mut Vec<Event>) {
        let id = self.id();
        let amount = self.rules.sun_amount;
        self.suns.push(Sun {
            id,
            position,
            amount,
            expires_at: tick.saturating_add(self.rules.sun_lifetime),
        });
        events.push(Event::SunSpawned {
            id,
            position,
            amount,
        });
    }

    fn simulate(&mut self, tick: Tick, rng: &mut DeterministicRng, events: &mut Vec<Event>) {
        self.suns.retain(|sun| {
            if tick >= sun.expires_at {
                events.push(Event::SunExpired { id: sun.id });
                false
            } else {
                true
            }
        });
        while self.next_spawn < self.rules.wave.len() && self.rules.wave[self.next_spawn].at <= tick
        {
            let spawn = self.rules.wave[self.next_spawn].clone();
            self.next_spawn += 1;
            let id = self.id();
            self.zombies.push(Zombie {
                id,
                row: spawn.row,
                x: spawn.x,
                health: self.rules.zombie_health,
                bite_at: tick,
            });
            events.push(Event::ZombieSpawned { id, row: spawn.row });
        }
        if tick >= self.sky_sun_at {
            let cell = Cell {
                row: (rng.next_u64() % u64::from(ROWS)) as u8,
                column: (rng.next_u64() % u64::from(COLUMNS)) as u8,
            };
            self.sun_spawn(cell.center(), tick, events);
            self.sky_sun_at = tick.saturating_add(self.rules.sky_sun_interval);
        }
        for index in 0..self.plants.len() {
            let plant = &self.plants[index];
            if tick < plant.action_at {
                continue;
            }
            let kind = plant.kind;
            let point = plant.cell.center();
            let row = plant.cell.row;
            let plant_id = plant.id;
            match kind {
                PlantKind::Sunflower => self.sun_spawn(point, tick, events),
                PlantKind::Peashooter => {
                    if !self.zombies.iter().any(|z| z.row == row && z.x >= point.x) {
                        continue;
                    }
                    let id = self.id();
                    self.peas.push(Pea {
                        id,
                        row,
                        x: point.x,
                    });
                    events.push(Event::ProjectileFired {
                        id,
                        plant: plant_id,
                        row,
                    });
                }
            }
            self.plants[index].action_at = tick.saturating_add(self.rules.action_interval(kind));
        }
        // 新发射豌豆也在本tick移动；扫掠范围命中最近僵尸，平局按稳定实体ID。
        for pea in &mut self.peas {
            let next = pea.x + self.rules.pea_speed;
            let radius = self.rules.zombie_half_width + self.rules.pea_radius;
            if let Some(zombie) = self
                .zombies
                .iter_mut()
                .filter(|z| {
                    z.health > 0
                        && z.row == pea.row
                        && z.x + radius >= pea.x
                        && z.x - radius <= next
                })
                .min_by_key(|z| (z.x, z.id))
            {
                events.push(Event::ProjectileHit {
                    id: pea.id,
                    zombie: zombie.id,
                });
                zombie.health = zombie.health.saturating_sub(self.rules.pea_damage);
                events.push(Event::Damaged {
                    target: zombie.id,
                    remaining: zombie.health,
                });
                if zombie.health == 0 {
                    events.push(Event::ZombieDied { id: zombie.id });
                }
                pea.x = BoardGeometry::RIGHT + 1;
            } else {
                pea.x = next;
            }
        }
        self.peas.retain(|pea| pea.x <= BoardGeometry::RIGHT);
        self.zombies.retain(|z| z.health > 0);
        let old_positions: Vec<_> = self.zombies.iter().map(|z| (z.id, z.x)).collect();
        for zombie in &mut self.zombies {
            let next = zombie.x - self.rules.zombie_speed;
            let radius = self.rules.plant_half_width + self.rules.zombie_half_width;
            let target = self
                .plants
                .iter_mut()
                .filter(|p| {
                    p.health > 0
                        && p.cell.row == zombie.row
                        && p.cell.center().x + radius >= next
                        && p.cell.center().x - radius <= zombie.x
                })
                .max_by_key(|p| (p.cell.center().x, p.id));
            if let Some(plant) = target {
                zombie.x = next.max(zombie.x.min(plant.cell.center().x + radius));
                if tick >= zombie.bite_at {
                    plant.health = plant.health.saturating_sub(self.rules.bite_damage);
                    zombie.bite_at = tick.saturating_add(self.rules.bite_interval);
                    events.push(Event::Damaged {
                        target: plant.id,
                        remaining: plant.health,
                    });
                    if plant.health == 0 {
                        events.push(Event::PlantRemoved { id: plant.id });
                    }
                }
            } else {
                zombie.x = next;
            }
        }
        self.plants.retain(|plant| plant.health > 0);
        for mower in &mut self.mowers {
            let starting = mower.status == MowerStatus::Ready
                && self
                    .zombies
                    .iter()
                    .any(|z| z.row == mower.row && z.x <= self.rules.mower_trigger_x);
            if starting {
                mower.status = MowerStatus::Moving;
                events.push(Event::MowerStarted { row: mower.row });
            }
            if mower.status != MowerStatus::Moving {
                continue;
            }
            let old_x = mower.x;
            mower.x += self.rules.mower_speed;
            let radius = self.rules.mower_half_width + self.rules.zombie_half_width;
            for zombie in &mut self.zombies {
                if zombie.row != mower.row || zombie.health == 0 {
                    continue;
                }
                let old_z = old_positions
                    .iter()
                    .find(|(id, _)| *id == zombie.id)
                    .expect("本阶段僵尸有旧位置")
                    .1;
                let from = old_z - old_x;
                let to = zombie.x - mower.x;
                if (starting && zombie.x <= self.rules.mower_trigger_x)
                    || (from.max(to) >= -radius && from.min(to) <= radius)
                {
                    zombie.health = 0;
                    events.push(Event::Damaged {
                        target: zombie.id,
                        remaining: 0,
                    });
                    events.push(Event::ZombieDied { id: zombie.id });
                }
            }
            if mower.x > BoardGeometry::RIGHT + self.rules.mower_half_width {
                mower.status = MowerStatus::Spent;
            }
        }
        self.zombies.retain(|z| z.health > 0);
        if self.zombies.iter().any(|z| z.x <= self.rules.defeat_x) {
            self.status = GameStatus::Lost;
            events.push(Event::Defeat);
        } else if self.next_spawn == self.rules.wave.len() && self.zombies.is_empty() {
            self.status = GameStatus::Won;
            events.push(Event::Victory);
        }
    }
}

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
    initial_seed: u64,
    initial_state: InitialState,
    initial_clock: SimulationClock,
    board: Option<Board>,
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GameState {
    clock: SimulationClock,
    level: LevelId,
    sun: u32,
    rng: DeterministicRng,
    initial_seed: u64,
    initial_state: InitialState,
    initial_clock: SimulationClock,
    board: Option<Board>,
}

impl GameState {
    /// 显式创建白天切片，空棋盘构造入口继续保持规则1.0。
    pub fn day(
        seed: u64,
        initial: InitialState,
        rules: SliceRules,
    ) -> Result<Self, InvalidSliceRules> {
        rules.validate()?;
        let mut game = Self::new(seed, initial);
        game.board = Some(Board::new(rules));
        Ok(game)
    }
    pub fn board(&self) -> Option<&Board> {
        self.board.as_ref()
    }

    fn step_slice(&mut self, commands: &[Command]) -> StepEvents {
        let board = self.board.as_ref().expect("切片状态存在");
        let restarting = commands.contains(&Command::Restart);
        let mut final_paused = board.paused;
        for command in commands {
            match command {
                Command::Pause => final_paused = true,
                Command::Resume => final_paused = false,
                _ => {}
            }
        }
        // 先检查将要推进的tick，溢出拒绝整个批次；暂停/重开仍能在上限处理。
        if !restarting && !final_paused && board.status == GameStatus::Running {
            self.tick()
                .checked_add(1)
                .expect("模拟时钟已达到 tick 上限，无法继续推进");
        }
        let mut events = Vec::new();
        for command in commands {
            self.slice_command(command, &mut events);
        }
        let mut board = self.board.take().expect("切片状态存在");
        if !restarting && !board.paused && board.status == GameStatus::Running {
            let tick = self.clock.advance();
            events.insert(0, Event::TickStarted { tick });
            board.simulate(tick, &mut self.rng, &mut events);
            events.push(Event::TickCompleted { tick });
        }
        self.board = Some(board);
        StepEvents::new(self.tick(), events)
    }

    fn slice_command(&mut self, command: &Command, events: &mut Vec<Event>) {
        if *command == Command::Restart {
            let rules = self.board.as_ref().expect("切片存在").rules.clone();
            self.clock = self.initial_clock;
            self.sun = self.initial_state.sun;
            self.rng = DeterministicRng::new(self.initial_seed);
            self.board = Some(Board::new(rules));
            events.push(Event::Restarted);
            return;
        }
        let tick = self.tick();
        let board = self.board.as_mut().expect("切片存在");
        let rejected =
            |events: &mut Vec<Event>, reason| events.push(Event::CommandRejected { reason });
        if board.status != GameStatus::Running {
            rejected(events, Rejection::Ended);
            return;
        }
        match command {
            Command::Pause => {
                board.paused = true;
                events.push(Event::Paused);
                return;
            }
            Command::Resume => {
                board.paused = false;
                events.push(Event::Resumed);
                return;
            }
            _ => {}
        }
        if board.paused {
            rejected(events, Rejection::Paused);
            return;
        }
        match *command {
            Command::SelectCard { kind } => {
                board.selected = Some(kind);
                events.push(Event::CardSelected { kind });
            }
            Command::Plant { cell } => {
                let reason = if !cell.is_valid() {
                    Some(Rejection::InvalidCell)
                } else if board.plants.iter().any(|plant| plant.cell == cell) {
                    Some(Rejection::Occupied)
                } else if let Some(kind) = board.selected {
                    if board.card_ready_at[kind.index()] > tick {
                        Some(Rejection::CoolingDown)
                    } else if board.rules.cost(kind) > self.sun {
                        Some(Rejection::NotEnoughSun)
                    } else {
                        None
                    }
                } else {
                    Some(Rejection::NoSelection)
                };
                if let Some(reason) = reason {
                    rejected(events, reason);
                    return;
                }
                let kind = board.selected.expect("已经检查选卡");
                let cost = board.rules.cost(kind);
                self.sun -= cost;
                events.push(Event::SunSpent {
                    amount: cost,
                    total: self.sun,
                });
                let id = board.id();
                board.plants.push(Plant {
                    id,
                    kind,
                    cell,
                    health: board.rules.plant_health,
                    action_at: tick.saturating_add(board.rules.action_interval(kind)),
                });
                board.card_ready_at[kind.index()] = tick.saturating_add(board.rules.card_cooldown);
                events.push(Event::Planted { id, kind, cell });
            }
            Command::Shovel { cell } => {
                if !cell.is_valid() {
                    rejected(events, Rejection::InvalidCell);
                } else if let Some(index) = board.plants.iter().position(|plant| plant.cell == cell)
                {
                    events.push(Event::PlantRemoved {
                        id: board.plants.remove(index).id,
                    });
                } else {
                    rejected(events, Rejection::MissingPlant);
                }
            }
            Command::CollectSun { id } => {
                if let Some(index) = board.suns.iter().position(|sun| sun.id == id) {
                    let amount = board.suns[index].amount;
                    if let Some(total) = self.sun.checked_add(amount) {
                        board.suns.remove(index);
                        self.sun = total;
                        events.push(Event::SunCollected { id, amount, total });
                    } else {
                        rejected(events, Rejection::BalanceOverflow);
                    }
                } else {
                    rejected(events, Rejection::MissingSun);
                }
            }
            _ => rejected(events, Rejection::Unsupported),
        }
    }
    /// 使用开发阶段时钟创建世界。
    pub fn new(seed: u64, initial_state: InitialState) -> Self {
        Self::with_clock(seed, initial_state, SimulationClock::development())
    }

    /// 使用调用方提供的时钟创建世界，便于测试和未来的参考时基接入。
    pub fn with_clock(seed: u64, initial_state: InitialState, clock: SimulationClock) -> Self {
        Self {
            clock,
            level: initial_state.level.clone(),
            sun: initial_state.sun,
            rng: DeterministicRng::new(seed),
            initial_seed: seed,
            initial_state,
            initial_clock: clock,
            board: None,
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
        if self.board.is_some() {
            return self.step_slice(commands);
        }
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
                _ => events.push(Event::CommandRejected {
                    reason: Rejection::Unsupported,
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
            rules_version: if self.board.is_some() {
                RulesVersion::current()
            } else {
                RulesVersion::new(1, 0).expect("空棋盘规则版本有效")
            },
            initial_seed: self.initial_seed,
            initial_state: self.initial_state.clone(),
            initial_clock: self.initial_clock,
            board: self.board.clone(),
        }
    }
}

#[cfg(test)]
mod slice_tests {
    use super::*;
    fn game() -> GameState {
        GameState::day(
            7,
            InitialState::new(LevelId::new("digest-isolation").unwrap(), 100),
            SliceRules::development(),
        )
        .unwrap()
    }
    #[test]
    fn 隐藏未来状态逐项单独改变必然改变摘要() {
        let base = game();
        let changes: &[fn(&mut GameState)] = &[
            |s| s.initial_seed += 1,
            |s| s.initial_state.sun += 1,
            |s| {
                s.initial_clock.advance();
            },
            |s| s.board.as_mut().unwrap().next_id += 1,
            |s| s.board.as_mut().unwrap().next_spawn += 1,
            |s| s.board.as_mut().unwrap().sky_sun_at += 1,
            |s| s.board.as_mut().unwrap().card_ready_at[0] += 1,
            |s| s.board.as_mut().unwrap().rules.bite_interval += 1,
            |s| s.board.as_mut().unwrap().selected = Some(PlantKind::Peashooter),
            |s| s.board.as_mut().unwrap().paused = true,
            |s| s.board.as_mut().unwrap().status = GameStatus::Won,
        ];
        for change in changes {
            let mut other = base.clone();
            change(&mut other);
            assert_ne!(base.state_digest(), other.state_digest());
        }
        let mut base = game();
        base.step(&[
            Command::SelectCard {
                kind: PlantKind::Sunflower,
            },
            Command::Plant {
                cell: Cell { row: 0, column: 0 },
            },
        ]);
        let mut other = base.clone();
        other.board.as_mut().unwrap().plants[0].action_at += 1;
        assert_ne!(base.state_digest(), other.state_digest());
    }
    #[test]
    fn 切片时钟上限拒绝整批但允许暂停和重开() {
        let mut state = game();
        state.clock = SimulationClock::at_limit();
        let before = state.state_digest();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.step(&[Command::SelectCard {
                kind: PlantKind::Sunflower,
            }]);
        }));
        assert!(result.is_err());
        assert_eq!(state.state_digest(), before);
        state.step(&[Command::Pause]);
        assert!(state.board().unwrap().paused());
        let before = state.state_digest();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.step(&[Command::Resume]);
        }));
        assert!(result.is_err());
        assert_eq!(state.state_digest(), before);
        state.step(&[Command::Restart]);
        assert_eq!(state.state_digest(), game().state_digest());
    }
}

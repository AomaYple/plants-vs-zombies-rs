use crate::{input::InputEvent, viewport::LogicalPoint};
use pvz_core::{
    BoardGeometry, Command, DEVELOPMENT_TICKS_PER_SECOND, Event, GameState, GameStatus,
    InitialState, PlantKind, Point, SliceRules, UNITS_PER_PIXEL,
};
use pvz_data::LevelId;

/// 事件到音效的表现映射，不包含伤害或余额规则。
pub fn sound_cue(event: &Event) -> Option<crate::resources::SoundCue> {
    use crate::resources::SoundCue;
    match event {
        Event::Planted { .. } => Some(SoundCue::Plant),
        Event::SunCollected { .. } => Some(SoundCue::Collect),
        Event::ProjectileHit { .. } => Some(SoundCue::Hit),
        Event::MowerStarted { .. } => Some(SoundCue::Mower),
        _ => None,
    }
}

/// 平台只排队命令，模拟负责暂停、伤害和胜负。
pub struct Runtime {
    pub game: GameState,
    pub animation_seconds: f64,
    pub render_frames: u64,
    pub paused: bool,
    pub shovel: bool,
    accumulator: f64,
    pending: Vec<Command>,
    events: Vec<Event>,
}
impl Default for Runtime {
    fn default() -> Self {
        Self {
            game: GameState::new(
                7,
                InitialState::new(
                    LevelId::new("development-viewport").expect("固定开发关卡标识有效"),
                    50,
                ),
            ),
            animation_seconds: 0.0,
            render_frames: 0,
            paused: false,
            shovel: false,
            accumulator: 0.0,
            pending: Vec::new(),
            events: Vec::new(),
        }
    }
}
impl Runtime {
    pub fn day() -> Self {
        Self {
            game: GameState::day(
                7,
                InitialState::new(
                    LevelId::new("development-day-slice").expect("开发关卡有效"),
                    150,
                ),
                SliceRules::development(),
            )
            .expect("开发规则有效"),
            ..Self::default()
        }
    }
    pub fn command(&mut self, command: Command) {
        self.pending.push(command);
    }
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
    pub fn input(&mut self, event: InputEvent) {
        if self.game.board().is_none() {
            match event {
                InputEvent::Pause => self.paused = true,
                InputEvent::Resume => self.paused = false,
                InputEvent::PointerDown { .. } if !self.paused => {
                    self.pending.push(Command::AddSun { amount: 1 })
                }
                _ => {}
            }
            return;
        }
        match event {
            InputEvent::Pause => self.command(Command::Pause),
            InputEvent::Resume => self.command(Command::Resume),
            InputEvent::PointerDown { point, .. } => self.pointer(point),
            _ => {}
        }
    }
    fn pointer(&mut self, point: LogicalPoint) {
        if !point.x.is_finite() || !point.y.is_finite() {
            return;
        }
        if (0.0..60.0).contains(&point.y) {
            match point.x {
                x if (130.0..285.0).contains(&x) => {
                    self.shovel = false;
                    self.command(Command::SelectCard {
                        kind: PlantKind::Sunflower,
                    });
                }
                x if (285.0..445.0).contains(&x) => {
                    self.shovel = false;
                    self.command(Command::SelectCard {
                        kind: PlantKind::Peashooter,
                    });
                }
                x if (445.0..550.0).contains(&x) => self.shovel = !self.shovel,
                x if (550.0..675.0).contains(&x) => self.command(
                    if self.pending.iter().fold(
                        self.game.board().is_some_and(|board| board.paused()),
                        |paused, command| match command {
                            Command::Pause => true,
                            Command::Resume | Command::Restart => false,
                            _ => paused,
                        },
                    ) {
                        Command::Resume
                    } else {
                        Command::Pause
                    },
                ),
                x if (675.0..800.0).contains(&x) => self.command(Command::Restart),
                _ => {}
            }
            return;
        }
        let point = Point {
            x: (point.x * UNITS_PER_PIXEL as f32) as i32,
            y: (point.y * UNITS_PER_PIXEL as f32) as i32,
        };
        let board = self.game.board().expect("切片输入拥有棋盘");
        if let Some(id) = board.sun_at(point) {
            self.command(Command::CollectSun { id });
        } else if let Some(cell) = BoardGeometry::cell_at(point) {
            self.command(if self.shovel {
                Command::Shovel { cell }
            } else {
                Command::Plant { cell }
            });
        }
    }
    fn step(&mut self) -> bool {
        let commands = std::mem::take(&mut self.pending);
        let events = self.game.step(&commands);
        let restarted = events.as_slice().contains(&Event::Restarted);
        if restarted {
            self.animation_seconds = 0.0;
            self.accumulator = 0.0;
            self.shovel = false;
        }
        self.events.extend_from_slice(events.as_slice());
        if let Some(board) = self.game.board() {
            self.paused = board.paused();
        }
        restarted
    }
    /// 开发帧耗时上限250ms；暂停与终局不积累追赶时间，控制命令仍可恢复。
    pub fn advance(&mut self, elapsed_seconds: f64) {
        self.render_frames += 1;
        if !elapsed_seconds.is_finite() || elapsed_seconds <= 0.0 {
            return;
        }
        if self.game.board().is_none() && self.paused {
            return;
        }
        let blocked = self
            .game
            .board()
            .is_some_and(|b| b.paused() || b.status() != GameStatus::Running);
        if blocked {
            self.accumulator = 0.0;
            if !self.pending.is_empty() {
                self.step();
            }
            return;
        }
        let elapsed = elapsed_seconds.min(0.25);
        self.accumulator += elapsed;
        let tick = 1.0 / f64::from(DEVELOPMENT_TICKS_PER_SECOND);
        let steps = (self.accumulator / tick).floor() as u32;
        self.accumulator -= f64::from(steps) * tick;
        let mut restarted = false;
        for _ in 0..steps {
            restarted = self.step();
            if restarted {
                break;
            }
            if self
                .game
                .board()
                .is_some_and(|b| b.paused() || b.status() != GameStatus::Running)
            {
                self.accumulator = 0.0;
                break;
            }
        }
        // 动画按帧耗时推进，即使尚不足一个模拟tick也能更新；重开不承接旧帧余量。
        if !restarted
            && !self.paused
            && self
                .game
                .board()
                .is_none_or(|board| board.status() == GameStatus::Running)
        {
            self.animation_seconds += elapsed;
        }
    }
}

use crate::input::InputEvent;
use pvz_core::{Command, DEVELOPMENT_TICKS_PER_SECOND, GameState, InitialState};
use pvz_data::LevelId;

/// 三个时钟分离的开发探针；真实游戏暂停规则仍待核心扩展。
pub struct Runtime {
    pub game: GameState,
    pub animation_seconds: f64,
    pub render_frames: u64,
    pub paused: bool,
    accumulator: f64,
    pending: Vec<Command>,
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
            accumulator: 0.0,
            pending: Vec::new(),
        }
    }
}
impl Runtime {
    pub fn input(&mut self, event: InputEvent) {
        match event {
            InputEvent::Pause => self.paused = true,
            InputEvent::Resume => self.paused = false,
            InputEvent::PointerDown { .. } if !self.paused => {
                self.pending.push(Command::AddSun { amount: 1 })
            }
            _ => {}
        }
    }
    /// 开发帧耗时上限 250ms 防止恢复窗口后无限追赶；不作为规则时基。
    pub fn advance(&mut self, elapsed_seconds: f64) {
        self.render_frames += 1;
        if self.paused || !elapsed_seconds.is_finite() || elapsed_seconds <= 0.0 {
            return;
        }
        let elapsed = elapsed_seconds.min(0.25);
        self.animation_seconds += elapsed;
        self.accumulator += elapsed;
        let tick = 1.0 / f64::from(DEVELOPMENT_TICKS_PER_SECOND);
        let steps = (self.accumulator / tick).floor() as u32;
        self.accumulator -= f64::from(steps) * tick;
        for _ in 0..steps {
            self.game.step(&self.pending);
            self.pending.clear();
        }
    }
}

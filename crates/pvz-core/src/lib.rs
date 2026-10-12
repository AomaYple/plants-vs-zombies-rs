//! 植物大战僵尸重写项目的纯 Rust 确定性模拟基础。
//!
//! 本 crate 不依赖游戏引擎、文件系统、设备输入、音频、操作系统 API 或墙钟时间。

mod board;
mod clock;
mod commands;
mod content;
mod events;
mod plants;
mod projectiles;
mod rng;
mod world;
mod zombies;

pub use board::{
    Board, BoardGeometry, COLUMNS, Cell, EntityId, GameStatus, InvalidSliceRules, LawnMower,
    MowerStatus, Point, ROWS, SliceRules, Spawn, UNITS_PER_PIXEL,
};
pub use clock::{DEVELOPMENT_TICKS_PER_SECOND, InvalidTickRate, SimulationClock, Tick};
pub use commands::Command;
pub use content::InvalidDayLevel;
pub use events::{Event, Rejection, StepEvents};
pub use plants::{Plant, PlantKind};
pub use projectiles::{Pea, Sun};
pub use rng::DeterministicRng;
pub use world::{GameState, InitialState, StateDigest};
pub use zombies::Zombie;

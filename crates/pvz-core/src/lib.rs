//! 植物大战僵尸重写项目的纯 Rust 确定性模拟基础。
//!
//! 本 crate 不依赖游戏引擎、文件系统、设备输入、音频、操作系统 API 或墙钟时间。

mod clock;
mod commands;
mod events;
mod rng;
mod world;

pub use clock::{DEVELOPMENT_TICKS_PER_SECOND, InvalidTickRate, SimulationClock, Tick};
pub use commands::Command;
pub use events::{Event, StepEvents};
pub use rng::DeterministicRng;
pub use world::{GameState, InitialState, StateDigest};

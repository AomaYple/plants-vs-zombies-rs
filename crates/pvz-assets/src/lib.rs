//! 版本化原版资源的清单审计和可复用格式解析。
//!
//! 清单审计负责宿主文件系统输入；Reanim 和粒子解析器只接受文本，
//! 因此核心解析合同可以在桌面、Web 和移动端复用。

mod diagnostics;
mod manifest;
mod particles;
mod reanim;

pub use diagnostics::{AuditReport, Diagnostic, EventKind};
pub use manifest::{AssetError, AssetIndex, AssetRoot};
pub use particles::ParticleSystem;
pub use reanim::{AnimationClip, AnimationTrack};

/// 解析 Reanim 文本的最小支持子集。
pub fn parse_reanim(source: &str) -> Result<AnimationClip, AssetError> {
    reanim::parse(source)
}

/// 解析粒子 XML 文本的最小支持子集。
pub fn parse_particle(source: &str) -> Result<ParticleSystem, AssetError> {
    particles::parse(source)
}

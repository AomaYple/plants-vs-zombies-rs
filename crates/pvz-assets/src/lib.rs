//! 版本化原版资源的清单审计和可复用格式解析。
//!
//! `Manifest::parse` 与记录字节校验不依赖宿主路径；`AssetRoot` 和
//! `AssetIndex` 单独适配只读文件系统。Reanim 与粒子描述树只接受文本，
//! 可在桌面、Web 和移动端资源提供层复用。

mod diagnostics;
mod manifest;
mod particles;
mod reanim;
mod xml;

pub use diagnostics::{AuditReport, Diagnostic};
pub use manifest::{
    AssetError, AssetIndex, AssetKind, AssetRecord, AssetRoot, Manifest, PackageMetadata,
    SourceArchive,
};
pub use particles::{ParticleEmitter, ParticleProperty, ParticleSystem};
pub use reanim::{AnimationClip, AnimationFrame, AnimationTrack};

/// 解析 Reanim 文本，保留原始稀疏帧并拒绝未知扩展。
pub fn parse_reanim(source: &str) -> Result<AnimationClip, AssetError> {
    reanim::parse(source)
}

/// 解析粒子 XML 描述树并保留曲线原文，不解释运行语义。
pub fn parse_particle(source: &str) -> Result<ParticleSystem, AssetError> {
    particles::parse(source)
}

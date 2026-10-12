//! 植物大战僵尸重写项目的版本化领域数据类型。
//!
//! 本 crate 只定义可序列化的纯数据和标识符，不读取文件、不访问资源，
//! 也不依赖游戏引擎或平台 API。

mod day;
mod ids;
mod version;

pub use day::{ContentBasis, DayLevelDefinition, DayRulesDefinition, DaySpawnDefinition};
pub use ids::{AssetId, IdentifierError, LevelId, PlantId, ReferenceBaselineId, ZombieId};
pub use version::{InvalidRulesVersion, RulesVersion, SchemaVersion, UnsupportedSchemaVersion};

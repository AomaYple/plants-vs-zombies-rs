use crate::{LevelId, RulesVersion, SchemaVersion};
use serde::{Deserialize, Serialize};

/// 内容参数的证据依据；开发参数尚不代表原版测量结果。
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentBasis {
    Development,
}

/// 版本化白天关卡的纯数据定义，游戏规则有效性由核心装配判断。
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DayLevelDefinition {
    pub schema_version: SchemaVersion,
    pub rules_version: RulesVersion,
    pub basis: ContentBasis,
    pub level: LevelId,
    pub seed: u64,
    pub initial_sun: u32,
    pub ticks_per_second: u32,
    pub rules: DayRulesDefinition,
}

/// 白天切片的规则参数，字段类型与核心切片规则一一对应。
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DayRulesDefinition {
    pub plant_health: u32,
    pub zombie_health: u32,
    pub pea_damage: u32,
    pub bite_damage: u32,
    pub zombie_speed: i32,
    pub pea_speed: i32,
    pub mower_speed: i32,
    pub plant_half_width: i32,
    pub zombie_half_width: i32,
    pub pea_radius: i32,
    pub mower_half_width: i32,
    pub sun_radius: i32,
    pub mower_trigger_x: i32,
    pub defeat_x: i32,
    pub sunflower_cost: u32,
    pub peashooter_cost: u32,
    pub card_cooldown: u64,
    pub shoot_interval: u64,
    pub bite_interval: u64,
    pub sunflower_interval: u64,
    pub sky_sun_interval: u64,
    pub sun_lifetime: u64,
    pub sun_amount: u32,
    pub wave: Vec<DaySpawnDefinition>,
}

/// 固定波次的一项，数组顺序及相同 tick 的输入顺序均予以保留。
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaySpawnDefinition {
    pub at: u64,
    pub row: u8,
    pub x: i32,
}

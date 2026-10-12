use crate::{GameState, InitialState, InvalidSliceRules, SimulationClock, SliceRules, Spawn};
use pvz_data::{DayLevelDefinition, DayRulesDefinition, RulesVersion};
use std::{error::Error, fmt};

/// 白天关卡定义不满足受支持版本、时钟或规则合同。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InvalidDayLevel {
    /// 装配入口严格只支持规则 2.0。
    UnsupportedRulesVersion(RulesVersion),
    /// schema 1 的频率必须在闭区间 1..=1000 内。
    InvalidTickRate(u32),
    /// 数值和波次未通过既有切片规则校验。
    InvalidRules(InvalidSliceRules),
}

impl fmt::Display for InvalidDayLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRulesVersion(version) => {
                write!(formatter, "不支持的白天关卡规则版本：{version}；仅支持 2.0")
            }
            Self::InvalidTickRate(frequency) => {
                write!(
                    formatter,
                    "白天关卡的每秒 tick 数必须在 1..=1000 内，实际为 {frequency}"
                )
            }
            Self::InvalidRules(error) => write!(formatter, "白天关卡规则无效：{error}"),
        }
    }
}

impl Error for InvalidDayLevel {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRules(error) => Some(error),
            Self::UnsupportedRulesVersion(_) | Self::InvalidTickRate(_) => None,
        }
    }
}

impl GameState {
    /// 从已解析的定义装配白天切片；不读取文件或解析 JSON。
    pub fn from_day_definition(definition: &DayLevelDefinition) -> Result<Self, InvalidDayLevel> {
        if definition.rules_version.major() != 2 || definition.rules_version.minor() != 0 {
            return Err(InvalidDayLevel::UnsupportedRulesVersion(
                definition.rules_version,
            ));
        }
        if !(1..=1000).contains(&definition.ticks_per_second) {
            return Err(InvalidDayLevel::InvalidTickRate(
                definition.ticks_per_second,
            ));
        }
        let clock = SimulationClock::new(definition.ticks_per_second)
            .map_err(|_| InvalidDayLevel::InvalidTickRate(definition.ticks_per_second))?;
        Self::day_with_clock(
            definition.seed,
            InitialState::new(definition.level.clone(), definition.initial_sun),
            slice_rules(&definition.rules),
            clock,
        )
        .map_err(InvalidDayLevel::InvalidRules)
    }
}

// 只作字段转换，规则有效性始终由 SliceRules::validate 判断。
fn slice_rules(definition: &DayRulesDefinition) -> SliceRules {
    SliceRules {
        plant_health: definition.plant_health,
        zombie_health: definition.zombie_health,
        pea_damage: definition.pea_damage,
        bite_damage: definition.bite_damage,
        zombie_speed: definition.zombie_speed,
        pea_speed: definition.pea_speed,
        mower_speed: definition.mower_speed,
        plant_half_width: definition.plant_half_width,
        zombie_half_width: definition.zombie_half_width,
        pea_radius: definition.pea_radius,
        mower_half_width: definition.mower_half_width,
        sun_radius: definition.sun_radius,
        mower_trigger_x: definition.mower_trigger_x,
        defeat_x: definition.defeat_x,
        sunflower_cost: definition.sunflower_cost,
        peashooter_cost: definition.peashooter_cost,
        card_cooldown: definition.card_cooldown,
        shoot_interval: definition.shoot_interval,
        bite_interval: definition.bite_interval,
        sunflower_interval: definition.sunflower_interval,
        sky_sun_interval: definition.sky_sun_interval,
        sun_lifetime: definition.sun_lifetime,
        sun_amount: definition.sun_amount,
        wave: definition
            .wave
            .iter()
            .map(|spawn| Spawn {
                at: spawn.at,
                row: spawn.row,
                x: spawn.x,
            })
            .collect(),
    }
}

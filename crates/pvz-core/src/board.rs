use crate::{Pea, Plant, PlantKind, Sun, Tick, Zombie};
use std::fmt;

/// 跨实体类型共享、单局单调递增的标识。
pub type EntityId = u64;
pub const ROWS: u8 = 5;
pub const COLUMNS: u8 = 9;
pub const UNITS_PER_PIXEL: i32 = 100;
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Cell {
    pub row: u8,
    pub column: u8,
}
impl Cell {
    pub const fn is_valid(self) -> bool {
        self.row < ROWS && self.column < COLUMNS
    }
    pub const fn center(self) -> Point {
        Point {
            x: BoardGeometry::LEFT
                + self.column as i32 * BoardGeometry::CELL_WIDTH
                + BoardGeometry::CELL_WIDTH / 2,
            y: BoardGeometry::TOP
                + self.row as i32 * BoardGeometry::CELL_HEIGHT
                + BoardGeometry::CELL_HEIGHT / 2,
        }
    }
}
/// 规则2.0的开发逻辑几何。视口缩放不会改变它。
pub struct BoardGeometry;
impl BoardGeometry {
    pub const LEFT: i32 = 8_000;
    pub const TOP: i32 = 10_000;
    pub const CELL_WIDTH: i32 = 8_000;
    pub const CELL_HEIGHT: i32 = 9_600;
    pub const RIGHT: i32 = Self::LEFT + COLUMNS as i32 * Self::CELL_WIDTH;
    pub const BOTTOM: i32 = Self::TOP + ROWS as i32 * Self::CELL_HEIGHT;
    pub const fn cell_at(point: Point) -> Option<Cell> {
        if point.x < Self::LEFT
            || point.x >= Self::RIGHT
            || point.y < Self::TOP
            || point.y >= Self::BOTTOM
        {
            return None;
        }
        Some(Cell {
            row: ((point.y - Self::TOP) / Self::CELL_HEIGHT) as u8,
            column: ((point.x - Self::LEFT) / Self::CELL_WIDTH) as u8,
        })
    }
    pub const fn row_y(row: u8) -> i32 {
        Self::TOP + row as i32 * Self::CELL_HEIGHT + Self::CELL_HEIGHT / 2
    }
}
/// 固定波次是纯数据；相同tick按数组顺序生成。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Spawn {
    pub at: Tick,
    pub row: u8,
    pub x: i32,
}
/// 所有数值都是原创开发参数，尚无原版测量证据。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SliceRules {
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
    pub card_cooldown: Tick,
    pub shoot_interval: Tick,
    pub bite_interval: Tick,
    pub sunflower_interval: Tick,
    pub sky_sun_interval: Tick,
    pub sun_lifetime: Tick,
    pub sun_amount: u32,
    pub wave: Vec<Spawn>,
}
impl SliceRules {
    pub fn development() -> Self {
        Self {
            plant_health: 300,
            zombie_health: 200,
            pea_damage: 20,
            bite_damage: 20,
            zombie_speed: 20,
            pea_speed: 600,
            mower_speed: 1_200,
            plant_half_width: 2_400,
            zombie_half_width: 2_000,
            pea_radius: 700,
            mower_half_width: 3_000,
            sun_radius: 3_000,
            mower_trigger_x: 6_000,
            defeat_x: 0,
            sunflower_cost: 50,
            peashooter_cost: 100,
            card_cooldown: 180,
            shoot_interval: 90,
            bite_interval: 30,
            sunflower_interval: 600,
            sky_sun_interval: 300,
            sun_lifetime: 900,
            sun_amount: 25,
            wave: [2, 0, 4, 1, 3, 2, 0, 4, 1, 3]
                .into_iter()
                .enumerate()
                .map(|(index, row)| Spawn {
                    at: 600 + index as u64 * 360,
                    row,
                    x: 79_000,
                })
                .collect(),
        }
    }
    pub const fn cost(&self, kind: PlantKind) -> u32 {
        match kind {
            PlantKind::Sunflower => self.sunflower_cost,
            PlantKind::Peashooter => self.peashooter_cost,
        }
    }
    pub const fn action_interval(&self, kind: PlantKind) -> Tick {
        match kind {
            PlantKind::Sunflower => self.sunflower_interval,
            PlantKind::Peashooter => self.shoot_interval,
        }
    }
    pub fn validate(&self) -> Result<(), InvalidSliceRules> {
        let positive = [
            self.plant_health,
            self.zombie_health,
            self.pea_damage,
            self.bite_damage,
            self.sun_amount,
        ];
        let intervals = [
            self.card_cooldown,
            self.shoot_interval,
            self.bite_interval,
            self.sunflower_interval,
            self.sky_sun_interval,
            self.sun_lifetime,
        ];
        let distances = [
            self.zombie_speed,
            self.pea_speed,
            self.mower_speed,
            self.plant_half_width,
            self.zombie_half_width,
            self.pea_radius,
            self.mower_half_width,
            self.sun_radius,
        ];
        if positive.contains(&0)
            || intervals.contains(&0)
            || distances.iter().any(|&n| !(1..=100_000).contains(&n))
            || self.defeat_x < -100_000
            || self.mower_trigger_x <= self.defeat_x
            || self.mower_trigger_x >= BoardGeometry::LEFT
            || self.wave.is_empty()
            || self.wave.len() > 10_000
            || self.wave.iter().any(|s| {
                s.at == 0
                    || s.row >= ROWS
                    || !(self.mower_trigger_x..=BoardGeometry::RIGHT).contains(&s.x)
            })
            || self.wave.windows(2).any(|w| w[0].at > w[1].at)
        {
            return Err(InvalidSliceRules);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InvalidSliceRules;
impl fmt::Display for InvalidSliceRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("切片规则包含无效生命、间隔、几何或固定波次")
    }
}
impl std::error::Error for InvalidSliceRules {}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GameStatus {
    Running,
    Won,
    Lost,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MowerStatus {
    Ready,
    Moving,
    Spent,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LawnMower {
    pub row: u8,
    pub x: i32,
    pub status: MowerStatus,
}
/// 所有会影响后续结果的切片状态，包括不可见计时与ID计数。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Board {
    pub(crate) rules: SliceRules,
    pub(crate) plants: Vec<Plant>,
    pub(crate) zombies: Vec<Zombie>,
    pub(crate) peas: Vec<Pea>,
    pub(crate) suns: Vec<Sun>,
    pub(crate) mowers: Vec<LawnMower>,
    pub(crate) selected: Option<PlantKind>,
    pub(crate) paused: bool,
    pub(crate) status: GameStatus,
    pub(crate) next_id: EntityId,
    pub(crate) next_spawn: usize,
    pub(crate) sky_sun_at: Tick,
    pub(crate) card_ready_at: [Tick; 2],
}
impl Board {
    pub(crate) fn new(rules: SliceRules) -> Self {
        Self {
            sky_sun_at: rules.sky_sun_interval,
            mowers: (0..ROWS)
                .map(|row| LawnMower {
                    row,
                    x: rules.mower_trigger_x,
                    status: MowerStatus::Ready,
                })
                .collect(),
            rules,
            plants: vec![],
            zombies: vec![],
            peas: vec![],
            suns: vec![],
            selected: None,
            paused: false,
            status: GameStatus::Running,
            next_id: 1,
            next_spawn: 0,
            card_ready_at: [0; 2],
        }
    }
    pub fn rules(&self) -> &SliceRules {
        &self.rules
    }
    pub fn plants(&self) -> &[Plant] {
        &self.plants
    }
    pub fn zombies(&self) -> &[Zombie] {
        &self.zombies
    }
    pub fn peas(&self) -> &[Pea] {
        &self.peas
    }
    pub fn suns(&self) -> &[Sun] {
        &self.suns
    }
    pub fn mowers(&self) -> &[LawnMower] {
        &self.mowers
    }
    pub const fn selected(&self) -> Option<PlantKind> {
        self.selected
    }
    pub const fn paused(&self) -> bool {
        self.paused
    }
    pub const fn status(&self) -> GameStatus {
        self.status
    }
    pub const fn spawned(&self) -> usize {
        self.next_spawn
    }
    pub fn cooldown_remaining(&self, kind: PlantKind, tick: Tick) -> Tick {
        self.card_ready_at[kind.index()].saturating_sub(tick)
    }
    pub fn sun_at(&self, point: Point) -> Option<EntityId> {
        self.suns
            .iter()
            .rev()
            .find(|sun| {
                let dx = i64::from(sun.position.x) - i64::from(point.x);
                let dy = i64::from(sun.position.y) - i64::from(point.y);
                let radius = i64::from(self.rules.sun_radius);
                dx.abs() <= radius && dy.abs() <= radius && dx * dx + dy * dy <= radius * radius
            })
            .map(|sun| sun.id)
    }
    pub(crate) fn id(&mut self) -> EntityId {
        let id = self.next_id;
        self.next_id = id.checked_add(1).expect("实体标识已达到上限");
        id
    }
}

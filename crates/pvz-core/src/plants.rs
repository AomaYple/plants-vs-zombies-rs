use crate::{Cell, EntityId, Tick};

/// 当前切片支持的两张植物卡。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlantKind {
    Sunflower,
    Peashooter,
}
impl PlantKind {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Sunflower => 0,
            Self::Peashooter => 1,
        }
    }
}
/// 核心拥有生命与动作计时，表现层只读。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Plant {
    pub id: EntityId,
    pub kind: PlantKind,
    pub cell: Cell,
    pub health: u32,
    pub(crate) action_at: Tick,
}

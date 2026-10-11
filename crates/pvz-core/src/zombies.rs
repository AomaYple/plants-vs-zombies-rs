use crate::{EntityId, Tick};
/// 普通僵尸的整数模拟状态。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Zombie {
    pub id: EntityId,
    pub row: u8,
    pub x: i32,
    pub health: u32,
    pub(crate) bite_at: Tick,
}

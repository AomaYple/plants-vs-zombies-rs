use crate::{EntityId, Point, Tick};
/// 以百分之一逻辑像素为单位的豌豆。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pea {
    pub id: EntityId,
    pub row: u8,
    pub x: i32,
}
/// 阳光只由核心产生、收集和过期。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Sun {
    pub id: EntityId,
    pub position: Point,
    pub amount: u32,
    pub(crate) expires_at: Tick,
}

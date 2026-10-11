use crate::{Cell, EntityId, PlantKind};
/// 模拟输入命令；切片中的控制与规则由核心按提交顺序处理。
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
    /// 增加阳光。
    AddSun {
        amount: u32,
    },
    /// 消耗阳光。
    SpendSun {
        amount: u32,
    },
    SelectCard {
        kind: PlantKind,
    },
    Plant {
        cell: Cell,
    },
    CollectSun {
        id: EntityId,
    },
    Shovel {
        cell: Cell,
    },
    Pause,
    Resume,
    Restart,
}

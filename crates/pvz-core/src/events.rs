use crate::{Cell, EntityId, PlantKind, Point, Tick};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Rejection {
    InvalidCell,
    Occupied,
    NoSelection,
    NotEnoughSun,
    CoolingDown,
    Paused,
    Ended,
    MissingSun,
    MissingPlant,
    Unsupported,
    BalanceOverflow,
}

/// 确定性模拟向表现层公开的状态变化事件。
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Event {
    /// 一个 tick 开始。
    TickStarted {
        tick: Tick,
    },
    /// 阳光增加成功。
    SunAdded {
        amount: u32,
        total: u32,
    },
    /// 阳光消耗成功。
    SunSpent {
        amount: u32,
        total: u32,
    },
    /// 阳光不足，消耗被拒绝。
    SunSpendRejected {
        amount: u32,
        available: u32,
    },
    /// 阳光增加溢出，命令被拒绝。
    SunAddRejected {
        amount: u32,
        available: u32,
    },
    /// 一个 tick 完成。
    TickCompleted {
        tick: Tick,
    },
    CardSelected {
        kind: PlantKind,
    },
    CommandRejected {
        reason: Rejection,
    },
    Planted {
        id: EntityId,
        kind: PlantKind,
        cell: Cell,
    },
    PlantRemoved {
        id: EntityId,
    },
    SunSpawned {
        id: EntityId,
        position: Point,
        amount: u32,
    },
    SunCollected {
        id: EntityId,
        amount: u32,
        total: u32,
    },
    SunExpired {
        id: EntityId,
    },
    ZombieSpawned {
        id: EntityId,
        row: u8,
    },
    ProjectileFired {
        id: EntityId,
        plant: EntityId,
        row: u8,
    },
    ProjectileHit {
        id: EntityId,
        zombie: EntityId,
    },
    Damaged {
        target: EntityId,
        remaining: u32,
    },
    ZombieDied {
        id: EntityId,
    },
    MowerStarted {
        row: u8,
    },
    Paused,
    Resumed,
    Restarted,
    Victory,
    Defeat,
}

/// 一次模拟推进产生的有序事件集合。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StepEvents {
    tick: Tick,
    events: Vec<Event>,
}

impl StepEvents {
    pub(crate) fn new(tick: Tick, events: Vec<Event>) -> Self {
        Self { tick, events }
    }

    /// 返回本次推进完成的 tick。
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    /// 以稳定顺序查看本次推进产生的事件。
    pub fn as_slice(&self) -> &[Event] {
        &self.events
    }
}

use crate::Tick;

/// 空棋盘模拟阶段对表现层可见的事件。
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Event {
    /// 一个 tick 开始。
    TickStarted { tick: Tick },
    /// 阳光增加成功。
    SunAdded { amount: u32, total: u32 },
    /// 阳光消耗成功。
    SunSpent { amount: u32, total: u32 },
    /// 阳光不足，消耗被拒绝。
    SunSpendRejected { amount: u32, available: u32 },
    /// 阳光增加溢出，命令被拒绝。
    SunAddRejected { amount: u32, available: u32 },
    /// 一个 tick 完成。
    TickCompleted { tick: Tick },
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

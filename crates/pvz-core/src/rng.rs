/// 不依赖系统时间的确定性伪随机数发生器。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    /// 使用显式种子创建发生器。
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// 返回下一项确定性 `u64`，算法属于模拟合同的一部分。
    pub fn next_u64(&mut self) -> u64 {
        if self.state == 0 {
            self.state = 0x9e37_79b9_7f4a_7c15;
        }
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(2_685_821_657_736_338_717)
    }

    /// 返回当前内部状态，用于状态摘要和回放核对。
    pub const fn state(self) -> u64 {
        self.state
    }
}

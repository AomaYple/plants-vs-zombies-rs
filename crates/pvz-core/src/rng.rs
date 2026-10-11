use oorandom::Rand64;

/// 使用显式 seed、无系统熵的开发阶段随机数领域包装。
///
/// 固定使用 `oorandom 11.1.5` 的 `Rand64` 合同，不代表原版随机算法。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DeterministicRng {
    inner: Rand64,
}

impl DeterministicRng {
    /// 使用显式种子创建发生器。
    pub fn new(seed: u64) -> Self {
        Self {
            inner: Rand64::new(u128::from(seed)),
        }
    }

    /// 返回下一项确定性 `u64`，算法属于模拟合同的一部分。
    pub fn next_u64(&mut self) -> u64 {
        self.inner.rand_u64()
    }

    /// 返回当前内部状态，用于状态摘要和回放核对。
    pub fn state(&self) -> (u128, u128) {
        self.inner.state()
    }
}

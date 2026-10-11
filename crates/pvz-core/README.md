# pvz-core

纯 Rust 模拟 crate，提供空棋盘规则 1.0 和白天开发切片规则 2.0；不依赖引擎、资源、设备 I/O 或墙钟。切片包含 5×9 棋盘、两种植物、普通僵尸、豌豆、阳光、割草机、固定波次和胜负流程。完整持久回放/存档格式尚未实现，当前回放由纯测试提交固定命令验证。

`GameState::new/with_clock` 保留空棋盘 1.0 和基础阳光命令；`GameState::day(seed, initial, rules)` 校验 `SliceRules` 后创建切片 2.0。默认开发时钟为 60 tick/s，`RulesVersion::current()` 为 2.0，空棋盘摘要仍明确返回 1.0。时钟和切片数值均不是原版规格。

`GameState::step` 严格按提交顺序处理命令，事件保留因果顺序。空棋盘每次调用推进一个 tick，不消耗 RNG；切片仅在本批无重开、最终未暂停且局面运行时推进。暂停、恢复、重开、终局与时钟上限合同见 [开发切片合同](../../docs/behavior/development-slice.md)，预计推进发生溢出时，在修改前明确拒绝整个批次。

`StateDigest` 包含当前与初始关卡/时钟/阳光合同、seed、RNG 完整 `(u128, u128)` 状态、规则、实体、计时、ID、波次游标、选择、暂停和终局。它用于判断未来演化是否一致，可克隆但不可 `Copy`，不是持久序列化格式。

`DeterministicRng` 是锁定 `oorandom 11.1.5` 的 `Rand64` 最小领域包装，以显式 `u64` seed 转换为 `u128` 初始化，不读取系统熵，也不声称采用原版算法。早期自写 xorshift 的输出已被替换，旧开发状态不兼容；后续已验收合同或持久回放行为变更须声明版本与兼容策略，当前没有迁移器消费者。

`tests/determinism.rs` 和 `tests/vertical_slice.rs` 使用原创输入覆盖顺序、边界、错误、完整摘要、暂停/重开和胜负回放。`benches/stepping.rs` 提供空棋盘、命令与切片场景，初始化/种植预热在计时外；切片调用次数与实际推进 tick 数须分别核对。当前测量和限制见 [阶段性结果](../../docs/behavior/vertical-slice-results.md)，只编译基准不算性能测量。

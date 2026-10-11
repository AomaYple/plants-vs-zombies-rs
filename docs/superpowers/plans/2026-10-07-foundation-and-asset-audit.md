# 基础设施、资源审计与首个可玩切片实施计划

> 当前已进入 Rust 基础设施实现阶段。版本化资源已导入，2026-10-10 同步资源方案。每项任务独立验收；具体代理工具由执行环境与适用指令决定。

**目标：** 建立包含版本化参考资源的 Rust workspace、资源审计与解析基础、跨平台表现探针和首个白天可玩切片，完成相应行为、覆盖率与性能证据复核；原版程序和临时派生产物不进入公开交付。

**架构：** 按任务依次注册 `pvz-data`、`pvz-core`、`pvz-assets`、`pvz-asset-tool` 和 `pvz-app`，每次只交付一个可验证边界。core 只依赖数据/纯逻辑；assets 解析资源为 CPU 模型；app 以 Macroquad 装配仓库资源；CLI 按仓库清单检查资源，转换输出与输入分离。

**技术栈：** Rust 1.99.0、Edition 2024、Cargo resolver 3、Macroquad、Serde + JSON（已用于 `pvz-data` schema 合同）、quick-xml 0.41（已用于资源 XML 子集解析）、Cargo test、rustfmt、Clippy、cargo-deny、GitHub Actions。

**设计文档：** `docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md`

## 实施前提与公共验收

长期约束遵循 [项目原则](../../principles/README.md)，架构遵循 [模块合同](../../architecture/README.md)，每次提交完成 [贡献指南的适用验收](../../../CONTRIBUTING.md#验证流程)。下列任务命令是局部反馈，不替代提交门槛；新增 package 同步锁文件、lint 与 CI，覆盖率和热路径基准随相关实现接入。

- 当前参考基线 ID 为 `goty-zh-a7ab7df8-7f1e824e`，但版本字段差异必须保留并可追溯。
- 原版模拟 tick 未测量；不能把 60 Hz 作为已知规格。
- 逻辑视口暂按 800×600；1400×600 背景的裁剪/滚动待测量。
- 已明确行为优先测试先行，探索探针先获取证据，正式交付前补齐适用测试。测试先明确预期合同，不把未注册 package 或环境错误当成行为证据。
- 使用版本化参考资源前核对 `assets/manifest.json`、资源文件指纹和参考基线；需要原版 EXE 行为测量时另确认本机 EXE 身份与可访问性。仓库资源缺失或损坏必须修复，不静默回退到占位素材；原创夹具与占位探针只验收各自的隔离合同，原版测量和还原验收单独完成。
- 下列文件清单与接口是任务边界，按实际消费者细化；不机械创建全部未来 ID、空模块、公开 API 或占位文件。新增 package 时同步其 README、根 README、路线图和 workspace 阶段元数据，避免阶段说明滞后。
- 首次使用额外验收工具时确认 cargo-deny、cargo-llvm-cov 等版本和 CI 安装方式；缺少工具的必要检查保持待验收，不视为通过。

---

### 任务 1：注册数据模型 package

**文件：**

- 新建：`crates/pvz-data/Cargo.toml`
- 新建：`crates/pvz-data/src/lib.rs`
- 新建：`crates/pvz-data/src/ids.rs`
- 新建：`crates/pvz-data/src/version.rs`
- 新建：`crates/pvz-data/tests/schema.rs`
- 修改：`Cargo.toml`
- 新建：`Cargo.lock`
- 修改：`.github/workflows/ci.yml`

**接口：**

- 提供 `AssetId`、`PlantId`、`ZombieId`、`LevelId`、`RulesVersion`、`ReferenceBaselineId`。
- 只提供版本化的纯数据类型，不包含文件或引擎 API。

- [x] 注册最小 package 和测试入口，继承 workspace lint，生成 `Cargo.lock`，并启用适用的实现 CI 与依赖审计。
- [x] 添加测试，对小型原创夹具进行序列化/反序列化，并拒绝不支持的模型版本。
- [x] 运行 `cargo test -p pvz-data --locked`，确认失败先指向缺少库入口，随后由实现补齐合同。
- [x] 添加最小的强类型 ID 和显式版本新类型，实现夹具合同。
- [x] 运行 `cargo fmt --all -- --check`、`cargo test -p pvz-data --locked` 和 `cargo clippy -p pvz-data --all-targets --locked -- -D warnings`，完成全部适用的提交检查。
- [x] 提交 `feat(data): 添加版本化领域标识符`。

### 任务 2：注册纯模拟 package

**文件：**

- 新建：`crates/pvz-core/Cargo.toml`
- 新建：`crates/pvz-core/src/lib.rs`
- 新建：`crates/pvz-core/src/clock.rs`
- 新建：`crates/pvz-core/src/commands.rs`
- 新建：`crates/pvz-core/src/events.rs`
- 新建：`crates/pvz-core/src/rng.rs`
- 新建：`crates/pvz-core/src/world.rs`
- 新建：`crates/pvz-core/tests/determinism.rs`
- 新建：`crates/pvz-core/benches/stepping.rs`
- 修改：`Cargo.toml`
- 修改：`.github/workflows/ci.yml`
- 修改：`docs/behavior/README.md`
- 修改：`docs/behavior/reference-baseline.md`
- 新建：`docs/adr/0003-simulation-clock.md`

**接口：**

- `GameState::new(seed, initial_state) -> GameState`.
- `GameState::step(commands: &[Command]) -> StepEvents`.
- `GameState::state_digest() -> StateDigest`.
- `SimulationClock` 保存显式且可替换的 tick 合同；参考测量确认原版时基前，开发参数保持明确标记。

- [x] 定义可替换的时钟合同，记录尚待确认的参考时基；标记开发参数，不将其声称为原版行为。
- [x] 注册 package 和测试入口；添加确定性回放测试，让两个实例接收相同命令，并断言状态摘要和事件一致。
- [x] 添加明确的预期 tick/状态转移、RNG 测试向量及命令/事件顺序断言；改变必须影响结果的输入或初始状态，以捕获固定摘要或被忽略的命令。只验证两个实例相等并不充分。
- [x] 运行 `cargo test -p pvz-core --test determinism --locked`，确认失败先指向缺少库入口，随后由实现补齐合同。
- [x] 实现空棋盘推进、命令排序、确定性 RNG 封装和事件排序，暂不加入植物或僵尸。
- [x] 添加可重复的模拟推进基准并记录首个基线；覆盖率报告在任务 8 汇总并复核。
- [x] 运行 core 测试和 `cargo clippy -p pvz-core --all-targets --locked -- -D warnings`；检查依赖树，确认不依赖引擎或平台。
- [x] 提交 `feat(core): 添加确定性模拟基础`。

### 任务 3：建立资源审计库

**文件：**

- 新建：`crates/pvz-assets/Cargo.toml`
- 新建：`crates/pvz-assets/src/lib.rs`
- 新建：`crates/pvz-assets/src/manifest.rs`
- 新建：`crates/pvz-assets/src/diagnostics.rs`
- 新建：`crates/pvz-assets/src/reanim.rs`
- 新建：`crates/pvz-assets/src/particles.rs`
- 新建：`crates/pvz-assets/tests/fixtures.rs`
- 新建：`crates/pvz-assets/benches/parsing.rs`
- 新建：`tests/fixtures/assets-minimal/README.md`
- 修改：`Cargo.toml`
- 修改：`.github/workflows/ci.yml`

**接口：**

- `AssetRoot::open(path: &Path) -> Result<AssetIndex, AssetError>`.
- `AssetIndex::audit() -> AuditReport`.
- `parse_reanim(text: &str) -> Result<AnimationClip, AssetError>`.
- `parse_particle(text: &str) -> Result<ParticleSystem, AssetError>`.
- API 不接受或内嵌固定的用户绝对路径。

- [x] 创建不含解包游戏内容的微型原创 XML 夹具；资源集成测试额外只读版本化资源清单。PNG 元数据尚无对应解析 API，不预建无消费者的二进制夹具。
- [x] 本任务未新增被跟踪的二进制夹具或参考资源；原版可执行文件和未登记临时载荷仍由清单与 CI 拒绝。
- [x] 添加有效清单引用、文件缺失、路径大小写碰撞、畸形 XML 和报告顺序确定性的测试。
- [x] 解析器实现前运行资源测试，确认失败先指向缺少库入口，随后由实现补齐合同。
- [x] 实现受支持的格式子集，诊断包含相对路径；模型外的格式扩展保持在解析器合同之外并通过所需字段校验，避免静默改变当前模型含义。
- [x] 将清单目录读取限制在显式 `AssetRoot` 宿主入口；清单、Reanim 和粒子解析接收字节或文本，使应用和浏览器资源提供接口无需固定路径即可复用。
- [x] 添加解析耗时基准并记录首个基线；覆盖率在任务 8 汇总并复核，当前实现使用原创成功/错误路径测试。
- [x] 运行 `cargo test -p pvz-assets --locked`、Clippy，并只读审计版本化清单与资源；完成提交检查，不写入来源目录。
- [x] 提交 `feat(assets): 添加资源审计模型与解析器`。

### 任务 4：添加仓库资源审计 CLI

**文件：**

- 新建：`tools/pvz-asset-tool/Cargo.toml`
- 新建：`tools/pvz-asset-tool/src/main.rs`
- 新建：`tools/pvz-asset-tool/tests/cli.rs`
- 修改：`Cargo.toml`
- 修改：`assets/README.md`
- 修改：`tools/pvz-asset-tool/README.md`

**接口：**

- 在仓库根目录运行 `pvz-asset-tool audit --format text|json`。
- 在仓库根目录运行 `pvz-asset-tool inspect-reanim <NAME>`。
- 命令读取 `assets/manifest.json` 和已登记输入；输入缺失或无效时显式失败。测试使用隔离的夹具仓库，不通过运行时配置覆盖资源根目录。

- [ ] 添加 CLI 测试，覆盖帮助、清单缺失或无效、哈希不匹配、夹具审计成功、JSON 顺序和绝对路径脱敏。
- [ ] 命令实现前运行 CLI 测试，确认预期失败。
- [ ] 使用 assets 库实现参数解析和报告序列化。
- [ ] 使用 CLI 只读检查仓库资源，确认 3229 条清单记录覆盖 3228 个解包文件和 1 个来源包，不生成或修改来源文件。
- [ ] 提交 `feat(tool): 添加仓库资源审计命令`。

### 任务 5：添加 Macroquad 验证应用

**文件：**

- 新建：`crates/pvz-app/Cargo.toml`
- 新建：`crates/pvz-app/src/main.rs`
- 新建：`crates/pvz-app/src/input.rs`
- 新建：`crates/pvz-app/src/viewport.rs`
- 新建：`crates/pvz-app/tests/viewport.rs`
- 修改：`Cargo.toml`
- 修改：`.github/workflows/ci.yml`

**接口：**

- `Viewport::fit(window_size, logical_size) -> Viewport`.
- `Viewport::to_logical(physical_point) -> Option<LogicalPoint>`.
- `InputEvent` 的变体包括 `PointerDown`、`PointerUp`、`PointerMove`、`PointerCancel`、`Pause`、`Resume`。

- [ ] 添加纯视口测试，覆盖 4:3、16:9、高 DPI 和视口外输入。
- [ ] 坐标变换实现前运行测试，确认预期失败。
- [ ] 实现带占位背景和可见逻辑网格诊断的 Macroquad 窗口，渲染代码与 core 分离。
- [ ] 通过平台资源提供接口加载仓库清单登记的最小原版图片/动画集合；将资源缺失诊断和路径处理的验证与占位探针分开。
- [ ] 使用原创夹具验证 Reanim 所需的仿射变换，包括两个轴的独立错切、图层顺序和裁剪；扩展动画播放器前确认后端方案。
- [ ] 验证音效播放及音乐解码、循环和暂停方案，包括 MO3 兼容性与浏览器用户手势要求；评估全部六个目标的依赖及许可兼容性，记录尚缺的平台证据。
- [ ] 运行 `cargo check -p pvz-app --locked`、桌面启动冒烟检查和仅包含 app 的 wasm 构建；在浏览器中实际启动占位探针，验证输入和高 DPI 映射。分别记录 Windows、macOS、Linux 和 Web 的构建/运行证据；不可用的环境保持待验收，不能仅凭编译通过将 P2 标记为完成。
- [ ] 提交 `feat(app): 添加跨平台视口验证应用`。

### 任务 6：实现首个可玩垂直切片

**文件：**

- 修改：`crates/pvz-core/src/world.rs`
- 新建：`crates/pvz-core/src/plants.rs`
- 新建：`crates/pvz-core/src/zombies.rs`
- 新建：`crates/pvz-core/src/projectiles.rs`
- 新建：`crates/pvz-core/src/board.rs`
- 新建：`crates/pvz-core/tests/vertical_slice.rs`
- 修改：`crates/pvz-app/src/main.rs`
- 修改：`docs/behavior/README.md`

**接口：**

- core 命令覆盖选卡、种植、收集阳光、铲除、暂停和重开。
- core 事件覆盖种植、阳光生成/收集、投射物发射/命中、伤害、僵尸死亡、割草机启动、胜利和失败。
- app 消费事件以驱动画面和音频，不能直接施加伤害。

- [ ] 测量切片所需的参考模拟时基、时间间隔、碰撞和事件顺序；原版行为验收前替换开发参数并记录证据。
- [ ] 将数值作为原版规则前，在行为文档中记录受控、重复的参考案例和预期结果；用这些案例比对可玩切片，区分参考行为一致性与内部回放确定性。
- [ ] 添加测试，覆盖 5×9 种植、阳光消耗、豌豆命中、僵尸啃食、割草机启动、固定波次、胜负、暂停和重开。
- [ ] 实体实现前运行垂直切片测试，确认预期失败。
- [ ] 仅实现最小的数据驱动向日葵（`Sunflower`）、豌豆射手（`Peashooter`）、普通僵尸（`Zombie`）、豌豆（`Pea`）、阳光（`Sun`）、割草机（`LawnMower`）和一次固定波次。
- [ ] 将覆盖率和热路径基准扩展到切片；运行 core 测试及隔离的无资源 app 合同检查，再使用已登记原版资源进行手工 Macroquad 冒烟检查。确认切片画面/音频消费模拟事件，资源缺失时显式失败；占位资源不能替代原版表现验收。
- [ ] 提交 `feat(core): 添加白天关卡的最小可玩切片`。

### 任务 7：内容扩展前复核切片证据

**文件：**

- 修改：`README.md`
- 修改：`CONTRIBUTING.md`
- 修改：`.github/workflows/ci.yml`
- 修改：`docs/adr/0003-simulation-clock.md`
- 新建：`docs/behavior/vertical-slice-results.md`

- [ ] 汇总任务 2 的时钟合同、任务 6 所需的参考测量、输入映射和切片比对，记录限制及尚存的行为缺口。
- [ ] 复核 package 注册时启用的 CI，包括依赖审计和适用的桌面/Web 检查；补齐实际缺口，不推迟前序任务的要求。
- [ ] 确认 `git ls-files` 中的参考资源均已登记清单，且不包含未登记资源包、原版可执行文件、用户绝对路径或敏感信息。
- [ ] 复核公开名称和 README 措辞，保留“非官方”和 MIT 适用范围说明。
- [ ] 提交 `docs: 记录可玩切片的验收结果`。

### 任务 8：复核覆盖率与性能基线

**文件：**

- 修改：`CONTRIBUTING.md`
- 修改：`.github/workflows/ci.yml`
- 修改：`docs/performance/README.md`
- 修改：`docs/testing/README.md`

**接口：**

- 覆盖率命令：`cargo llvm-cov --workspace --locked --summary-only`，采用 CONTRIBUTING.md 定义的有效 feature 矩阵。
- 基准命令：`cargo bench --workspace --locked`。
- Criterion 基准放在被测热路径所属的 package 中；基准名称标明输入规模和规则版本。

- [ ] 复核实现过程中收集的覆盖率报告，补齐遗漏的关键/错误路径，记录有依据的门槛和尚存缺口。
- [ ] 复核确定性且不依赖资源的基准输入；正确性测试保持并行，性能测量避免与其他测试或任务争抢资源。
- [ ] 确认现有文本/资源检查仍覆盖新增跟踪文件的分类、清单哈希和公共夹具来源。
- [ ] 复核文件、模块、API、依赖的消费者及有效 target/feature 组合；删除过时占位和未使用结构，删除前核对调用方、文档、测试、生成与配置入口。
- [ ] 汇总耗时、适用的内存/分配基线和覆盖率限制；任何优化比较都附带正确性证据。
- [ ] 提交 `test: 完善覆盖率与性能验收基线`。

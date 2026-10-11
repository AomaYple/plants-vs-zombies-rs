# 基础设施、资源审计与首个可玩切片实施计划

> 当前状态（2026-10-12）：用户已重新授权执行计划。任务 1–5 保持完成；任务 6 的开发切片已完成本地工程验收和局部独立复核，任务 8 的错误边界、覆盖率与基准合同补强通过局部独立复核。任务 7 的最终证据和十场景计时已汇总，任务 6–8 的开发工程范围及最终整分支独立复核均通过，Git 交付按贡献流程及对应 PR/CI 记录核验；原版测量、完整动态表现、Web 实际失败和其余平台运行继续待验。当前证据见 [切片验收结果](../../behavior/vertical-slice-results.md)。

**目标：** 建立包含版本化参考资源的 Rust workspace、资源审计与解析基础、跨平台表现探针和首个白天可玩切片，完成相应行为、覆盖率与性能证据复核；原版程序和临时派生产物不进入公开交付。

**架构：** 已注册 `pvz-data`、`pvz-core`、`pvz-assets`、`pvz-asset-tool` 和 `pvz-app`。core 只依赖数据/纯逻辑；assets 解析资源为 CPU 模型；app 以 Bevy 装配仓库资源；CLI 按仓库清单只读检查资源。转换工具按后续实际需求引入，输出与输入分离。

**技术栈：** Rust 1.99.0、Edition 2024、Cargo resolver 3、Bevy、Serde + JSON（已用于 `pvz-data` schema 合同）、quick-xml 0.41（已用于资源 XML 子集解析）、Cargo test、rustfmt、Clippy、cargo-deny、GitHub Actions。

**设计文档：** `docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md`

## 任务状态与恢复入口

| 任务 | 当前状态 | 剩余工作 |
| --- | --- | --- |
| 1–4：数据、模拟、资源库与 CLI | 已完成并复核 | CLI 闭管道和 XML 语义已由任务 8 补强；后续解析扩展按实际消费者推进 |
| 5：Bevy 验证应用 | 已完成并复核 | 截图等待已补最终退出合同；instrumented shader 提示及中文首字裁切保持已知限制，未验平台另列 |
| 6：白天开发切片 | 开发工程验收和局部独立复核通过 | 原版规则与动态表现另行验收；Web 实际失败及其他平台运行待补 |
| 7：切片证据复核 | 已完成并独立复核 | 后续依据新参考测量和平台证据更新专题 |
| 8：覆盖率与性能复核 | 已完成并独立复核，最终计时与消费者核查通过 | 后续变更继续运行适用覆盖率、回放及基准 |
| 最终收口 | 本地工程验收和整分支独立复核通过 | 按对应 PR/CI 核验 Git 交付及主线，随后清理任务分支 |

本轮复用任务 1–5 的已完成结果，不重做已完成任务。`development-slice-validated` 表示开发工程合同通过本地验收，不表示原版规则或六平台均已完成。勾选项只代表所写范围完成；本地验收、独立复核、远端 CI、原版行为一致性与目标平台支持分别记录。

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
- [x] 运行 `cargo test -p pvz-data --locked`，核对最终合同通过；早期入口缺失属于配置失败，不计作有效行为红测。
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
- [x] 运行 `cargo test -p pvz-core --test determinism --locked`，核对最终合同通过；早期入口缺失属于配置失败，不计作有效行为红测。
- [x] 实现空棋盘推进、按提交顺序执行命令、成熟确定性 RNG 封装和有因果顺序的事件，暂不加入植物或僵尸。
- [x] 添加可重复的模拟推进基准并记录首个基线；覆盖率门槛已随实现进入 CI，任务 8 汇总并复核。
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
- [x] 解析器实现前运行资源测试，核对最终合同通过；早期入口缺失属于配置失败，不计作有效行为红测。
- [x] 实现受支持的格式子集，诊断包含相对路径；模型外的格式扩展保持在解析器合同之外并通过所需字段校验，避免静默改变当前模型含义。
- [x] 将清单目录读取限制在显式 `AssetRoot` 宿主入口；清单、Reanim 和粒子解析接收字节或文本，使应用和浏览器资源提供接口无需固定路径即可复用。
- [x] 添加解析耗时基准并记录首个基线；覆盖率已随实现接入，任务 8 汇总并复核，当前实现使用原创成功/错误路径测试。
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

- [x] 添加 CLI 测试，覆盖帮助、清单缺失或无效、哈希不匹配、夹具审计成功、JSON 顺序和绝对路径脱敏。
- [x] 命令实现前运行 CLI 测试，确认预期失败。
- [x] 使用 clap 解析参数，assets 库验证输入，Serde 序列化显式报告。
- [x] 使用 CLI 只读检查仓库资源，确认 3229 条清单记录覆盖 3228 个解包文件和 1 个来源包，不生成或修改来源文件。
- [x] 提交 `feat(tool): 添加仓库资源审计命令`。

### 任务 5：添加 Bevy 验证应用

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

- [x] 添加纯视口测试，覆盖 4:3、16:9、高 DPI 和视口外输入。
- [x] 坐标变换实现前运行测试，确认预期失败。
- [x] 实现带占位背景和可见逻辑网格诊断的 Bevy 窗口，渲染代码与 core 分离。
- [x] 通过平台资源提供接口加载仓库清单登记的最小原版图片/动画集合；将资源缺失诊断和路径处理的验证与占位探针分开。
- [x] 使用原创夹具验证 Reanim 所需的仿射变换，包括两个轴的独立错切、图层顺序和裁剪；扩展动画播放器前确认后端方案。
- [x] 验证 OGG 音效/音乐解码、播放游标及暂停恢复方案与浏览器用户手势；记录循环播放请求已验证、完整曲尾未验收，MO3 明确未支持。记录六平台依赖/许可路线及实际证据缺口。
- [x] 完成 Windows 隐藏窗口冒烟、默认/probe app WASM 构建与浏览器实际启动；验证有序输入、DPR 1 下 1280×720/2K/4K framebuffer 尺寸及坐标映射。高 DPI 数学合同由纯测试覆盖，真实 DPR 2/4、设备触摸和 macOS/Linux 仍待验收，见 [平台探针结果](../../behavior/platform-probe-results.md)。
- [x] 提交 `feat(app): 添加跨平台视口验证应用`。

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
- [x] 添加测试，覆盖 5×9 种植、阳光消耗、豌豆命中、僵尸啃食、割草机启动、固定波次、胜负、暂停和重开。
- [x] 实体实现前运行垂直切片测试，确认预期失败。
- [x] 仅实现最小的数据驱动向日葵（`Sunflower`）、豌豆射手（`Peashooter`）、普通僵尸（`Zombie`）、豌豆（`Pea`）、阳光（`Sun`）、割草机（`LawnMower`）和一次固定波次。
- [x] 将纯测试覆盖率和 core 时间基准扩展到切片，追加同一次 instrumented Windows 构建的实际运行覆盖率；最终 feature 范围和基准规模语义由任务 8 复核。
- [x] 完成 core 与无资源 app 合同测试、登记资源集成检查及默认/all-features workspace 门槛；Windows 隐藏窗口完成切片胜利与音频冒烟，实际查看 GPU 截图。当前画面使用六类对象的登记静态构图和事件音效，失败不回退占位。
- [x] 完成默认/probe WASM 构建和 Web 部分操作：重开、两卡种植、暂停及恢复。
- [x] 补齐最新 Web 包的阳光收集、铲除、两卡胜利与 DPR 1 下切片 2K/4K 指定逻辑格子输入证据；大截图存在裁切/缩放，不冒称全帧像素验收。
- [ ] 补齐 Web 实际失败流程；无防守的开发波次可被割草机清空而胜利，不能把该运行当作失败验收。真实 DPR 2/4、触摸设备与其他平台继续待验。
- [x] 完成任务 6 的独立规范与质量复核，修复有序键盘/指针、同批重开后铲除与旧阳光 ID、截图最终退出和日志时刻标注；音频补强经另一次独立复核通过，并完成最终 native/Web 复验。
- [x] 已提交 `feat(core): 添加白天关卡的最小可玩切片`；阶段性提交当时待独立复核，本轮局部复核和补强已通过，新交付仍按本轮收口流程完成。

开发切片允许明确标记的开发参数与静态构图。上面的两项原版规则测量仍未完成；完整 idle/射击/啃食/死亡动画混合、原版粒子和 MO3 由 [路线图 P4](../../roadmap.md#p4规则与内容)继续接入，在相应原版表现验收前完成。不能把开发切片的内部验证或静态素材加载当作完整原版还原。

### 任务 7：内容扩展前复核切片证据

**文件：**

- 修改：`README.md`
- 修改：`CONTRIBUTING.md`
- 修改：`.github/workflows/ci.yml`
- 修改：`docs/adr/0003-simulation-clock.md`
- 修改：`docs/behavior/vertical-slice-results.md`

- [x] 保留暂停时的原始证据，按最终代码更新验收记录，区分开发工程合同、原版还原与各平台状态。
- [x] 汇总任务 2 的时钟合同、任务 6 所需的参考测量、输入映射和切片比对，记录限制及尚存的行为缺口。
- [x] 复核 package 注册时启用的 CI，包括依赖审计和适用的桌面/Web 检查；补齐实际缺口，不推迟前序任务的要求。
- [x] 确认 `git ls-files` 中的参考资源均已登记清单，且不包含未登记资源包、原版可执行文件、用户绝对路径或敏感信息。
- [x] 复核公开名称和 README 措辞，保留“非官方”和 MIT 适用范围说明。
- [x] 随本轮完整目的提交验收文档，使用 `fix: 完成开发切片边界修复与质量验收`，不拆开修复所需测试和说明。

### 任务 8：复核覆盖率与性能基线

**文件：**

- 修改：`CONTRIBUTING.md`
- 修改：`.github/workflows/ci.yml`
- 修改：`docs/performance/README.md`
- 修改：`docs/testing/README.md`
- 修改：`tools/pvz-asset-tool/src/main.rs` 及 CLI 回归测试
- 修改：`crates/pvz-assets/tests/contracts.rs` 等相关解析测试
- 修改：`crates/pvz-app/src/main.rs` 及截图退出边界的相关测试
- 修改：受影响 package 的 `benches/` 与专题结果记录

**接口：**

- 覆盖率命令：`cargo llvm-cov --workspace --locked --summary-only`，采用 CONTRIBUTING.md 定义的有效 feature 矩阵。
- 基准命令：`cargo bench --workspace --locked`。
- Criterion 基准放在被测热路径所属的 package 中；基准名称标明输入规模和规则版本。

- [x] 复核实现过程中收集的覆盖率报告，补齐遗漏的关键/错误路径，记录有依据的门槛和尚存缺口。
- [x] 补齐默认/all-features 覆盖率，分别报告纯测试与追加 native 窗口的范围；Web 条件代码另验，不排除 main/render 制造通过，不把未启用的分支覆盖 count 0 当作 100%。
- [x] 补齐资源 XML 的显式空值与缺省、CDATA 成功路径、重复 Field/SystemField 顺序测试，断言实际解析语义。
- [x] 修复 CLI 两处 `eprintln!` 在 stderr 不可写时 panic/退出 101 的边界，改用可失败输出并补 Windows 聚焦子进程回归；按目标平台隔离测试。
- [x] 为 GPU 截图请求后的等待建立最终上限和明确失败退出合同，补充有实际消费者的边界测试。
- [x] 复核 instrumented 运行中的 shader unresolved import WARN，核对同一次构建的日志与实际截图；已有任务 5/6 正常画面证据保留，不屏蔽诊断或复制引擎内部管线。
- [x] 复核确定性且不依赖资源的基准输入；正确性测试保持并行，性能测量避免与其他测试或任务争抢资源。
- [x] 核对 `std::hint::black_box` 等现代惯例及基准标签；切片的 45 株预热、100 只波次总量、1000 次 `step` 调用与实际推进 tick 数分别说明，验证终局冻结的影响，不将波次总量写成同时存活数。
- [x] 确认现有文本/资源检查覆盖 3337 个跟踪及未忽略新文件的分类、清单哈希和公共夹具来源；暂存时再核对实际 Git 集合。
- [x] 复核文件、模块、API、依赖的消费者及有效 target/feature 组合；删除过时占位和未使用结构，删除前核对调用方、文档、测试、生成与配置入口。
- [x] 按最终代码在无本项目应用/测试/构建竞争时复测十个基准，汇总 Criterion 均值/95% 置信区间、适用的内存/分配基线和覆盖率限制；未控制维护者其他活动和 CPU 电源/温度，不声称全机器空闲。OS 进程峰值包含 harness，不冒称核心堆分配。合同改变的旧基线和与 Web 渲染重叠的探索记录保留，不能把不同条件的 Criterion 比较当作优化收益。
- [x] 随本轮完整目的提交质量补强、回归测试和基线说明，最终消息遵循贡献指南。

## 最终收口

- [x] 任务 6–8 的实施与独立复核结果已汇总，遗留项逐项解决或明确记录依据、影响范围与后续验收；不得隐藏有效测试失败。
- [x] 完成整体复核与 [贡献指南的适用验收](../../../CONTRIBUTING.md#验证流程)，同步 README、路线图、设计与专题结果，核对文件/依赖消费者及资源来源。
- [x] 分别说明开发切片、原版还原和六平台的实际状态；原版测量和未运行平台继续显示待验收，不因工程收口自动完成。

本轮使用短期任务分支，完成适用验证、整体复核与必要 CI 后通过 PR 合入，核验主线并清理已完成的任务分支；步骤遵循 [贡献指南](../../../CONTRIBUTING.md#提交推送与评审)，外部操作按用户授权范围执行。不以提交、推送、合入或 CI 成功代替尚未完成的游戏验收。原始运行/覆盖率/计时证据保留在忽略的 `artifacts/`，任务账本和临时脚本在收口时核查保留或清理。后续优先完成原版受控测量，再按路线图推进 P4–P6。

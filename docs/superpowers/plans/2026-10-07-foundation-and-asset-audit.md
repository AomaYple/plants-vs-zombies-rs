# 基础设施、资源审计与首个可玩切片实施计划

> 当前为文档、结构与参考资源阶段。版本化资源已导入，2026-10-10 同步资源方案。用户明确授权实现后再逐项执行，每项任务独立验收；具体代理工具由执行环境与适用指令决定。

**Goal:** 建立包含版本化参考资源的 Rust workspace、资源审计与解析基础、跨平台表现探针和首个白天可玩切片，完成相应行为、覆盖率与性能证据复核；原版程序和临时派生产物不进入公开交付。

**Architecture:** 按任务依次注册 `pvz-data`、`pvz-core`、`pvz-assets`、`pvz-asset-tool` 和 `pvz-app`，每次只交付一个可验证边界。core 只依赖数据/纯逻辑；assets 解析资源为 CPU 模型；app 以 Macroquad 装配仓库资源；CLI 按仓库清单检查资源，转换输出与输入分离。

**Tech Stack:** Rust 1.99.0、Edition 2024、Cargo resolver 3、Macroquad、Serde/JSON 或 RON（选定时记录）、quick-xml 或等价 XML 解析库（选定时记录）、Cargo test、rustfmt、Clippy、cargo-deny、GitHub Actions。

**Spec:** `docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md`

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

### Task 1: Register the data schema package

**Files:**
- Create: `crates/pvz-data/Cargo.toml`
- Create: `crates/pvz-data/src/lib.rs`
- Create: `crates/pvz-data/src/ids.rs`
- Create: `crates/pvz-data/src/version.rs`
- Create: `crates/pvz-data/tests/schema.rs`
- Modify: `Cargo.toml`
- Create: `Cargo.lock`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Produces `AssetId`, `PlantId`, `ZombieId`, `LevelId`, `RulesVersion`, `ReferenceBaselineId`.
- Produces versioned plain data types only; no file or engine APIs.

- [ ] Register the minimal package and test entry, inherit workspace lint, generate `Cargo.lock`, and enable the applicable implementation CI and dependency audit.
- [ ] Add a test that serializes/deserializes a small self-authored fixture and rejects an unsupported schema version.
- [ ] Run `cargo test -p pvz-data --locked`; verify failure identifies the intended API/behavior gap.
- [ ] Add the smallest typed IDs and explicit version newtypes; implement the fixture contract.
- [ ] Run `cargo fmt --all -- --check`, `cargo test -p pvz-data --locked`, and `cargo clippy -p pvz-data --all-targets --locked -- -D warnings`; complete all applicable submission checks.
- [ ] 提交 `feat(data): 添加版本化领域标识符`。

### Task 2: Register the pure simulation package

**Files:**
- Create: `crates/pvz-core/Cargo.toml`
- Create: `crates/pvz-core/src/lib.rs`
- Create: `crates/pvz-core/src/clock.rs`
- Create: `crates/pvz-core/src/commands.rs`
- Create: `crates/pvz-core/src/events.rs`
- Create: `crates/pvz-core/src/rng.rs`
- Create: `crates/pvz-core/src/world.rs`
- Create: `crates/pvz-core/tests/determinism.rs`
- Create: `crates/pvz-core/benches/stepping.rs`
- Modify: `Cargo.toml`
- Modify: `.github/workflows/ci.yml`
- Modify: `docs/behavior/README.md`
- Modify: `docs/behavior/reference-baseline.md`
- Create: `docs/adr/0003-simulation-clock.md`

**Interfaces:**
- `GameState::new(seed, initial_state) -> GameState`.
- `GameState::step(commands: &[Command]) -> StepEvents`.
- `GameState::state_digest() -> StateDigest`.
- `SimulationClock` stores an explicit, replaceable tick contract; development parameters remain labeled until reference measurement confirms the original timebase.

- [ ] Define the replaceable clock contract; record measured reference evidence or the still-pending timebase, and label any development parameter without claiming original behavior.
- [ ] Register the package and test entry; add deterministic replay tests with two instances receiving identical commands and assert equal digests/events.
- [ ] Add explicit expected tick/state transitions, RNG vectors and command/event ordering assertions; perturb an input or initial state that must affect the result to catch constant digests or ignored commands. Two-instance equality alone is insufficient.
- [ ] Run `cargo test -p pvz-core --test determinism --locked`; verify failure identifies the intended simulation contract gap.
- [ ] Implement empty-board stepping, command ordering, deterministic RNG wrapper, and event sequencing without plants/zombies yet.
- [ ] Add repeatable stepping benchmarks and applicable coverage reporting; record the first baseline and complete submission checks.
- [ ] Run core tests and `cargo clippy -p pvz-core --all-targets --locked -- -D warnings`; inspect dependency tree to confirm no engine/platform dependency.
- [ ] 提交 `feat(core): 添加确定性模拟基础`。

### Task 3: Build the resource audit library

**Files:**
- Create: `crates/pvz-assets/Cargo.toml`
- Create: `crates/pvz-assets/src/lib.rs`
- Create: `crates/pvz-assets/src/manifest.rs`
- Create: `crates/pvz-assets/src/diagnostics.rs`
- Create: `crates/pvz-assets/src/reanim.rs`
- Create: `crates/pvz-assets/src/particles.rs`
- Create: `crates/pvz-assets/tests/fixtures.rs`
- Create: `crates/pvz-assets/benches/parsing.rs`
- Create: `tests/fixtures/assets-minimal/README.md`
- Modify: `Cargo.toml`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- `AssetRoot::open(path: &Path) -> Result<AssetIndex, AssetError>`.
- `AssetIndex::audit() -> AuditReport`.
- `parse_reanim(text: &str) -> Result<AnimationClip, AssetError>`.
- `parse_particle(text: &str) -> Result<ParticleSystem, AssetError>`.
- No API accepts or embeds a fixed user absolute path.

- [ ] Create original tiny XML/PNG metadata fixtures that contain no extracted game content; resource integration tests may additionally read the versioned asset manifest.
- [ ] When adding a new tracked binary fixture or reference asset, update the manifest and CI classification with its provenance and purpose; keep original executables and unregistered temporary payloads forbidden without weakening text-format checks.
- [ ] Add tests for valid manifest references, missing files, path case collisions, malformed XML, and deterministic report ordering.
- [ ] Run asset tests and verify failure before parsers exist.
- [ ] Implement supported subsets with path-aware diagnostics; reject unknown content affecting interpretation, and report any extension confirmed safe to preserve or skip.
- [ ] Keep directory traversal/file reads in a host adapter; manifest, Reanim and particle parsing accept bytes/text so app/browser resource providers can reuse them without filesystem paths.
- [ ] Add parser time benchmarks and coverage for success/error paths; measure memory/allocation when affected and record the baseline.
- [ ] Run `cargo test -p pvz-assets --locked`, Clippy, and a read-only audit against the versioned manifest and assets; complete submission checks without writing into the source directory.
- [ ] 提交 `feat(assets): 添加资源审计模型与解析器`。

### Task 4: Add the repository asset audit CLI

**Files:**
- Create: `tools/pvz-asset-tool/Cargo.toml`
- Create: `tools/pvz-asset-tool/src/main.rs`
- Create: `tools/pvz-asset-tool/tests/cli.rs`
- Modify: `Cargo.toml`
- Modify: `assets/README.md`
- Modify: `tools/pvz-asset-tool/README.md`

**Interfaces:**
- `pvz-asset-tool audit --format text|json` from the repository root.
- `pvz-asset-tool inspect-reanim <NAME>` from the repository root.
- Commands read `assets/manifest.json` and registered inputs; missing or invalid inputs fail explicitly. Tests use isolated fixture repositories, not a runtime resource-override configuration.

- [ ] Add CLI tests for help, missing/invalid manifest, hash mismatch, successful fixture audit, JSON ordering, and absolute-path redaction.
- [ ] Run CLI tests and verify failure before the commands exist.
- [ ] Implement argument parsing and report serialization using the assets library.
- [ ] Run the CLI against repository resources read-only; confirm 3229 manifest records cover 3228 extracted files and the one source package without emitting or modifying source files.
- [ ] 提交 `feat(tool): 添加仓库资源审计命令`。

### Task 5: Add the Macroquad probe application

**Files:**
- Create: `crates/pvz-app/Cargo.toml`
- Create: `crates/pvz-app/src/main.rs`
- Create: `crates/pvz-app/src/input.rs`
- Create: `crates/pvz-app/src/viewport.rs`
- Create: `crates/pvz-app/tests/viewport.rs`
- Modify: `Cargo.toml`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- `Viewport::fit(window_size, logical_size) -> Viewport`.
- `Viewport::to_logical(physical_point) -> Option<LogicalPoint>`.
- `InputEvent` variants: `PointerDown`, `PointerUp`, `PointerMove`, `PointerCancel`, `Pause`, `Resume`.

- [ ] Add pure viewport tests for 4:3, 16:9, high-DPI and out-of-viewport input.
- [ ] Run tests and verify failure before the transform exists.
- [ ] Implement a Macroquad window with placeholder background and a visible logical-grid diagnostic; keep rendering code separate from core.
- [ ] Load a minimal original image/animation set registered in the repository manifest through the platform provider; verify missing-resource diagnostics and path handling separately from the placeholder probe.
- [ ] Probe Reanim's required affine transforms with self-authored fixtures, including independent skew axes, layer ordering and clipping; confirm the backend route before expanding the animation player.
- [ ] Probe sound playback and a music decode/loop/pause route, including MO3 compatibility and browser gesture requirements; assess dependency/license compatibility for all six targets and record unresolved platform evidence.
- [ ] Run `cargo check -p pvz-app --locked`, desktop launch smoke tests and an app-only wasm build; actually launch the placeholder probe in a browser and verify input and high-DPI mapping. Record Windows, macOS, Linux and Web build/runtime evidence separately; unavailable environments remain pending and P2 cannot be marked complete on compilation alone.
- [ ] 提交 `feat(app): 添加跨平台视口验证应用`。

### Task 6: Implement the first playable vertical slice

**Files:**
- Modify: `crates/pvz-core/src/world.rs`
- Create: `crates/pvz-core/src/plants.rs`
- Create: `crates/pvz-core/src/zombies.rs`
- Create: `crates/pvz-core/src/projectiles.rs`
- Create: `crates/pvz-core/src/board.rs`
- Create: `crates/pvz-core/tests/vertical_slice.rs`
- Modify: `crates/pvz-app/src/main.rs`
- Modify: `docs/behavior/README.md`

**Interfaces:**
- Core commands cover seed selection, planting, sun collection, shovel, pause and restart.
- Core events cover planting, sun spawn/collect, projectile launch/hit, damage, zombie death, mower activation, win and loss.
- App consumes events for visuals/audio; it cannot apply damage directly.

- [ ] Measure the reference simulation timebase, timing, collision and event-order facts needed by the slice; replace development parameters and record evidence before original-behavior acceptance.
- [ ] Record controlled, repeated reference cases and expected results in the behavior documents before treating values as original rules; compare the playable slice against those cases and separate reference agreement from internal replay determinism.
- [ ] Add tests for 5×9 placement, sun cost, pea hit, zombie eating, mower activation, fixed wave, win/loss, pause and restart.
- [ ] Run vertical-slice tests and verify failure before entities exist.
- [ ] Implement only the minimum data-driven Sunflower, Peashooter, normal Zombie, Pea, Sun, LawnMower and one fixed wave.
- [ ] Extend coverage and hot-path benchmarks to the slice; run core tests and isolated no-resource app contracts, then a manual Macroquad smoke test using registered original assets. Confirm slice visuals/audio consume simulation events and missing assets fail explicitly; placeholders are not original-presentation acceptance.
- [ ] 提交 `feat(core): 添加白天关卡的最小可玩切片`。

### Task 7: Review slice evidence before content expansion

**Files:**
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`
- Modify: `.github/workflows/ci.yml`
- Modify: `docs/adr/0003-simulation-clock.md`
- Create: `docs/behavior/vertical-slice-results.md`

- [ ] Summarize the clock contract from Task 2, reference measurements required by Task 6, input mapping and slice comparisons; record limitations and remaining behavior gaps.
- [ ] Review the CI enabled by package registration, including dependency audit and applicable desktop/Web checks; fill real gaps without postponing earlier task requirements.
- [ ] Confirm `git ls-files` contains only manifest-registered reference assets and no unregistered packs, original executables, absolute user paths, or secrets.
- [ ] Review all public names and README wording; retain “unofficial” and MIT scope statements.
- [ ] 提交 `docs: 记录可玩切片的验收结果`。

### Task 8: Review coverage and performance baselines

**Files:**
- Modify: `CONTRIBUTING.md`
- Modify: `.github/workflows/ci.yml`
- Modify: `docs/performance/README.md`
- Modify: `docs/testing/README.md`

**Interfaces:**
- Coverage command: `cargo llvm-cov --workspace --locked --summary-only`; use the effective feature matrix from CONTRIBUTING.md.
- Benchmark command: `cargo bench --workspace --locked`.
- Criterion benchmarks live under the package that owns the measured hot path; benchmark names identify input scale and rules version.

- [ ] Review coverage reports collected during implementation, fill missing critical/error paths and document justified thresholds and remaining gaps.
- [ ] Review deterministic, resource-independent benchmark inputs; correctness tests remain parallel, while performance measurements avoid competing tests or tasks.
- [ ] Verify the existing text/resource checks still cover the new tracked file classes, manifest hashes and public fixture provenance.
- [ ] Review file/module/API/dependency consumers and effective target/feature combinations; remove obsolete placeholders and unused structures, checking callers, docs, tests and generation/configuration entry points before removal.
- [ ] Summarize timing, applicable memory/allocation baselines and coverage limitations; compare any optimization with its correctness evidence.
- [ ] 提交 `test: 完善覆盖率与性能验收基线`。

# Foundation and Asset Audit Implementation Plan

> 当前为文档与结构阶段。用户明确授权实现后再逐项执行，每项任务独立验收；具体代理工具由执行环境与适用指令决定。

**Goal:** 建立可公开的 Rust workspace 基线，并实现首个不携带原版内容的资源审计与解析基础，为 PvZ 核心垂直切片提供稳定边界。

**Architecture:** 按任务依次注册 `pvz-data`、`pvz-core`、`pvz-assets`、`pvz-asset-tool` 和 `pvz-app`，每次只交付一个可验证边界。core 只依赖数据/纯逻辑；assets 解析文件为 CPU 模型；app 以 Macroquad 装配表现；CLI 只读取用户指定目录。

**Tech Stack:** Rust 1.99.0、Edition 2024、Cargo resolver 3、Macroquad、Serde/JSON 或 RON（选定时记录）、quick-xml 或等价 XML 解析库（选定时记录）、Cargo test、rustfmt、Clippy、cargo-deny、GitHub Actions。

**Spec:** `docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md`

## 实施前提与公共验收

长期约束遵循 [项目原则](../../principles/README.md)，架构遵循 [模块合同](../../architecture/README.md)，每次提交完成 [贡献指南的适用验收](../../../CONTRIBUTING.md#验证流程)。下列任务命令是局部反馈，不替代提交门槛；新增 package 同步锁文件、lint 与 CI，覆盖率和热路径基准随相关实现接入。

- 当前参考基线 ID 为 `goty-zh-a7ab7df8-7f1e824e`，但版本字段差异必须保留并可追溯。
- 原版模拟 tick 未测量；不能把 60 Hz 作为已知规格。
- 逻辑视口暂按 800×600；1400×600 背景的裁剪/滚动待测量。
- 已明确行为优先测试先行，探索探针先获取证据，正式交付前补齐适用测试。测试先明确预期合同，不把未注册 package 或环境错误当成行为证据。

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
- Create: `docs/adr/0002-simulation-clock.md`

**Interfaces:**
- `GameState::new(seed, initial_state) -> GameState`.
- `GameState::step(commands: &[Command]) -> StepEvents`.
- `GameState::state_digest() -> StateDigest`.
- `SimulationClock` stores an explicit, replaceable tick contract; development parameters remain labeled until reference measurement confirms the original timebase.

- [ ] Define the replaceable clock contract; record measured reference evidence or the still-pending timebase, and label any development parameter without claiming original behavior.
- [ ] Register the package and test entry; add deterministic replay tests with two instances receiving identical commands and assert equal digests/events.
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

- [ ] Create original tiny XML/PNG metadata fixtures that contain no extracted game content.
- [ ] Add tests for valid manifest references, missing files, path case collisions, malformed XML, and deterministic report ordering.
- [ ] Run asset tests and verify failure before parsers exist.
- [ ] Implement supported subsets with path-aware diagnostics; reject unknown content affecting interpretation, and report any extension confirmed safe to preserve or skip.
- [ ] Add parser time benchmarks and coverage for success/error paths; measure memory/allocation when affected and record the baseline.
- [ ] Run `cargo test -p pvz-assets --locked`, Clippy, and a read-only audit against the user-specified input; complete submission checks without writing into the source directory.
- [ ] 提交 `feat(assets): 添加资源审计模型与解析器`。

### Task 4: Add the local asset audit CLI

**Files:**
- Create: `tools/pvz-asset-tool/Cargo.toml`
- Create: `tools/pvz-asset-tool/src/main.rs`
- Create: `tools/pvz-asset-tool/tests/cli.rs`
- Modify: `Cargo.toml`
- Modify: `config/local.example.toml`
- Modify: `assets-private/README.md`

**Interfaces:**
- `pvz-asset-tool audit --root <PATH> --format text|json`.
- `pvz-asset-tool inspect-reanim --root <PATH> <NAME>`.
- Input precedence for future app: CLI flag > `PVZ_ASSET_ROOT` > local config > placeholder; invalid explicit original path is an error.

- [ ] Add CLI tests for help, missing root, successful fixture audit, JSON ordering, and absolute-path redaction.
- [ ] Run CLI tests and verify failure before the commands exist.
- [ ] Implement argument parsing and report serialization using the assets library.
- [ ] Run the CLI against user resources read-only; compare its file count with the recorded 3228-file baseline without emitting source files.
- [ ] 提交 `feat(tool): 添加本地资源审计命令`。

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
- [ ] Run `cargo check -p pvz-app --locked`, desktop launch smoke test, and a wasm compile check when the target/toolchain is available; record build and runtime evidence separately.
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
- [ ] Add tests for 5×9 placement, sun cost, pea hit, zombie eating, mower activation, fixed wave, win/loss, pause and restart.
- [ ] Run vertical-slice tests and verify failure before entities exist.
- [ ] Implement only the minimum data-driven Sunflower, Peashooter, normal Zombie, Pea, Sun, LawnMower and one fixed wave.
- [ ] Extend coverage and hot-path benchmarks to the slice; run core tests, a no-resource app test, and a manual Macroquad smoke test with explicitly selected placeholders.
- [ ] 提交 `feat(core): 添加白天关卡的最小可玩切片`。

### Task 7: Review slice evidence before content expansion

**Files:**
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`
- Modify: `.github/workflows/ci.yml`
- Modify: `docs/adr/0002-simulation-clock.md`
- Create: `docs/behavior/vertical-slice-results.md`

- [ ] Summarize the clock contract from Task 2, reference measurements required by Task 6, input mapping and slice comparisons; record limitations and remaining behavior gaps.
- [ ] Review the CI enabled by package registration, including dependency audit and applicable desktop/Web checks; fill real gaps without postponing earlier task requirements.
- [ ] Confirm `git ls-files` contains no original resources, generated packs, absolute user paths, or secrets.
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
- [ ] Verify the existing text/resource checks still cover the new tracked file classes and public fixture provenance.
- [ ] Summarize timing, applicable memory/allocation baselines and coverage limitations; compare any optimization with its correctness evidence.
- [ ] 提交 `test: 完善覆盖率与性能验收基线`。

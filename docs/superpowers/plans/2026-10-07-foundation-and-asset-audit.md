# Foundation and Asset Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task after the design is approved. Each task is independently testable and uses checkbox syntax for tracking.

**Goal:** 建立可公开的 Rust workspace 基线，并实现首个不携带原版内容的资源审计与解析基础，为 PvZ 核心垂直切片提供稳定边界。

**Architecture:** 先注册 `pvz-data`、`pvz-core`、`pvz-assets`、`pvz-app` 和 `pvz-asset-tool`，但每次只交付一个可验证边界。core 只依赖数据/纯逻辑；assets 解析文件为 CPU 模型；app 以 Macroquad 装配表现；CLI 只读取用户指定目录。

**Tech Stack:** Rust 1.99.0、Edition 2024、Cargo resolver 3、Macroquad、Serde/JSON 或 RON（选定时记录）、quick-xml 或等价 XML 解析库（选定时记录）、Cargo test、rustfmt、Clippy、cargo-deny、GitHub Actions。

**Spec:** `docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md`

## Global Constraints

- `pvz-core` 不依赖 Macroquad、窗口、设备、音频、文件系统或资产路径。
- 原版资源、原版文本、EXE、DLL、`main.pak` 和派生包不进入 Git、CI 或公开发行包。
- 当前参考基线 ID 为 `goty-zh-a7ab7df8-7f1e824e`，但版本字段差异必须保留并可追溯。
- 原版模拟 tick 未测量；不能把 60 Hz 作为已知规格。
- 逻辑视口暂按 800×600；1400×600 背景的裁剪/滚动待测量。
- 桌面与 Web 先行，Android/iOS 随后接入；平台通过构建和运行证据声明支持。
- 每个任务先添加有意义的测试，再实现最小行为，最后运行适用检查并聚焦提交。
- 测试默认并行；共享外部状态只在最小范围串行，并记录原因。
- 实现核心或资源热路径时必须提供 Criterion 基准；正确性测试先于性能优化。
- 提交前检查 UTF-8/LF/无 BOM、死代码、未使用项、无用途文件和依赖许可证。
- Git 提交遵循 Conventional Commits，使用 `type(scope): 中文说明`（scope 可省略）；提交说明和正文使用中文，type/scope 与技术名称保留标准写法。每次聚焦一个可审阅的变更，破坏性变更明确标记，具体约定见 CONTRIBUTING.md。

---

### Task 1: Register the data schema package

**Files:**
- Create: `crates/pvz-data/Cargo.toml`
- Create: `crates/pvz-data/src/lib.rs`
- Create: `crates/pvz-data/src/ids.rs`
- Create: `crates/pvz-data/src/version.rs`
- Create: `crates/pvz-data/tests/schema.rs`
- Modify: `Cargo.toml`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Produces `AssetId`, `PlantId`, `ZombieId`, `LevelId`, `RulesVersion`, `ReferenceBaselineId`.
- Produces versioned plain data types only; no file or engine APIs.

- [ ] Add a test that serializes/deserializes a small original fixture and rejects an unsupported schema version.
- [ ] Run `cargo test -p pvz-data`; verify the test fails before the types exist.
- [ ] Add the smallest typed IDs and explicit version newtypes; implement the fixture contract.
- [ ] Run `cargo fmt --all -- --check`, `cargo test -p pvz-data`, and `cargo clippy -p pvz-data --all-targets -- -D warnings`.
- [ ] Generate and commit `Cargo.lock` once the first package exists.
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
- Modify: `Cargo.toml`

**Interfaces:**
- `GameState::new(seed, initial_state) -> GameState`.
- `GameState::step(commands: &[Command]) -> StepEvents`.
- `GameState::state_digest() -> StateDigest`.
- `SimulationClock` stores the chosen measured tick contract; it must not assume 60 Hz without evidence.

- [ ] Add deterministic replay tests with two instances receiving identical commands and assert equal digests/events.
- [ ] Run `cargo test -p pvz-core --test determinism`; verify it fails before the simulation types exist.
- [ ] Implement empty-board stepping, command ordering, deterministic RNG wrapper, and event sequencing without plants/zombies yet.
- [ ] Run core tests and `cargo clippy -p pvz-core --all-targets -- -D warnings`; inspect dependency tree to confirm no engine/platform dependency.
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
- Create: `tests/fixtures/assets-minimal/README.md`
- Modify: `Cargo.toml`

**Interfaces:**
- `AssetRoot::open(path: &Path) -> Result<AssetIndex, AssetError>`.
- `AssetIndex::audit() -> AuditReport`.
- `parse_reanim(text: &str) -> Result<AnimationClip, AssetError>`.
- `parse_particle(text: &str) -> Result<ParticleSystem, AssetError>`.
- No API accepts or embeds a fixed user absolute path.

- [ ] Create original tiny XML/PNG metadata fixtures that contain no extracted game content.
- [ ] Add tests for valid manifest references, missing files, path case collisions, malformed XML, and deterministic report ordering.
- [ ] Run asset tests and verify failure before parsers exist.
- [ ] Implement supported subsets with path-aware diagnostics and explicit unsupported-feature errors.
- [ ] Run `cargo test -p pvz-assets`, Clippy, and an audit against a copy of the user path without writing into it.
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
- [ ] Run `cargo check -p pvz-app`, desktop launch smoke test, and a wasm compile check when the target/toolchain is available.
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

- [ ] Add tests for 5×9 placement, sun cost, pea hit, zombie eating, mower activation, fixed wave, win/loss, pause and restart.
- [ ] Run vertical-slice tests and verify failure before entities exist.
- [ ] Implement only the minimum data-driven Sunflower, Peashooter, normal Zombie, Pea, Sun, LawnMower and one fixed wave.
- [ ] Run core tests, a no-resource app test, and a manual Macroquad smoke test with placeholders.
- [ ] 提交 `feat(core): 添加白天关卡的最小可玩切片`。

### Task 7: Add review gates before content expansion

**Files:**
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`
- Modify: `.github/workflows/ci.yml`
- Create: `docs/adr/0002-simulation-clock.md`
- Create: `docs/behavior/vertical-slice-results.md`

- [ ] Record measured simulation clock, input mapping, and first reference comparisons with evidence.
- [ ] Add CI for fmt, check, test, Clippy, docs, cargo-deny, and desktop/Web compile checks supported by runners.
- [ ] Confirm `git ls-files` contains no original resources, generated packs, absolute user paths, or secrets.
- [ ] Review all public names and README wording; retain “unofficial” and MIT scope statements.
- [ ] 提交 `docs: 记录可玩切片的验收结果`。

### Task 8: Add coverage and performance gates

**Files:**
- Modify: `CONTRIBUTING.md`
- Modify: `.github/workflows/ci.yml`
- Create: `docs/performance/README.md`
- Create: `docs/testing/README.md`

**Interfaces:**
- Coverage command: `cargo llvm-cov --workspace --all-features --summary-only`.
- Benchmark command: `cargo bench --workspace`.
- Criterion benchmarks live under the package that owns the measured hot path; benchmark names identify input scale and rules version.

- [ ] Define coverage reporting without imposing an arbitrary percentage before the first implementation reveals realistic module boundaries.
- [ ] Define benchmark fixtures that are deterministic, resource-independent and safe to run in parallel with ordinary tests.
- [ ] Add CI checks for text encoding and tracked binary/resource policy without embedding user paths or original assets.
- [ ] Record baseline benchmark results and coverage limitations before optimizing.
- [ ] 提交 `chore: 添加测试与性能质量门槛`。

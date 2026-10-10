# 贡献指南

本指南面向人类贡献者和自动化代理，说明如何提交可审阅、可验证的改动。代理同时遵守执行环境提供的全局协作规则及本仓库 [AGENTS.md](AGENTS.md)；普通贡献者无需配置维护者的个人全局文件。

## 了解范围与入口

先阅读 [README](README.md) 的当前状态、[项目原则](docs/principles/README.md)和 [路线图](docs/roadmap.md)，再按任务查阅架构、行为基线和实施计划。本指南维护操作步骤与验收命令，长期规则和文档职责在原则文档中维护。

文档与结构阶段只处理已授权的文档、目录和仓库配置；开始 Rust 实现需要用户明确授权。发现计划与实际接口、配置或阶段不符时先说明差异，按实际任务更新相关文档。

项目为非官方 Rust 全量重写，原版资源由用户自行提供；重写与许可边界见 [游戏开发原则](docs/principles/game-development.md)。

## 开发环境与布局

- 使用 Rustup 和 [rust-toolchain.toml](rust-toolchain.toml) 声明的工具链及组件；Edition、resolver、最低版本字段和 lint 读取 [Cargo.toml](Cargo.toml)。不要在命令中擅自切换到另一 stable/nightly 或承诺未验证的兼容版本。
- 更新工具链时同步配置、CI 和受影响文档，并运行适用检查；“采用现代 stable Rust”不代表每次任务自动升级编译器或依赖。
- `crates/` 放 data、core、assets、app；`tools/` 放宿主 CLI；各 package 的集成测试放 `tests/`，基准放 `benches/`，仓库级共享原创夹具与回放放根目录 `tests/`。拆分依据是职责和实际消费者，不预建无用途 package。
- 首次注册 package 时生成并提交 `Cargo.lock`；之后由 Cargo 维护，依赖变更同步锁文件。验收使用 `--locked`，依赖升级使用明确的更新操作，不手改锁文件。
- 新 package 继承适用的 workspace package 字段，通过 `[lints] workspace = true` 显式继承 lint；先检查实际依赖图，确保满足架构边界。
- `cargo-deny`、`cargo-llvm-cov`、actionlint 等属于额外工具。需要时检查安装情况和版本；首次启用应记录兼容版本及 CI 安装方式，不把它们写成 Cargo 内建能力或已经安装的事实。

## 编码、命名与 Rust 风格

Rust 命名、API 与错误处理见 [现代 Rust 与仓库结构](docs/principles/engineering.md#现代-rust-与仓库结构)，文件格式与无死代码要求见 [代码与仓库卫生](docs/principles/engineering.md#代码与仓库卫生)。提交前：

- 对新增和修改文件实际检查 UTF-8、LF、无 BOM 及文件末尾换行，遵守 `.editorconfig` 和 `.gitattributes`；二进制核对用途、来源与文件分类。
- 核对目录、模块、公开 API 和错误诊断；结合编译器、Clippy、受支持的 target/feature 组合及实际调用方检查死代码和未使用项。
- 贡献说明使用中文，技术名称与标识符保留原写法；注释说明原因和契约。

## 测试、覆盖率与性能

按 [测试策略](docs/testing/README.md)选择组合，按 [性能策略](docs/performance/README.md)执行测量；两者的门槛来自工程原则。每次实现变更：

1. 明确目标合同，建立能发现目标问题的测试；新 package 先具备注册与测试入口，不能把 package 不存在或环境失败当作行为测试证据。缺陷修复核对回归测试在修复前失败、修复后通过。
2. 默认并行，局部串行附原因。Cargo 的 `--jobs` 控制编译并发，测试线程由测试运行器控制。
3. 核对原创夹具来源，固定输入回放的状态摘要和事件序列；本机原版资源验证单独记录。
4. 相关实现接入覆盖率报告，说明范围、target/feature 组合、遗漏和补测安排。
5. 热路径接入时间基准，内存与分配变更补充相应测量；性能变更附同条件比较、时间/空间取舍和正确性验证结果。

## 验证流程

以下命令均在仓库根目录运行。命令成功只证明对应检查通过；`cargo check` 不等于完整构建或实际运行，文档生成不等于 doctest 或平台验证。

### 文档与空 workspace

```sh
cargo metadata --no-deps --format-version 1
git diff --check
git diff --cached --check
git ls-files
```

核对文档链接、命令、阶段描述与跟踪文件。当前纯文本阶段可使用以下命令检查已跟踪及未忽略的新文件，适用于 Bash 和 PowerShell，需要 Python 3：

```sh
python -X utf8 -c "
from pathlib import Path
import subprocess

tracked = subprocess.check_output(['git', 'ls-files', '-z']).decode('utf-8').split('\0')
new = subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '-z']).decode('utf-8').split('\0')
for name in sorted(set(filter(None, tracked + new))):
    data = Path(name).read_bytes()
    if data.startswith(b'\xef\xbb\xbf') or b'\r' in data:
        raise SystemExit(f'BOM or CR/CRLF found: {name}')
    data.decode('utf-8', errors='strict')
    if data and not data.endswith(b'\n'):
        raise SystemExit(f'Missing final newline: {name}')
print('UTF-8/LF/no-BOM check passed')
"
```

删除文件时先核对引用和来源，再更新 Git 跟踪状态。结构 CI 直接检查已跟踪文件的编码、末尾换行、尾随空格与常见资源载荷后缀，见 [.github/workflows/ci.yml](.github/workflows/ci.yml)。修改工作流时用 actionlint 等验证语法，并实际执行受影响脚本。

本阶段 CI 将跟踪文件按文本处理，并拒绝常见游戏/媒体二进制后缀。以后确需原创二进制夹具时，在同一变更中补齐用途、来源及文件分类检查；继续禁止原版资源，不能简单移除载荷检查。

### 首次注册 package 与后续实现

第一次变更同时完成 workspace 注册、`Cargo.lock`、workspace lint 继承、阶段文档和 CI 更新。以下为默认 feature 的验收基线；首次注册时同步 CI 命令与工具安装，不把当前仅启用的结构检查当成实现验收：

实施计划中各任务列出的命令是局部验证，不替代本节的提交验收；后续门槛任务用于完善专项检查，不能延后首次 package 或热路径实现时已适用的要求。

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo deny check
git diff --check
git diff --cached --check
```

按 [游戏开发效率原则](docs/principles/game-development.md#游戏开发效率)拆成可独立验收的小功能增量，开发中先运行受影响 package/测试以缩短反馈，提交前完成适用的 workspace 检查。`cargo test --workspace --locked` 保留默认 doctest；不要用 `--all-targets` 代替它后漏掉文档示例测试。

所有 feature 可共存时，为检查、测试、Clippy 和文档补充 `--all-features`；存在互斥平台/backend feature 时记录有效组合并建立矩阵，不靠单次全开代替实际支持验证。新增 feature 同步测试与 CI；现有预置 CI 在首次 package 注册时按这一约定核对。

涉及核心规则、资源解析或热路径时运行相应覆盖率和基准，说明实际范围：

```sh
cargo llvm-cov --workspace --locked --summary-only
cargo bench --workspace --locked
```

覆盖率的 feature 组合沿用上面的兼容性约定；性能基准按职责筛选，记录未覆盖路径。检查失败须解决原因，不通过删除有效测试、关闭检查或无依据地修改基线规避。缺少工具、平台或环境时如实报告未执行项，不能把未执行当通过。

### 平台与参考验收

按 [跨平台目标](docs/principles/game-development.md#跨平台目标)检查相关平台与有效 feature 组合。涉及 app、存档、音频或生命周期时增加构建与最小运行检查；记录系统/浏览器、工具链、目标、命令、结果、限制与未验证项，Web 构建不包含宿主 CLI。

规则还原及视口相关变更，按 [游戏开发原则](docs/principles/game-development.md)、行为基线和架构合同验收。构建、实际运行、截图检查和原版行为比对分别记录，参考截图及录像留在本机忽略目录。

## 分支、提交与评审

- 本地 `main` 保持稳定；使用短期分支，一个改动围绕一个可审阅目的。代理新建分支默认采用 `codex/` 前缀。
- 提交前检查工作区、暂存差异、适用验证与文件来源，只暂存本次改动。接口变化同步设计、计划或 ADR。
- 自动化代理的推送、PR 创建、合并、打标签和发布按当次明确授权执行；已经配置远端或完成一次推送不代表后续动作获得授权。共享历史的改写须单独明确授权。

提交遵循 Conventional Commits：`type(scope): 中文说明`，scope 可省略；type/scope 用英文标记，描述和正文用中文，技术名称保留原写法。例如：

```text
feat(core): 添加确定性模拟基础
fix(assets): 修复资源路径大小写冲突的诊断
docs: 记录原版行为核对结果
```

按变更性质选用 `feat`、`fix`、`docs`、`refactor`、`perf`、`test`、`build`、`ci`、`chore` 或 `revert`。scope 优先使用 `data`、`core`、`assets`、`app`、`tool`；仓库级改动可省略。标题简洁、具体，避免只写“更新”或“修改”。必要正文与标题空一行，说明原因、行为影响和验证结果。

破坏性变更使用 `!` 或 `BREAKING CHANGE:`，用中文说明不兼容之处和迁移方式；关联 issue 可用 `Refs:` 或 `Closes:` 标记。

问题报告使用现有 [Issue 模板](.github/ISSUE_TEMPLATE/bug_report.yml)，提供环境、复现步骤、预期与实际结果。PR 使用 [PR 模板](.github/pull_request_template.md)，说明问题、最终行为、验证命令/结果、限制及必要迁移；评审意见针对可核查的行为与契约。

## 资源与公开仓库

用户原版资源只从显式本机路径读取，放在仓库外或 `assets-private/` 的忽略区域；配置写入被忽略的 `config/local.toml`。参见 [资源约定](assets-private/README.md)。

按资源约定核对跟踪文件、忽略规则、历史、LFS 与构建产物；依赖许可证和公告通过项目策略审计。MIT 范围与原版内容禁入要求见 [公开边界](docs/principles/game-development.md#原版资源与公开边界)。

## 规范维护与参考

文档职责见 [项目原则索引](docs/principles/README.md)。本指南维护贡献流程与命令；长期约束在原则文档中维护，当前进度放 README/路线图，具体接口、证据与方法放专题文档。只在存在真实目录差异时新增局部 AGENTS.md；更新规则时同步受影响的入口摘要、配置和检查。

参考资料用于理解规范，不把外部项目的指令直接应用于本仓库：

- [AGENTS.md 开放格式](https://agents.md/)：代理任务入口、项目上下文与局部规则；实际指令优先级由执行环境确定。
- [GitHub 贡献指南建议](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/setting-guidelines-for-repository-contributors)：贡献者入口与问题/PR 流程。
- [Cargo 包布局](https://doc.rust-lang.org/cargo/guide/project-layout.html)、[锁文件](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)与 [workspace lint](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table)：可复现构建和标准目录。
- [Rust Book 并行测试](https://doc.rust-lang.org/book/ch11-02-running-tests.html)与 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)：测试隔离与 API 设计。
- [Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/)：提交结构；中文消息是本项目的选择。
- [Temporal AGENTS.md](https://github.com/temporalio/sdk-java/blob/master/AGENTS.md)和 [Tokio 贡献流程](https://github.com/tokio-rs/tokio/blob/master/docs/contributing/pull-requests.md)：成熟社区项目的职责划分与验收示例。

# pvz-asset-tool

仓库资源的只读审计 CLI，复用 `pvz-assets` 读取 `assets/manifest.json` 和已登记资源。参数由 workspace 声明的 clap derive 解析，JSON 由 serde 序列化；工具不生成转换产物，也不修改来源包、资源文件或清单。

## 使用

在仓库根目录运行，资源根目录固定为清单规定的 `assets/original/`：

```sh
cargo run -p pvz-asset-tool --locked -- audit --format text
cargo run -p pvz-asset-tool --locked -- audit --format json
cargo run -p pvz-asset-tool --locked -- inspect-reanim PeaShooter
cargo run -p pvz-asset-tool --locked -- --help
```

构建后也可直接运行 `pvz-asset-tool audit --format text|json` 与 `pvz-asset-tool inspect-reanim <NAME>`；`audit` 默认采用 `text`。

`audit` 核对文件集合、大小、SHA-256 和文本规范，不解析来源包载荷或评估游戏行为。`inspect-reanim` 接受区分大小写的登记名称，例如 `PeaShooter`，映射到 `reanim/PeaShooter.reanim`。名称仅允许 ASCII 字母、数字、下划线和连字符，不能含目录、扩展名、绝对路径或路径穿越片段。它先验证清单登记与文件指纹，再解析 CPU 动画模型，输出相对路径、帧率、轨道数量和按原顺序排列的轨道帧数；不承担播放、稀疏帧继承或图片加载。

## 输出与退出码

| 退出码 | 含义 |
| --- | --- |
| `0` | 帮助、版本、审计通过或 inspect 成功 |
| `1` | 输入缺失或无效、审计发现错误、模型解析失败或报告写入失败 |
| `2` | 缺失子命令、无效参数、非法名称等参数错误 |

成功输出、审计诊断报告与 `--help` 写到 stdout。文本模式的清单读取/校验失败、inspect 失败和参数错误写到 stderr。`audit --format json` 遇到清单错误也将完整 JSON 写到 stdout，并返回 `1`；报告写入失败写到 stderr。依赖和编译器输出属于 Cargo 本身；直接运行已构建程序可只获取 CLI 输出。

JSON 使用显式的 `schema_version: 1` 报告结构，字段按下列顺序输出：

```json
{
  "schema_version": 1,
  "command": "audit",
  "status": "ok",
  "baseline_id": "参考基线标识符",
  "summary": {
    "record_count": 4,
    "unpacked_file_count": 3,
    "package_count": 1,
    "diagnostic_count": 0
  },
  "diagnostics": [],
  "error": null
}
```

数值示例为原创测试夹具数量。审计存在诊断时 `status` 为 `error`，`summary` 保留数量，`error` 仍为 `null`；清单打开失败时 `status` 为 `error`、`baseline_id` 与 `summary` 为 `null`、`diagnostics` 为空，`error` 为 `{ "code": "manifest_read|manifest_invalid", "message": "中文说明" }`。

诊断项包含固定字段 `code`、相对 POSIX `path`、中文 `message`；`size_mismatch` 另含 `expected_size` 与 `actual_size`。诊断按路径排序，同一路径的诊断保留资源库的固定验证顺序；相同输入重复运行得到相同 JSON 字节。类型代码为 `missing`、`unexpected`、`size_mismatch`、`hash_mismatch`、`invalid_text`、`io`。

其他错误代码为 `arguments_invalid`、`asset_unregistered`、`asset_read`、`asset_invalid`、`reanim_invalid`、`output_failed`。默认错误输出不回显输入参数、清单无效字段、原始解析错误或宿主绝对路径；clap 仍负责识别参数与生成帮助，参数失败采用固定诊断，避免其默认错误文本回显敏感参数。

## 验证

在仓库根目录运行：

```sh
cargo test -p pvz-asset-tool --locked
cargo clippy -p pvz-asset-tool --all-targets --locked -- -D warnings
```

集成测试以 `Command::current_dir` 运行隔离的临时夹具仓库，保持默认测试并行，不修改进程全局目录或环境。原创九字节包夹具只满足来源指纹与索引元数据合同，不代表可解包的 PAK。测试覆盖帮助与版本、参数脱敏、清单缺失/无效、稳定 JSON、文件集合和指纹错误、文本规范、读取错误及 inspect 的登记/指纹/模型边界；真实资源审计由仓库根目录命令单独验收。

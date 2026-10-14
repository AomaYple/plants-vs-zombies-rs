# 向日葵 idle（S2-A）实施计划

> **面向代理执行者：** 使用 `superpowers:subagent-driven-development` 或 `superpowers:executing-plans` 按任务实施；是否委派由当次授权和文件归属决定。所有步骤使用 checkbox 跟踪。S1 交付门槛已满足，本计划源码与本地可玩验收完成；独立复核、文档及 Git/对应 CI 正在收口。

**目标：** 在已有可玩白天单关中让向日葵使用动态 idle，保留暂停、双终局冻结、重开与再种植流程，独立交付一个可玩增量。

**架构：** 资源装配时一次解析并缓存向日葵 `PreparedAnimation`、`anim_idle` 区段和可绘轨道；`Scene` 借用离散采样帧，复用当前图像、仿射、裁剪和网格池。入口只把 `Runtime.animation_seconds` 传给表现层，核心不读取动画、不反向依赖表现。

**技术栈：** 仓库锁定的 Rust 1.99.0、Bevy 0.20、`pvz-assets` Reanim 解析、Criterion、cargo-llvm-cov；不新增依赖、package、feature 或 CLI 开关。

**设计依据：** [总体设计](../specs/2026-10-07-repository-and-runtime-design.md)、[总计划 S2](2026-10-13-minimum-playable-vertical-slices.md#s2有动态反馈的白天单关)、[项目原则](../../principles/README.md)。

## 全局约束与进入条件

- **状态：2026-10-14，S2-A 独立可玩交付完成。** 起点 `35bfd8c`；默认/all-features 各 210 项测试、两组合干净覆盖率、七个 app 基准、固定相位 GPU 截图、Windows/Web 正常双终局/冻结/重开再植均已完成。最终独立复核通过，目的提交 `3e6634e` 已推送 `main`，对应八项 CI 全部成功，总计划 S2-A 已可勾选；整个 S2 和原版表现还原未完成。详细范围及失败记录见[本轮结果](../../behavior/content-foundation-results.md#2026-10-14-向日葵动态待机增量)。
- 原版程序只作外部参考，不作运行依赖。登记原版资源是运行输入，通过已有清单加载；公开测试和基准夹具原创且注明用途。截图、日志、录像和报告放在忽略的 `artifacts/validation/s2-a-2026-10-14/`，不提交到 Git 或发行包。
- 项目自写正文、注释、诊断与显示文字使用中文；文本 UTF-8、LF、无 BOM、末尾换行；保留标识符与原版资源 ID。
- 正确性第一、速度第二、内存第三。测试默认并行；Cargo 调用、独占基准、真实窗口/Web 服务由主线程协调，避免锁争用和测量重叠。
- `pvz-core` 不依赖引擎、资产、I/O、输入、音频或墙钟；保持既有规则 2.0、摘要、命令/事件顺序和回放结果。不用动画控制产阳光、伤害、死亡或胜负。
- 本次只替换向日葵静态姿态。射手、僵尸、割草机、豌豆、阳光保持现有消费者；不扩展全播放器、插值、混合、粒子、音乐、独立实体相位或 Zombie 动作。
- 接口/依赖评估 Windows、macOS、Linux、Web、Android、iOS；本增量实际运行验收针对已支持的 Windows 和 Web，其余平台不写成已验。

## 事实、开发合同与接口起点

2026-10-14 对登记的 `assets/original/reanim/SunFlower.reanim` 进行了只读 XML 核查：`fps=12`，29 条轨道，`anim_idle` 首个非负 `f` 连续区段为 Rust 半开区间 `4..29`，长度 25。这是资源事实；开发实现采用现有离散采样、稀疏继承和图层顺序合同，不代表已测量原版循环边界、插值、混合或每株初始相位。

| 当前文件 | 已存在接口与实际消费者 | 本次变化 |
| --- | --- | --- |
| `crates/pvz-app/src/animation.rs` | `PreparedAnimation::new(&AnimationClip)`、`marker_segment(&str) -> Option<Range<usize>>`、`drawable_tracks(&Range<usize>) -> Vec<usize>`、`frame_at(f64, &Range<usize>) -> usize`；公开 `tracks/names/fps` | 复用，优先只补测试，不预建播放器 |
| `crates/pvz-app/src/resources.rs` | `POSE_SOURCES` 的向日葵目前取帧 4；`LoadedResources` 保存静态 `poses`、图像与现有射手 `animation`；`resolve_images` 严格解析完整 clip 的图像映射 | 单独缓存向日葵所选 idle，移除其静态姿态入口；保留其他静态来源 |
| `crates/pvz-app/src/render.rs` | 二进制私有 `mod render`；`Scene::upload(&mut World, LoadedResources) -> Self`；`Scene::update(&mut self, &mut World, &GameState)`；`TrackMesh` 池复用网格/材质 | 新增向日葵缓存字段，`update` 增加 `animation_seconds: f64`；在此文件内验证真实 Scene |
| `crates/pvz-app/src/main.rs` | `update` 先 `Runtime::advance`，后 `scene.update(world, &state.runtime.game)`；已有 smoke 截图等待消费者 | 同一位置传 `state.runtime.animation_seconds`；仅修正既有冒烟截图等待期间的冻结，保留正常入口自动接入 |
| `crates/pvz-app/src/runtime.rs` | `advance(f64)`，`animation_seconds`；暂停/终局冻结、重开清零已存在 | 作为既有合同使用，不另写时钟或改变推进顺序 |
| `crates/pvz-app/tests/presentation.rs`、`tests/day_content.rs` | 已有稀疏继承、图层筛选、时基、双终局和重开合同 | 增加本增量可捕获接线错误的边界与回放验证 |
| `crates/pvz-app/benches/presentation.rs` | 原创 8 轨道/128 帧索引、仿射裁剪、内容装配基准 | 补所选 idle 采样及 1/45 株构图工作量 |

下表保留实施前的接口起点；`SunflowerIdle` 现已实现，其定义、构造及资源/Scene/测试/基准消费者同一目的交付。`render` 保持二进制私有，不为集成测试公开整个模块。

### 播放和失败合同

| 输入或状态 | 明确结果 |
| --- | --- |
| 资源装配 | `PreparedAnimation::new` 一次；`marker_segment("anim_idle")` 一次；`drawable_tracks` 一次；图像仍通过现有映射、解码、上传路径一次装配 |
| 首帧与循环 | 时间 0 取源帧 4；`1/12` 取 5；`24/12` 取 28；`25/12` 回到 4；区段终点不播放 |
| 新种植 | 采用当前全局 `Runtime.animation_seconds` 相位，不把种植时间写入 core，也不承诺每株从首帧起播；重开后的时间 0 采样是首帧，正常种植后的首个渲染按当时表现时钟取样 |
| 负时间、NaN、正负无穷 | 复用 `frame_at` 返回区段首帧；超大有限时间也须返回区段内索引，不 panic；不是原版异常时间语义 |
| 缺 marker、marker 全负或无帧、无可绘轨道 | 资源装配返回带 `reanim/SunFlower.reanim` 上下文的中文 `ResourceError`；不静默回退静态向日葵。现有 `marker_segment` 不会产出空的有效段，不新增不可达检查；空 Range 的低层 `frame_at` 边界单独测试 |
| 未知非空图像 ID、非唯一映射、映射文件未登记、损坏图像 | 仍由完整 clip 的 `resolve_images`/清单/解码路径严格失败；错误归属向日葵路径，不沿用射手路径误报；不猜文件名，不增加占位回退 |
| 稀疏字段 | 使用 `PreparedAnimation::new` 的已有字段继承；缺省 `i` 继承，显式空 `i` 清空。`f < 0` 隐藏；`f` 缺省延续既有可见性规则 |
| 所选轨道在该帧缺帧或图像为空、隐藏 | 跳过这一帧的这一轨道，不复制末帧，不索引越界；短轨道不延长。所有轨道本帧都隐藏时不画该植物，并隐藏旧网格池尾部，禁止残影 |
| 源图层顺序 | `drawable_tracks` 保留源索引；深度用源索引计算，不因某层隐藏而改变其余层的相对次序 |
| alpha、仿射、裁剪 | 沿用现有 `a.unwrap_or(1.0).clamp(0.0, 1.0)`、`Affine::from_frame`、`clipped_quad`；完全越界不提交网格 |
| 暂停/恢复 | 动画仅消费 Runtime 冻结后的时间；暂停前后采样/构图相同；恢复保留相位，不补停顿时间；恢复命令当帧仍遵守现有 Runtime 行为 |
| 胜利/失败与重开 | 两种终局都冻结采样/构图；重开清零表现时钟并移除旧实体图层；再次种植按新局表现时间采样，无旧局图像残留 |

## 任务 1：缓存所选向日葵 idle 并验证资源边界

**文件：** 修改 `crates/pvz-app/src/resources.rs`、`crates/pvz-app/tests/presentation.rs`；必要低层边界测试仍放既有 `presentation.rs`。不修改解析器或清单，除非先拿到真实解析缺口并重新审查范围。

**接口：** 消费现有 `PreparedAnimation` 三个方法；产出以下最小接口，由资源装配、二进制 Scene、原创测试与基准实际消费，字段保持私有：

```rust
pub struct SunflowerIdle {
    animation: PreparedAnimation,
    segment: std::ops::Range<usize>,
    drawable_tracks: Vec<usize>,
}
impl SunflowerIdle {
    pub fn new(animation: PreparedAnimation) -> Result<Self, ResourceError>;
    pub fn sample(
        &self,
        seconds: f64,
    ) -> impl Iterator<Item = (usize, &pvz_assets::AnimationFrame)>;
}
// LoadedResources 拟新增字段：
// pub sunflower_idle: SunflowerIdle
```

- [x] 在 `tests/presentation.rs` 写原创 XML：marker 为 `1..4`；底层第 1 帧继承 `x=7`、第 2 帧改 `x=9`、第 3 帧显式空图；上层依次隐藏/可见/隐藏；再加只到帧 1 的短轨道。每项断言具体 `(源索引, 图像ID, x)`，覆盖跳过、恢复及源顺序，不能只断言结果非空。

```rust
// 原创夹具，仅验证本项目开发采样合同。
let source = "<fps>12</fps><track><name>anim_idle</name><t><f>-1</f></t><t><f>0</f></t><t/><t/><t><f>-1</f></t></track><track><name>底层</name><t><x>7</x><i>原创底图</i></t><t/><t><x>9</x></t><t><i></i></t></track><track><name>上层</name><t><f>-1</f><i>原创上图</i></t><t/><t><f>0</f></t><t><f>-1</f></t></track><track><name>短轨道</name><t><i>原创短图</i></t><t/></track>";
let idle = SunflowerIdle::new(PreparedAnimation::new(
    &pvz_assets::parse_reanim(source).unwrap(),
)).unwrap();
let snapshot = |seconds| idle.sample(seconds)
    .map(|(layer, frame)| (layer, frame.i.as_deref().unwrap(), frame.x))
    .collect::<Vec<_>>();
assert_eq!(snapshot(0.0), [(1, "原创底图", Some(7.0)), (3, "原创短图", None)]);
assert_eq!(snapshot(1.0 / 12.0), [(1, "原创底图", Some(9.0)), (2, "原创上图", None)]);
assert!(snapshot(2.0 / 12.0).is_empty());
assert_eq!(snapshot(3.0 / 12.0), snapshot(0.0));
```

- [x] 增加参数化时间边界：0、帧边界前后、循环端点、两个循环、-1、NaN、正负无穷、`f64::MAX`。对登记区段的首帧/末帧/循环另在资源模块测试断言 `4/28/4`；低层空段 `5..5` 和反向区段 `5..4` 只要求返回 5，不把其交给渲染。
- [x] 增加缺 marker、marker 无帧、marker 全负、仅 marker 无图片、全隐藏、全空图的构造失败测试。未知 ID 测试继续调用真正 `resolve_images`，不能仅测试采样器。记录每个失败预期的中文原因。
- [x] 由主线程按既有锁定工具链运行红测，确认新类型/合同尚未实现导致目标失败，不能把编译环境失败当红测：

```sh
cargo test -p pvz-app --test presentation --locked
```

- [x] 实现上述缓存和借用迭代器；核心逻辑遵循下面代码，构造错误使用现有私有 `error`：

```rust
pub fn new(animation: PreparedAnimation) -> Result<Self, ResourceError> {
    let segment = animation.marker_segment("anim_idle")
        .ok_or_else(|| error("reanim/SunFlower.reanim", "没有向日葵 idle marker 区段"))?;
    let drawable_tracks = animation.drawable_tracks(&segment);
    if drawable_tracks.is_empty() {
        return Err(error("reanim/SunFlower.reanim", "向日葵 idle 没有可绘制轨道"));
    }
    Ok(Self { animation, segment, drawable_tracks })
}
pub fn sample(&self, seconds: f64)
    -> impl Iterator<Item = (usize, &pvz_assets::AnimationFrame)> {
    let index = self.animation.frame_at(seconds, &self.segment);
    self.drawable_tracks.iter().filter_map(move |&layer| {
        let frame = self.animation.tracks[layer].get(index)?;
        (!frame.f.is_some_and(|n| n < 0.0)
            && frame.i.as_ref().is_some_and(|id| !id.is_empty()))
            .then_some((layer, frame))
    })
}
```

- [x] 从 `POSE_SOURCES` 移除向日葵静态条目，在 `LoadedResources::from_provider` 单独读取 `reanim/SunFlower.reanim`，通过原清单/解析/`resolve_images`/解码路径建立图像，再将 prepared clip 移入 `SunflowerIdle::new`。复用现有 `BTreeMap::Entry::Vacant` 图像去重；将映射失败转为向日葵资源上下文。不要解析两次、保留第二份静态向日葵姿态或给每株 clone clip。
- [x] 增加 `LoadedResources.sunflower_idle`；在登记资源测试核对 `fps=12`、轨道数 29、区段 `4..29`，静态 `poses` 为其余 5 类，完整图像/音效集合仍保持既有合同。这些断言在同模块中访问私有缓存即可，不增加只供测试的 getter。
- [x] 由主线程运行 `cargo test -p pvz-app --lib --test presentation --locked`；复核新 API 都有真实消费者，错误测试不复制生产逻辑。任务 2 接线完成前，本任务不单独宣称可玩交付。

## 任务 2：接入真实 Scene 并验证借用绘制

**文件：** 修改 `crates/pvz-app/src/render.rs`、`crates/pvz-app/src/main.rs`。`render.rs` 新增 `#[cfg(test)] mod tests`，不移动模块、不改变 `lib.rs` 可见性。

**接口：** `Scene::upload` 消费任务 1 的 `LoadedResources.sunflower_idle`；唯一修改的调用签名如下，所有调用点在同一增量更新：

```rust
pub fn update(
    &mut self,
    world: &mut World,
    game: &pvz_core::GameState,
    animation_seconds: f64,
);
// main.rs 原 Scene 更新位置：
scene.update(world, &state.runtime.game, state.runtime.animation_seconds);
```

- [x] 先在 `render.rs` 内添加真实 `Scene` 红测：`World::new()` 插入 `Assets<Image>`、`Assets<Mesh>`、`Assets<ColorMaterial>`；使用三张不同颜色且 Handle 各不相同的原创 2×2 RGBA `CpuImage`、任务 1 同类原创 XML 构造缓存。底图红色、上图蓝色、短图绿色，各自保留实际生成的 Handle 供材质断言；不能共用同一纹理而漏检池槽位换层时未更新 `material.texture`。在同文件测试中直接构造私有 `Scene` 字段/调用私有 `texture`，不制造无效 OGG 或伪造 `LoadedResources` 音频对象来凑入口。

```rust
let mut world = World::new();
world.insert_resource(Assets::<Image>::default());
world.insert_resource(Assets::<Mesh>::default());
world.insert_resource(Assets::<ColorMaterial>::default());
let bottom = texture(&mut world, CpuImage {
    width: 2, height: 2, rgba: [255, 0, 0, 255].repeat(4),
});
let upper = texture(&mut world, CpuImage {
    width: 2, height: 2, rgba: [0, 0, 255, 255].repeat(4),
});
let short = texture(&mut world, CpuImage {
    width: 2, height: 2, rgba: [0, 255, 0, 255].repeat(4),
});
assert_ne!(bottom, upper);
assert_ne!(bottom, short);
assert_ne!(upper, short);
// 保留三个实际 Handle，后续断言对应池槽位的材质绑定。
// Scene 拟新增 sunflower_idle 字段。
let images = [
    ("原创底图".to_owned(), (bottom.clone(), [2.0, 2.0])),
    ("原创上图".to_owned(), (upper.clone(), [2.0, 2.0])),
    ("原创短图".to_owned(), (short.clone(), [2.0, 2.0])),
].into_iter().collect();
```

- [x] 用 `Runtime::failure_day()` 正常命令在 row 1/column 0 种向日葵；为该 World 夹具提供合法的割草机等静态 `poses` 和图像（静态对象可复用底图 Handle，三个动画层仍分别绑定不同 Handle）。分别调用 `Scene::update` 于 0、`1/12`、`2/12`、`3/12`。读取实际 `Mesh::ATTRIBUTE_POSITION`、UV、材质纹理/alpha、`Transform.z`、`Visibility` 与池长度；断言底层位置确实变化、显式空图/短轨道消失、隐藏上层恢复、回环恢复、尾池隐藏。时间 0 的向日葵两个槽位分别绑定 `Some(bottom.clone())`、`Some(short.clone())`；保存第二个槽位的 entity/material Handle，时间 `1/12` 断言槽位和材质被复用而纹理改为 `Some(upper.clone())`，回环时再改回 `Some(short.clone())`。这些预期必须直接引用夹具实际生成的 Handle，能捕获未更新 `material.texture` 的错误。选择固定的植物逻辑坐标计算预期顶点，不把 `clipped_quad` 输出再次当作唯一预期。
- [x] 补部分越界、完全越界、alpha、图层稀疏显示、多植物、移除后网格隐藏与容量复用。每次更新后 `Assets<Image>` 数量/Handle 不变；池在达到最大所需图层数后网格/材质数量稳定，证明未逐帧上传纹理或累积资产；不把该数量断言称作堆分配测量。
- [x] 由主线程运行 `cargo test -p pvz-app --bin pvz-app render::tests --locked` 确认红测，然后把缓存移动进 `Scene`。沿现有 actor 列表/排序绘制：向日葵遍历 `self.sunflower_idle.sample(animation_seconds)`，其他类型继续借用 `self.poses[&kind]`；可用小范围私有绘制辅助函数合并原绘制体，但仅在 Scene 实际调用时引入。
- [x] 使用源轨道索引计算动态图层深度，保留 actor 行深度、原图像尺寸、逻辑坐标和裁剪；每帧借用帧与已有纹理 Handle，不 `clone` 完整帧表/clip。允许已有顶点/网格更新成本，不宣称零分配。
- [x] 在 `main.rs::update` 只增加表现时钟参数。既有暂停、F、R、资源错误 UI、冒烟流程继续复用；不新增 S2 开关或开发界面，不改变 `Runtime::advance` 的位置。
- [x] 在已有 main 无窗口 World 测试框架加一项：让 `AppState` 持有上述原创 Scene，调用真正应用 `update`，在暂停状态固定时间调用两次得到相同网格，恢复后时间推进并改变图层；这能捕获 main 错传 0 或使用模拟 tick 的接线错误。新增 `#[cfg(test)] pub(crate) fn sunflower_scene_fixture(world: &mut World) -> Scene`，放在 `render.rs` 测试作用域辅助区；其内部按本任务上述 World 图像/静态 poses/原创 idle 装配 Scene，由 render 单元测试和 main 测试共同消费，不增加生产公开 API。
- [x] 由主线程运行下面局部绿测；记录纯 CPU/Bevy World 验证范围，不声称 GPU、窗口或原版还原已验：

```sh
cargo test -p pvz-app --bin pvz-app --test presentation --locked
cargo clippy -p pvz-app --all-targets --locked -- -D warnings
```

## 任务 3：保护表现时钟、双终局和同版本核心回放

**文件：** 真实 Scene 快照与回放对照测试放在 `crates/pvz-app/src/render.rs`/`crates/pvz-app/src/main.rs` 的二进制内部测试，复用任务 2 的原创 Scene 夹具。`crates/pvz-app/tests/day_content.rs` 只承担纯 Runtime 集成合同，优先复用已有暂停、终局和重开测试，仅补安全行向日葵存活等实际覆盖缺口；不从集成测试访问 binary 私有 render。仅核对 `crates/pvz-app/src/runtime.rs`，无证据不修改 Runtime/core 或内容 JSON。

**接口：** 消费既有 `Runtime::failure_day/command/advance/take_events`、`game.state_digest()`、`board.status()` 和任务 2 的 Scene 更新。所有命令由现有 core 决定。

- [x] 在二进制内部测试写两条真实 Scene 完整测试，均在 row 1/column 0 种向日葵。失败场景 row 0 不种防守；成功场景再在 row 0/column 0 种射手，总初始消耗 150。用 `SelectCard`、`Plant` 命令，验证种植事件/余额；最多 1000 次 `advance(0.25)`，必须实际达到 `Lost` 或 `Won`，两个终局向日葵仍存活。纯 Runtime 的相同场景已有覆盖时直接复用，不在 `day_content.rs` 重复完整流程。

```rust
runtime.command(Command::SelectCard { kind: PlantKind::Sunflower });
runtime.command(Command::Plant { cell: Cell { row: 1, column: 0 } });
// 成功场景紧接着排队：
runtime.command(Command::SelectCard { kind: PlantKind::Peashooter });
runtime.command(Command::Plant { cell: Cell { row: 0, column: 0 } });
```

- [x] 在上述二进制内部测试中对每个终局保存摘要、事件序列、`animation_seconds`、所选帧和实际 Scene 网格快照；`advance(100.0)` 后再次 Scene 更新，必须全部冻结且没有重复 Victory/Defeat。随后 R 对应的 `Command::Restart`、`advance(1/60)`，断言 tick/time 为 0、旧向日葵已移除、旧池隐藏；再次种植向日葵并推进，图层恢复且按新局时钟变化。
- [x] 在二进制内部测试补暂停的真实 Scene 断言：种植后在非循环首帧 Pause，记录时间与实际 mesh；多次大 elapsed 都不变；Resume 首调用按现有 Runtime 只消费恢复命令，随后正 elapsed 开始推进，不累计停顿。测试无效 elapsed 对 Scene 输入不产生越界；纯 Runtime 时间合同复用既有 `day_content.rs` 测试，不修改 Runtime 来迎合新测试。
- [x] 在二进制内部测试写同规则/同 seed/同命令固定回放对照：两个 `Runtime::failure_day()` 以完全相同 elapsed/命令推进，一边每次执行 Scene 更新，另一边只推进 core；逐批比较摘要及 `take_events()`，含暂停/恢复、两种终局和重开后的再种植。Scene 可用不同采样时间额外更新，core 仍不变；不使用截图像素作为回放判据，也不扩大 render 的生产可见性。
- [x] 核对已有默认关卡、30/120 Hz、双终局与错误退出码回归仍在，不改既有预期或规则版本。由主线程运行：

```sh
cargo test -p pvz-app --test day_content --test input_runtime --bin pvz-app --locked
cargo test -p pvz-core --locked
```

## 任务 4：交付原创采样与多植物构图基准

**文件：** 修改 `crates/pvz-app/benches/presentation.rs`；沿用 Criterion，不新建只供未来使用的库模块或夹具文件。

**接口：** 消费任务 1 `SunflowerIdle::new/sample`、既有 `Affine::from_frame`、`clipped_quad`。与 Scene 使用相同离散采样和几何函数；明确计时不包含 Bevy 网格池/GPU 提交。

- [x] 在既有基准函数内构造原创 29 轨道、marker `4..29`、12fps 的输入：marker 和主体完整轨道至少 30 帧，包含可见层、隐藏层、空图；至少一条真实短轨道仅 6 帧（源索引 0..5），在源帧 4/5 带可见非空图像，源帧 6..28 实际不存在，不补齐或复制末帧。解析/prepare/选段/轨道筛选全在计时外；先断言短轨道在时间 0 被借用采样消费，而 `2/12` 后因真实缺帧被跳过。此规模只模拟代表性工作量，不拷贝原版轨道或把它当原版像素证据。
- [x] 添加 `表现/原创29轨道25帧_12fps向日葵所选idle采样`，消费全部采样帧并 `black_box` 帧/源索引，防止只测迭代器构造。
- [x] 添加 1 株和 45 株的 `表现/原创向日葵idle采样与多植物仿射裁剪`：固定 5×9 原点、2D 图像尺寸和 800×600 裁剪，逐株遍历借用采样帧，累计输出顶点数并 `black_box`；同一缓存供全部植物消费，不在计时内解析/上传/clone clip。下列主体直接使用生产几何路径：

```rust
let mut count = 0;
for origin in &origins {
    for (_, frame) in idle.sample(black_box(1.25)) {
        let mut affine = Affine::from_frame(frame);
        affine.tx += origin[0];
        affine.ty += origin[1];
        count += black_box(clipped_quad(
            affine, [40.0, 50.0],
            ClipRect { min: [0.0, 0.0], max: [800.0, 600.0] },
        )).len();
    }
}
black_box(count);
```

- [x] 主线程在无本项目构建、测试、窗口或浏览器竞争的条件下运行 `cargo bench -p pvz-app --bench presentation --locked`，保存本次命令、实际运行标签、均值、95% 区间、环境与计时范围；只引用本轮运行场景，不把旧 Criterion 缓存视为本轮重测。
- [x] 本增量缓存改变了 clip 保存/共享策略，补对应测量：记录单缓存的展开帧数、字符串字节总量、`Vec` 长度/容量及缓存数量；1/45 株实际 Scene 验证缓存和纹理不随植物复制。若使用分配分析器，记录工具/范围/预热；若只统计结构占用，明确不等于进程峰值、堆分配次数或 GPU 显存。没有同条件前后测量时不作内存或性能改善主张。

## 任务 5：完整工程门槛、固定时间证据和 Windows/Web 可玩交付

**文件与产物：** 实施后同步 `docs/behavior/content-foundation-results.md` 的新 S2-A 结果节及总计划状态；这些文件由主线程指定负责人更新，不在规划分支抢占。原始产物放 `artifacts/validation/s2-a-2026-10-14/`。本任务由主线程统一执行，不预先勾选任何结果。

- [x] S1 完成与本增量最终源码状态经主线程确认后，执行 [CONTRIBUTING](../../../CONTRIBUTING.md#rust依赖与构建配置变更) 的完整适用 L0/L1/L2/L3。最低命令如下，默认与 all-features 分别完成，doctest 保留：

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --all-features --no-deps --locked
cargo deny check
```

- [x] 按现有覆盖率隔离方法先清理当前 workspace 插桩缓存；默认/all-features 分别运行、导出并保存对应产物，核对只有当前 checkout 路径。命令中的 `--all-features` 只传给插桩测试，不传给 `report`：

```sh
cargo llvm-cov clean --workspace --locked
cargo llvm-cov --workspace --locked --no-report
cargo llvm-cov report --json --output-path artifacts/validation/s2-a-2026-10-14/coverage-default.json
cargo llvm-cov clean --workspace --locked
cargo llvm-cov --workspace --all-features --locked --no-report
cargo llvm-cov report --json --output-path artifacts/validation/s2-a-2026-10-14/coverage-all.json
```

- [x] 报告采样、资源失败和实际 Scene 接线的命中/未覆盖路径；新增关键分支缺口先补有意义的测试。原生纯测试不覆盖 GPU 像素、Web 条件代码或未运行平台；不靠加公开 API、排除源码或任意降低门槛获得覆盖率通过。
- [x] 先为固定时间截图修正已有冒烟消费者：在 `main.rs::update` 的 `poll_resources` 之后、输入读取与各脚本之前，若已有 smoke 截图请求且 `screenshot_wait` 存在，只调用现有 `smoke` 等待处理后返回。`screenshot_wait` 字段仅 native 编译，整个 guard 使用同一 `#[cfg(not(target_arch = "wasm32"))]` 边界，Web 不编译对该字段的访问；对应等待测试也放在 native 条件下。Bevy 继续泵渲染、截图 observer 和 `Time<Real>`；不再推进 Runtime、脚本、输入命令或 Scene，冻结 game/animation_seconds/render_frames。正常无 smoke 的入口不受影响，不新增 CLI、开发 UI 或隐藏规则注入。这项修正随 S2-A 实施交付，S1 已交付结果维持原有范围。

```rust
#[cfg(not(target_arch = "wasm32"))]
if state.options.smoke.is_some()
    && state.screenshot_requested
    && state.screenshot_wait.is_some()
{
    smoke(world, &mut state);
    return;
}
```

- [ ] 先用已有 main World harness 在 `#[cfg(not(target_arch = "wasm32"))]` 条件下红测截图等待：预设一个真实 `ScreenshotWait::new(Time<Real>::elapsed())`，标记已有请求，保存 Runtime 摘要、表现时间和帧数；多次应用 `update` 后三者相同，`Time<Real>` 仍可推进。再推进真实等待时钟到已有 30 秒边界，检查 `smoke` 返回错误 AppExit，不因 Runtime 冻结失去超时退出。保持现有失败400/期限2、slice、audio、无截图 smoke 及 screenshot 成功/失败回归；只冻结截图等待，不延长或隐藏超时失败。
- [x] 固定时间的确定性构图由任务 2 的实际 Scene World 在 0、`1/12`、`24/12`、`25/12` 生成并核对。GPU 截图复用现有 `--slice-smoke` 每帧 0.25 秒脚本及上述等待冻结，不新增 CLI/UI：主线程后台运行下列两个固定预算，保存各自截图避免后一次覆盖，核对实际向日葵完整绘制和不同相位。短预算图形未完成时选取更长且向日葵仍存活、两个相位不同的预算重测并记录原因。

```sh
cargo run -p pvz-app --locked -- --slice-smoke --smoke 4 --screenshot
cargo run -p pvz-app --locked -- --slice-smoke --smoke 5 --screenshot
```

  这两项分别在预算处对应 Runtime 累计 1.0/1.25 秒，须用真实推进日志/测试核对；在已有 GPU 截图日志中补记录冻结的 `animation_seconds` 和 `render_frames`。GPU 捕获虽晚于请求，等待中的 Runtime/Scene 保持固定，因此取得对应固定表现时间的真实 GPU 截图。仍须逐张核对资源完整与向日葵区域，不能把文件存在或字节不同直接当动画正确；正常入口的暂停截图另补实际玩法冻结证据。

- [x] 构建当前 Windows 程序与默认/probe Web 包，复用已有 Web 页面和部署资源方式；主线程先核对 `wasm-bindgen --version` 与锁定依赖兼容，运行：

```sh
cargo build -p pvz-app --locked
cargo build -p pvz-app --target wasm32-unknown-unknown --release --locked
wasm-bindgen --target web --out-name pvz_app --out-dir target/web target/wasm32-unknown-unknown/release/pvz-app.wasm
cargo build -p pvz-app --target wasm32-unknown-unknown --release --features probe --locked
wasm-bindgen --target web --out-name pvz_app --out-dir target/web-probe target/wasm32-unknown-unknown/release/pvz-app.wasm
```

- [x] Windows 正常入口后台启动 `target/debug/pvz-app.exe`，Web 使用本增量生成包和主线程管理的回环服务/隐藏浏览器；实际输入需要前台时说明最小必要范围。两平台各执行下面相同完整流程，保留窗口身份、源码/包身份、版本、输入及截图时间，不借用 S1 旧截图代替 S2：

| 流程 | 正常输入与观察 | 接受条件 |
| --- | --- | --- |
| 动态与暂停 | F 进入开发防守关卡；在安全 row 1/column 0 种向日葵；观察不同时间相位，再 P 暂停、间隔截图、P 恢复 | 向日葵完整、可见变化；暂停时同一植物相位/位置不变；恢复继续变化；其他静态消费者正常 |
| 失败 | row 0 不种防守，等待失败；向日葵留在 row 1 | 明确 Lost，存活向日葵仍可见；间隔截图确认其冻结；R 重开清空，再种植恢复动态 |
| 成功 | 再 F/R 确认同一 F 关卡新局；row 1 向日葵加 row 0 射手，总成本 150；恢复运行至胜利 | 明确 Won，向日葵仍存活可见；间隔截图确认冻结；R 重开后正常选卡/种植再次成功并恢复动态 |

- [x] 固定棋盘状态、植物位置、窗口尺寸、DPI 和暂停状态比较截图；变化证据比较同一向日葵区域，排除其他实体移动、UI 和捕获延迟。保存 Scene 固定时间快照与实际截图各自范围，不将 CPU 构图、GPU 实跑或资源值等同原版播放器验证。
- [x] 主线程核对错误退出码、资源失败和 S1 原有成功/失败冒烟回归；记录当前浏览器错误/警告与历史 GPU 诊断限制。关闭本次自有进程、标签、服务，保留证据，不干扰无关应用。
- [x] 完成独立复核、编码/链接/差异检查，按用户持续授权直接在 main 提交推送，核验该提交对应 CI 与远端主线。建议完整目的提交 `feat(app): 接入向日葵动态待机与冻结回归`；不机械按测试/实现拆成不可玩的公开提交，不直接向 main 提交，除非用户明确授权。
- [x] 工程门槛、原创基准、同版本回放、Windows/Web 两终局和重开/再种植已有当前证据，授权 Git/对应 CI 已收口，主线程勾选总计划 S2-A。S2-B 仍未实施，整个 S2 与原版表现还原继续待验。

## 交付核对表

- [x] 所有新类型、字段、函数都有实际消费者；只缓存一份向日葵 clip，没有全播放器、未来 Zombie 空实现或测试便利公开接口。
- [x] 资源事实、开发离散播放合同、工程测试、实际平台运行和原版行为证据各自有清楚边界。
- [x] 解析/资源图像装配只发生一次；短轨道、隐藏帧、空图、移除和重开没有越界/残影；多个植物借用缓存。
- [x] 暂停、双终局、重开后的时间和实际 Scene 一致；规则/摘要/事件/回放无变化。
- [x] 采样与多植物构图基准、缓存策略测量、覆盖率与平台限制随实现交付，必要检查失败和未运行项保持可见。
- [x] S2-A 独立可玩交付与对应 CI 已完成，允许按已提交细化计划进入 S2-B1；没有提前把整个 S2 标记完成。

## 本地验收补充与执行偏差

最终源码默认/all-features workspace 各 210 项测试通过；两组合分别 clean 后为 4546/5189 行（87.6084%），全部源码路径属于当前 checkout。实际 Scene 验证固定源帧 4/5/28/4、异常时间、动态图层纹理换绑、45 株共享、暂停/双终局/重开及逐批摘要/事件一致。具体命令、基准、结构统计、平台证据及未验证范围统一在[本轮结果](../../behavior/content-foundation-results.md#2026-10-14-向日葵动态待机增量)维护。

任务 5 的“先红测截图等待”流程未取得守护缺失导致的红测证据：首次新增测试失败来自 main 夹具未提供实体查询条件，不能作为生产红测；随后修复夹具并通过，最终补强到恰好 30 秒仅一次错误退出。该过程与真正捕获缺陷的资源/Scene 红测分开登记，不补造红测；此项保留未勾选作为流程偏差，功能验收由最终真实 update 回归和固定相位 GPU 截图支撑。Windows 正常验收使用与 `target/debug` 哈希相同的忽略目录副本，避免运行中锁住 Cargo 的 exe；OS 调度控制只用于输入准备，终局截对之间进程正常运行。

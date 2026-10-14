# 普通僵尸 walk（S2-B1）实施计划

**当前状态：** 2026-10-14 冻结的 B1 源码已完成任务 1 和步骤 1/3/5 的工程范围；Windows 正常窗口缺项、Web 诊断子项、最终复核及 Git/CI 仍待验。本页旧读数只适用于冻结源码；仓库维护后如继续验证，必须重新绑定新源码、锁文件和平台包，不能把旧结果直接当作当前结果。

> **面向执行代理：** 使用 `superpowers:subagent-driven-development` 按任务实施与独立复核，主线程统一构建、Git、服务与平台验收。步骤使用复选框追踪；S2-A 完整交付和本计划提交后才开始源码。

**目标：** 在 S1 已完成的白天成功/失败关卡中，让普通僵尸消费登记的 `anim_walk` 动态资源，保留种植、战斗、暂停、双终局、重开和 S2-A 向日葵动态 idle。

**架构：** `resources.rs` 新增单独的 `ZombieWalk`，拥有一份 `PreparedAnimation`、所选半开区段和源轨道索引；按现有 `Runtime.animation_seconds` 全局开发时钟离散循环，采样借用展开帧。`Scene` 为 Zombie 新增分支并复用现有网格、材质与图像池；移除旧 Zombie 静态 pose。只播放 `anim_walk`，不建立全播放器或动作状态机。

**技术栈：** Rust 1.99.0、edition 2024、Bevy 0.20.0、现有 `pvz-assets` Reanim 解析器、Criterion、cargo-llvm-cov、wasm-bindgen-cli 0.2.129；不增加依赖。

**设计依据：** [总体设计](../specs/2026-10-07-repository-and-runtime-design.md)、[总计划 S2](2026-10-13-minimum-playable-vertical-slices.md#s2有动态反馈的白天单关)、[项目原则](../../principles/README.md)、[贡献指南](../../../CONTRIBUTING.md)。本文锁定下一个独立可玩增量的事实、接口和开发合同，不重建 workspace。

## 全局约束与进入门槛

- 正确性第一、速度第二、内存第三；速度包括运行与开发迭代。
- 文本 UTF-8、LF、无 BOM、末尾换行；自写说明、注释、诊断、显示名使用中文，技术标识符保留原写法。
- 测试默认并行，测试独立 World/临时目录；后台优先，性能基准不与构建、测试、服务或窗口运行重叠。
- 每个交付增量必须从正常入口完成最小可玩闭环；本计划的任务是一个 S2-B1 交付中的审阅/验证步骤，不能把仅缓存或单元测试通过单独称作可玩阶段完成。
- 用户已要求直接在 `main` 提交和推送，由主线程统一执行 Git、安装、构建、服务及外部操作；本文采用该持续授权，不创建分支；提交与推送由主线程执行。
- S2-A 已完成最终复核、目的提交和对应 CI；主线程重新核对本文依赖接口后，先提交本正式计划再实施。S2-A 目的提交为 `3e6634e5103de15b3f8f8ffe5d6c363895e47a3b`，对应 [CI 运行 38536185894](https://github.com/AomaYple/plants-vs-zombies-rs/actions/runs/38536185894) 的 SHA 完全一致，八个 job 全部成功；B1 的 CI 仍须独立核验。
- B1 不修改 `pvz-core`、`runtime.rs`、内容 JSON、事件、规则版本或资源原文；不按动画改变伤害、碰撞、移动、存活或胜负。
- B1 不播放 `anim_walk2`、`anim_eat` 或死亡区段。啃食者 ID/目标、死亡 row/x 快照属于后续 S2-B2 的可靠观察接口增量，禁止几何猜测或未经合同定义的上一帧位置替代。
- 动画资源 fps 是资源事实；全局相位、离散采样、循环及首帧选择是明确开发合同，不是原版程序时序结论。原版有效重复仍为 0/3。

## 已核查事实与证据边界

2026-10-14 初次只读预检针对 S2-A 待提交工作区；随后相同源码已提交为 `3e6634e` 并推送 `main`。静态调查读取项目规则、贡献指南、S2 总计划与 app 相关文件，以下资源统计由 Python XML 稀疏继承核对得到；这些资源结论不构成 B1 构建、测试或平台运行证据。

### 原始资源完整区段核对

`assets/original/reanim/Zombie.reanim` 顶层 fps=12，共 42 条唯一名称轨道，每条 504 帧；一次完整 prepare 展开 21,168 个 `AnimationFrame`。`anim_walk` 的首个连续非负 marker 为 `44..91`，47 帧；索引从 0 开始、区段右端不含 91。

当前 `ZOMBIE_TRACKS` 的 19 条名称全部存在，且顺序与源轨道升序一致。以下每条在全部 47 个帧都 `f >= 0`、图像非空、有效 alpha=1，无隐藏帧、无显式清空、无短轨道；每条仅引用表内一个图像。这里的有效 alpha 按既有绘制合同 `frame.a.unwrap_or(1.0)` 解释，未显式提供的字段在 `PreparedAnimation` 中仍为 `None`，不要求内部值为 `Some(1.0)`。所有 47 个源帧都同时可绘 19 层。

| 源索引 | 白名单名称 | 图像 ID 的 `IMAGE_REANIM_ZOMBIE_` 后缀 |
| --- | --- | --- |
| 12 | `anim_innerarm3` | `INNERARM_HAND` |
| 13 | `anim_innerarm2` | `INNERARM_LOWER` |
| 14 | `anim_innerarm1` | `INNERARM_UPPER` |
| 17 | `Zombie_neck` | `NECK` |
| 18 | `anim_head1` | `HEAD` |
| 19 | `Zombie_innerleg_upper` | `INNERLEG_UPPER` |
| 20 | `Zombie_innerleg_lower` | `INNERLEG_LOWER` |
| 21 | `Zombie_innerleg_foot` | `INNERLEG_FOOT` |
| 22 | `Zombie_outerleg_upper` | `OUTERLEG_UPPER` |
| 23 | `Zombie_outerleg_foot` | `OUTERLEG_FOOT` |
| 24 | `Zombie_outerleg_lower` | `OUTERLEG_LOWER` |
| 25 | `Zombie_body` | `BODY` |
| 28 | `Zombie_tie` | `TIE` |
| 29 | `anim_head2` | `JAW` |
| 30 | `anim_tongue` | `TONGUE` |
| 35 | `Zombie_outerarm_hand` | `OUTERARM_HAND` |
| 36 | `Zombie_outerarm_upper` | `OUTERARM_UPPER` |
| 38 | `Zombie_outerarm_lower` | `OUTERARM_LOWER` |
| 39 | `anim_hair` | `HAIR` |

walk 白名单有 19 个唯一图像；完整 clip 有 37 个唯一图像。37 个 ID 全部在现有 `IMAGE_MAPPING` 中有唯一对应路径并在清单登记，文件存在且导入 SHA-256 与清单一致。加载继续验证完整 clip，不能只验证 walk 后绕过其他区段的引用。

Zombie 文本导入大小 683,819 字节，SHA-256 `4b53ccfa0c000b203405a1f7666eec761935553e533e1746ec0dea517897b9cc`，与 manifest 一致；清单保留来源大小 705,114 与来源 SHA-256 `e8ab9b719bd4262f6914cd6184a42d4a85ffe0804429f4dcdb73d0bd7863cd76`。本次不改资源或清单。

### 当前生产接口与必须保留的消费者

| 文件/API | 当前事实与 B1 处理 |
| --- | --- |
| `animation.rs`：`PreparedAnimation::new(&AnimationClip) -> Self` | 完整稀疏继承，拥有 `tracks/names/fps`。直接复用，不修改公共采样器。 |
| `marker_segment(&str) -> Option<Range<usize>>` | 返回首个连续非负区段；Zombie 用 `anim_walk`。 |
| `drawable_tracks(&Range<usize>) -> Vec<usize>` | 输出源索引，排除全隐藏/全空图；与白名单交集保留升序。 |
| `frame_at(f64, &Range<usize>) -> usize` | 有限非负时间按 `floor(seconds*fps) % length` 采样；空段、负/非有限时间返回起点。保留已有合同和边界测试。 |
| `resources.rs`：`SunflowerIdle::{new,sample}` | B1 复用其缓存模式，保持向日葵行为。 |
| `resolve_images(&AnimationClip, &Manifest) -> Result<BTreeMap<String,String>, ResourceError>` | 完整 clip 映射验证；内部错误当前固定归属 `ANIMATION`（PeaShooter），Zombie 专属装配必须重写 `path` 为 `reanim/Zombie.reanim`。不改全局函数签名。 |
| `LoadedResources::from_provider` | 单次读取/解析、prepare、映射与去重解码；新增 `zombie_walk`。从 `POSE_SOURCES` 删除唯一 Zombie 条目，其余 LawnMower/Sun 保留。 |
| `render.rs`：`Scene::{upload,update}` | `update(&mut self, &mut World, &GameState, f64)` 已接时钟；新增 Zombie 分支，保留 actor 排序、逻辑原点和 `draw_frame`。 |
| `draw_frame` / `TrackMesh` | 池按可见槽位复用实体、Mesh、ColorMaterial；采样输出的源索引用于 `depth + layer*0.01`，材质每帧替换 texture/alpha；尾槽隐藏。源索引不是池槽号。 |
| `main.rs` 的实际 `update` | 已传 `state.runtime.animation_seconds`；生产无需再改时钟，新增实际入口测试捕获误接 tick/零时间的回归。 |
| `runtime.rs` | 已定义暂停、终局冻结、重开清零。只读取，继续用现有 Runtime 回归。 |

Zombie actor 原点维持 `[x/UNITS_PER_PIXEL - 40, row_y/UNITS_PER_PIXEL - 90]`，基础深度 `2 + row*10`。现在静态 pose 使用筛后枚举序号作为深度，B1 动态改用真实源索引（12..39），所以需要真实 Scene 层深度测试；不改变跨行与 actor 基础排序。

## 文件职责与接口锁定

只计划修改以下生产/测试入口，不重构当前较大的 `render.rs`，不创建通用模块、公共测试 API 或新 package：

| 文件 | 具体变化 |
| --- | --- |
| `crates/pvz-app/src/resources.rs` | 新增常量 `ZOMBIE_ANIMATION` 和 `ZombieWalk`；装配字段/读取路径；移除 Zombie 静态 pose；更新旧静态测试为动态整段集成测试，新增资源错误和缓存结构统计。 |
| `crates/pvz-app/src/render.rs` | 新增 `zombie_walk` 字段，upload 移入所有权，Zombie sample 分支；为现有向日葵测试夹具提供合法的裁剪外原创 walk，新增独立 Zombie Scene 夹具和针对性测试。 |
| `crates/pvz-app/src/main.rs` | 仅 `#[cfg(test)]` 部分新增真实 `update` 的 Zombie 时钟接线测试；生产 update 保留。 |
| `crates/pvz-app/tests/presentation.rs` | 原创 walk 缓存、稀疏继承、短轨道、错误、时间与借用断言。 |
| `crates/pvz-app/benches/presentation.rs` | 原创完整 walk 采样及 1/10 只僵尸仿射裁剪基准。 |
| `docs/behavior/content-foundation-results.md` | 新增 B1 实际命令/结果/证据索引/限制；原始产物进入 `artifacts/validation/2026-10-14-zombie-walk/` 或实际执行日期目录。 |
| S2 总计划、README、路线图、架构、测试/性能摘要、`docs/README.md`、项目 AGENTS 导航 | 正式计划与实际完成状态同步，不重复完整结果，不把 B1 完成勾成整个 S2。 |

本计划源码范围固定为上述五个 app 文件，公开结果和入口随完整交付同步。

```rust
const ZOMBIE_ANIMATION: &str = "reanim/Zombie.reanim";

/// 单次装配普通僵尸 walk；源索引和展开帧借用属于开发采样合同。
pub struct ZombieWalk {
    animation: PreparedAnimation,
    segment: std::ops::Range<usize>,
    drawable_tracks: Vec<usize>,
}

impl ZombieWalk {
    pub fn new(animation: PreparedAnimation) -> Result<Self, ResourceError> {
        let segment = animation.marker_segment("anim_walk")
            .ok_or_else(|| error(ZOMBIE_ANIMATION, "没有普通僵尸 walk marker 区段"))?;
        if ZOMBIE_TRACKS.iter().any(|name| !animation.names.iter().any(|n| n == name)) {
            return Err(error(ZOMBIE_ANIMATION, "普通僵尸 walk 指定身体轨道不存在"));
        }
        let drawable_tracks: Vec<_> = animation.drawable_tracks(&segment).into_iter()
            .filter(|&layer| ZOMBIE_TRACKS.contains(&animation.names[layer].as_str()))
            .collect();
        if drawable_tracks.is_empty() {
            return Err(error(ZOMBIE_ANIMATION, "普通僵尸 walk 没有可绘制身体轨道"));
        }
        Ok(Self { animation, segment, drawable_tracks })
    }

    pub fn sample(&self, seconds: f64)
        -> impl Iterator<Item = (usize, &pvz_assets::AnimationFrame)>
    {
        let index = self.animation.frame_at(seconds, &self.segment);
        self.drawable_tracks.iter().filter_map(move |&layer| {
            let frame = self.animation.tracks[layer].get(index)?;
            (!frame.f.is_some_and(|n| n < 0.0)
                && frame.i.as_ref().is_some_and(|id| !id.is_empty()))
                .then_some((layer, frame))
        })
    }
}
```

只有 `new` 和 `sample` 是生产消费者需要的公开 API；结构统计在 `resources.rs` 私有测试中读取字段。所有僵尸共享 Scene 的一个缓存和 Runtime 当前全局相位；新刷出的僵尸按该全局相位开始，不按 ID/出生时刻重置。只在 Runtime 重开时重置全局时钟；这明确属于开发选择。

## 任务 1：缓存、装配与真实 Scene walk 接入，形成一个可玩的目的变更

**文件：** resources.rs、render.rs、main.rs 的测试区、tests/presentation.rs、benches/presentation.rs。消费上述现有接口，产出 `LoadedResources.zombie_walk: ZombieWalk`、`Scene.zombie_walk: ZombieWalk`；不改 `Scene::update` 签名。本任务完成相关代码、测试及基准目标，进入任务 2 的完整验收前不提交未闭环源码到 main。

- [x] **步骤 1：在 presentation.rs 写原创最小 walk 夹具与先失败测试。** 名称采用当前 19 条白名单（名称是接口选择，不复制资源变换）；marker 为源帧 `1..4`。源序是 marker=0、可绘附件=1、`anim_innerarm3`=2、可绘非白名单=3、剩余 18 条白名单=4..21。全隐藏的白名单轨道仍存在，证明“存在核对”与“可绘选择”分别执行。

```rust
fn original_walk() -> PreparedAnimation {
    use pvz_app::resources::ZOMBIE_TRACKS;
    let mut xml = String::from("<fps>12</fps><track><name>anim_walk</name><t><f>-1</f></t><t><f>0</f></t><t/><t/><t><f>-1</f></t></track><track><name>原创附件</name><t><f>0</f><i>原创附件图</i></t><t/><t/><t/></track>");
    for (n, name) in ZOMBIE_TRACKS.iter().enumerate() {
        if n == 1 {
            xml.push_str("<track><name>原创非身体</name><t><f>0</f><i>原创附件图</i></t><t/><t/><t/></track>");
        }
        xml.push_str(&format!("<track><name>{name}</name>"));
        xml.push_str(match n {
            0 => "<t><x>7</x><a>0.5</a><i>原创红图</i><f>0</f></t><t/><t><x>9</x><a>2</a><i>原创蓝图</i></t><t><i/></t>",
            1 => "<t><f>-1</f><a>-1</a><i>原创绿图</i></t><t/><t><f>0</f></t><t><f>-1</f></t>",
            18 => "<t><x>20</x><y>4</y><sx>2</sx><sy>3</sy><i>原创绿图</i><f>0</f></t><t/>",
            _ => "<t><f>-1</f><i>原创绿图</i></t><t/><t/><t/>",
        });
        xml.push_str("</track>");
    }
    PreparedAnimation::new(&pvz_assets::parse_reanim(&xml).unwrap())
}

#[test]
fn 普通僵尸walk借用源帧过滤附件保留源索引() {
    use pvz_app::resources::ZombieWalk;
    let walk = ZombieWalk::new(original_walk()).unwrap();
    let snapshot = |seconds| walk.sample(seconds)
        .map(|(layer, frame)| (layer, frame.i.as_deref().unwrap(), frame.x))
        .collect::<Vec<_>>();
    assert_eq!(snapshot(0.0), [(2, "原创红图", Some(7.0)), (21, "原创绿图", Some(20.0))]);
    assert_eq!(snapshot(1.0 / 12.0), [(2, "原创蓝图", Some(9.0)), (4, "原创绿图", None)]);
    assert!(snapshot(2.0 / 12.0).is_empty());
    assert_eq!(snapshot(3.0 / 12.0), snapshot(0.0));
    let first = walk.sample(0.0).next().unwrap().1;
    let again = walk.sample(0.0).next().unwrap().1;
    assert!(std::ptr::eq(first, again));
}
```

另写逐帧位置夹具（白名单全部存在、仅一条有图），断言 `1/12 ± 1e-9`、`3/12 ± 1e-9`、多循环、负值、NaN、±Infinity 和 f64::MAX；非有限/负时间首帧，有限极大时间只落在区段内且不 panic。缺 marker、marker 全负、直接构造空 marker、缺一个白名单轨道、白名单全隐藏/全空图分别拒绝，`error.path == "reanim/Zombie.reanim"`，reason 对应上面三条中文诊断。缺轨道病例从 `original_walk()` 的 `names/tracks` 同步移除一个索引；全隐藏病例把白名单帧 `f=Some(-1.0)`；全空图病例把 `i=Some(String::new())`，不依赖源资源或 GPU。

- [x] **步骤 2：主线程运行 `cargo test -p pvz-app --test presentation --locked 普通僵尸walk`，确认新测试因缺 ZombieWalk 而失败，再实现上方缓存。** 测试保留默认线程；不与其他 Cargo 共享 target 锁并发。

- [x] **步骤 3：在 from_provider 显式读取一次 Zombie，完成完整映射/去重解码并保存缓存，删除旧静态来源条目。** 每个阶段错误保留 Zombie 路径；`resolve_images`、provider.read 的图像错误、decode 错误分别用 `.map_err(|failure| error(ZOMBIE_ANIMATION, failure.reason))`。其余静态来源、图像集合、音频和向日葵路径保留。实际插入位置为 sunflower_idle 装配后、剩余 POSE_SOURCES 循环前。

```rust
let bytes = provider.read(ZOMBIE_ANIMATION).await?;
let source = std::str::from_utf8(&bytes)
    .map_err(|_| error(ZOMBIE_ANIMATION, "动画不是 UTF-8 文本"))?;
let clip = parse_reanim(source)
    .map_err(|_| error(ZOMBIE_ANIMATION, "动画解析失败"))?;
let mapping = resolve_images(&clip, provider.manifest())
    .map_err(|failure| error(ZOMBIE_ANIMATION, failure.reason))?;
for (id, path) in mapping {
    if let std::collections::btree_map::Entry::Vacant(entry) = images.entry(id) {
        let bytes = provider.read(&path).await
            .map_err(|failure| error(ZOMBIE_ANIMATION, failure.reason))?;
        entry.insert(decode_image(&path, &bytes)
            .map_err(|failure| error(ZOMBIE_ANIMATION, failure.reason))?);
    }
}
let zombie_walk = ZombieWalk::new(PreparedAnimation::new(&clip))?;
```

`LoadedResources` 加字段并在 `Ok(Self {...})` 移入；`POSE_SOURCES` 不再含 Zombie；`poses` 精确键为 Peashooter/Pea/Sun/LawnMower。更新 `ZOMBIE_TRACKS` 注释说明身体选择用于 walk，避免仍描述只取第44帧。全映射/去重保证图像总数仍是 `IMAGE_MAPPING.len()+1`，没有重复解码/prepare Zombie，也没有静态副本。

- [x] **步骤 4：更新资源集成测试。** 对全部 47 帧采样 `step/12`：索引为 `44+step`，输出精确源索引 `[12,13,14,17,18,19,20,21,22,23,24,25,28,29,30,35,36,38,39]`，每次 19 层、`frame.a.unwrap_or(1.0) == 1.0`、图像 ID 属于表内身体集、实际 images 有对应数据，借用地址等于缓存 `tracks[layer][44+step]`。`47/12` 回到44；不包含任何附件。把旧 `登记普通僵尸构图精确选择身体轨道排除附件` 改成动态整段核对，不保留访问旧 pose 的断言。

新增隔离临时清单病例移除 `reanim/Zombie_head.png`：维护 package.entry_count，复制 BACKGROUND、PeaShooter/SunFlower/Zombie XML 以及前两完整 clip 需要的图像，按现有向日葵错误测试方法装配；预期资源错误 path=Zombie、reason=图像引用没有唯一登记映射。未知图像原创 clip 直接调用 resolve_images 验证拒绝；生产装配显式重归属保证不误报 PeaShooter。测试不改变工作目录或共享环境。

- [x] **步骤 5：接入 Scene，保持池及坐标。** `Scene` 和 `upload` 增加 zombie_walk；源帧借用交给同一个 draw 闭包，所有 actor 仍消费当前 board 存活集合。

```rust
match kind {
    VisualKind::Sunflower => self.sunflower_idle.sample(animation_seconds).for_each(&mut draw),
    VisualKind::Zombie => self.zombie_walk.sample(animation_seconds).for_each(&mut draw),
    _ => self.poses[&kind].iter().enumerate().for_each(&mut draw),
}
```

现有 `sunflower_scene_fixture` 创建一个全部白名单存在、仅一条可绘且 `x=1000` 的原创 walk，保证向日葵测试的隔离范围；去掉其 Zombie 静态键。新增 `#[cfg(test)] pub(crate) fn zombie_scene_fixture(world: &mut World) -> Scene` 供 render/main 测试使用，创建独立 World 所需的 Mesh2d/Visibility/VisibilityClass 注册、Assets 三资源，上传红蓝绿各2×2图像。walk 采用步骤1原创 XML语义；向日葵缓存保留合法裁剪外图层，其余静态帧裁剪外。该函数仅测试编译可见，不扩大库 API；无需重构 render.rs 或现有夹具。

- [x] **步骤 6：先写并验证真实 Scene 回归，再修正接线至通过。** 测试从原创 `GameState::from_day_definition` 装配，不直接修改 core 私有字段：规则基于现有 development-failure JSON 的严格解析变体，仅测试内替换 wave 为 tick1 的两只 row0/row2、x=30000/50000，`zombie_speed=1`（规则校验要求所有距离为正），`mower_trigger_x=4000`、`defeat_x=-20000`，确保僵尸留在视口且不被割草机吞掉。先 step 到 tick1，再对同一 game 多次 Scene 更新；这些是原创测试参数，不改变版本化内容。

真实消费者断言如下，不用只比较 sample 返回值代替 Scene 验收：

| 场景 | 具体预期 |
| --- | --- |
| 首相位0 | 两个 Zombie 各画2层，源索引2/21；底图红alpha=.5，短图绿，短图缩放2×3。row0 基础depth2，底层depth2.02；row2 底层depth22.02。 |
| 相位1/12 | 每只仍2层，底图变蓝、x局部7→9、alpha夹到1；源索引4的绿图alpha夹到0且槽依然Visible；entity/Mesh/ColorMaterial Handle相等，图像数量稳定。 |
| 相位2/12 | 所选帧全部空/隐藏/超出短轨道，已分配4槽全部Hidden，池容量和Assets计数不下降不增长。 |
| 相位3/12 | 回首相位，同一槽恢复Visible并恢复红/绿图、alpha、顶点、源深度；没有新纹理/材质。 |
| 源索引空洞 | 2和21/4的深度是源索引，不被压成0/1，也不按白名单列序重排。 |
| 两只同时绘制 | 每只原点来自自己的 row/x，局部x7/9分别叠加到两原点；全部层共享缓存和3纹理，不能把第二只画在第一只位置。 |
| 单只消失 | 用原创规则快速击杀其中一只，读取 board 确认只剩另一ID，Scene按新存活集重建；只保留幸存者图层，其余尾槽Hidden，幸存者原点正确，不播放死亡。 |
| 完全/部分裁剪 | 私有测试把帧平移改到边界，顶点均在800×600范围、UV插值正确；完全越界隐藏旧槽，恢复视口后复用槽。 |
| 额外采样 | 同一game多次不同时间update前后digest、Zombie id/row/x/health、植物状态完全相同；Scene不消费/重排 Runtime事件。 |

几何断言用现有 `positions/uvs/alpha/asset_counts/scene_snapshot`；slot只用于读取复用对象，深度检查读取 Transform.z。首tick后两只实际x为29999/49999，读取board的x后计算锚点，避免把出生x误当推进后的x。单只消失测试通过原创规则/命令使core实际移除，不直接删除生产实体：

```rust
fn zombie_game() -> pvz_core::GameState {
    let mut definition: pvz_data::DayLevelDefinition = serde_json::from_str(
        include_str!("../../../content/levels/development-failure.json"),
    ).unwrap();
    definition.initial_sun = 150;
    definition.rules.zombie_speed = 1;
    definition.rules.mower_trigger_x = 4000;
    definition.rules.defeat_x = -20000;
    definition.rules.zombie_health = 20;
    definition.rules.pea_damage = 20;
    definition.rules.pea_speed = 100000;
    definition.rules.shoot_interval = 1;
    definition.rules.wave = vec![
        pvz_data::DaySpawnDefinition { at: 1, row: 0, x: 30000 },
        pvz_data::DaySpawnDefinition { at: 1, row: 2, x: 50000 },
    ];
    let mut game = pvz_core::GameState::from_day_definition(&definition).unwrap();
    game.step(&[]);
    assert_eq!(game.board().unwrap().zombies().len(), 2);
    game
}
```

在消失测试中：首相位0绘制并保存4槽及3类资产数量；记录row0/row2两个ID；用同批 `SelectCard(Peashooter)`/`Plant(Cell { row:0,column:0 })`，随后最多4次 `game.step(&[])`，逐批收集事件，直到row0 ID消失。必须收到该ID的 `ZombieDied`，row2 ID仍存在，棋盘Running，Scene下一次在相位0只绘2槽且其余2槽Hidden。射手静态测试图裁剪外，不增加可见槽；row2 x每tick减少1，锚点从当前board读取。期间不产生天空阳光，沿用基准内容的sky_sun_interval=300。随后对同局采样1/12和3/12验证容量不增长、幸存者纹理与坐标正确。

- [x] **步骤 7：在 main 测试调用实际 update 验证时钟，保持生产接口。** 用 zombie_scene_fixture 和上述稳定存活测试game；提交 Pause 后设置 `animation_seconds=0.12`（与tick数不同），`Time<Real>` 推0.01两次且实际 `update`：底层顶点局部x=9、蓝纹理，时钟=.12不变。Resume命令帧相位不变，再推.14秒：相位=.26，`floor(.26*12)%3=0`，局部x=7且红纹理。按基础depth+源索引2定位两行底层；复用现有 harness，不改变 main 的 elapsed 或 ScreenshotWait 行为。

- [x] **步骤 8：扩展暂停、双终局、重开和回放真实 Scene 测试。** 在 `Runtime::failure_day()` 正常场景中保持 row1 向日葵，失败不种row0射手、胜利种row0射手；确认途中有存活 Zombie 的动态层被实际绘制。运行到 Lost/Won，冻结100秒后digest、animation_seconds、可见层顶点/纹理/alpha完全不变，Victory/Defeat只出现一次。失败终局保留的存活Zombie冻结，胜利移除后的槽隐藏；不强求胜利存在Zombie层。R后tick0、时钟0、旧Zombie层隐藏；新波次出现后按新全局相位重新绘制，仍使用旧池，不继承上局Zombie位置。保留向日葵重开再种植测试。将带/不带Scene的同版本回放逐批比较 `(state_digest, take_events, animation_seconds)`，涵盖暂停恢复、伤害、死亡、移除、终局、重开；在每批额外调用不同时间Scene，核心结果仍相同。

- [x] **步骤 9：写原创基准并运行其断言。** 详见下一节，生产数据和公共 fixture 不复制原版帧变换或图片。使用独立构造代码，不为 benchmark 公开缓存内部字段。

- [x] **步骤 10：主线程运行局部验证。** `cargo test -p pvz-app --locked`（含bin真实Scene/main测试、integration和doctest）、`cargo clippy -p pvz-app --all-targets --locked -- -D warnings`、`cargo fmt --all -- --check`。记录源码状态、命令、结果；失败修正原因，不跳过旧向日葵测试或降低lint。

## 基准与缓存结构统计合同

新增2类基准，保留已有基准。构造和parse/prepare/marker/索引选择在计时外；完整迭代 `ZombieWalk::sample`，不能只测frame_at然后声称全采样耗时。

1. **`表现/原创42轨道504帧_19身体47帧_12fps普通僵尸walk采样`：** 原创42轨道×504帧，其中1 marker、19白名单、22非白名单；marker44..91，主体图像为同一原创标识，变换为原创固定`x=frame % 7`、`y=layer % 5`，全程可绘。在计时外断言0→源44（x2）、46/12→源90（x6）、47/12→源44（x2），输出19层；计时遍历所有借用对并black_box，输出计数19。名称借用白名单只验证筛选合同，不读取原版 XML。实现以下原创构造再建立Criterion消费者：

```rust
let mut xml = String::from("<fps>12</fps><track><name>anim_walk</name>");
for frame in 0..504 {
    xml.push_str(match frame {
        0 | 91 => "<t><f>-1</f></t>",
        44 => "<t><f>0</f></t>",
        _ => "<t/>",
    });
}
xml.push_str("</track>");
let names: Vec<_> = pvz_app::resources::ZOMBIE_TRACKS.iter()
    .map(|name| (*name).to_owned())
    .chain((0..22).map(|n| format!("原创非白名单{n}")))
    .collect();
for (layer, name) in names.iter().enumerate() {
    xml.push_str(&format!("<track><name>{name}</name>"));
    for frame in 0..504 {
        if frame == 0 {
            xml.push_str(&format!("<t><f>0</f><i>原创walk图</i><x>0</x><y>{}</y></t>", layer % 5));
        } else {
            xml.push_str(&format!("<t><x>{}</x></t>", frame % 7));
        }
    }
    xml.push_str("</track>");
}
let prepared = PreparedAnimation::new(&pvz_assets::parse_reanim(&xml).unwrap());
assert_eq!(prepared.tracks.len(), 42);
assert_eq!(prepared.tracks.iter().map(Vec::len).sum::<usize>(), 21168);
assert_eq!(prepared.marker_segment("anim_walk"), Some(44..91));
let walk = pvz_app::resources::ZombieWalk::new(prepared).unwrap();
for (seconds, x) in [(0.0, 2.0), (46.0 / 12.0, 6.0), (47.0 / 12.0, 2.0)] {
    let frames: Vec<_> = walk.sample(seconds).collect();
    assert_eq!(frames.len(), 19);
    assert!(frames.iter().all(|(_, f)| f.x == Some(x)));
}
c.bench_function("表现/原创42轨道504帧_19身体47帧_12fps普通僵尸walk采样", |b| {
    b.iter(|| {
        let mut count = 0;
        for item in walk.sample(black_box(1.25)) {
            black_box(item);
            count += 1;
        }
        black_box(count)
    });
});
```
2. **`表现/原创普通僵尸walk采样与多对象仿射裁剪/1` 与 `/10`：** 使用同一缓存，10原点设5行×2列，`[80+column*300, 100+row*80]`，图像40×50，逻辑裁剪800×600。固定秒1.25=源59；全部19层输出4顶点，计数分别76/760。计时包含采样、仿射、几何分配与裁剪，不含prepare、纹理上传、Bevy池更新或GPU。对1/10均记录Mean、95%区间及输入规模；不把10只称为关卡真实同时存活上限。

私有资源测试 `登记普通僵尸walk缓存结构统计` 统计一份Zombie缓存：fps12、range44..91、42轨道、各504帧、21168展开帧、19源索引；`i/font/text` 字符串有效字节量、names字节量、外层/内层/names/索引Vec的len与capacity。用与当前向日葵测试相同的serde_json println输出到原始日志，`len <= capacity`，字符串量与独立flatten计算核对；实际数值来自 Rust 所有权结构，不把Python计数当capacity测量。确认ZombieWalk只拥有一份PreparedAnimation，Scene从LoadedResources移入而非clone，旧Zombie pose不存在；多对象/重开测试确认Images/Meshes/Materials数量在峰值池容量内稳定。

命令由主线程在无其他项目测量争抢时执行：

```sh
cargo test -p pvz-app --lib --locked 登记普通僵尸walk缓存结构统计 -- --nocapture
cargo bench -p pvz-app --bench presentation --locked -- 普通僵尸
```

本批引入缓存及采样策略，记录空间代价。结构len/capacity、字符串有效字节、资产数量均不等于堆分配次数、峰值进程内存或显存；没有同条件旧动态基准，不宣称优化收益。若后续为节约21,168展开帧改布局/分配，另立优化目的、提供同条件时间和内存测量，不能混入B1。

## 任务 2：完整平台可玩验收、文档与主线交付

**文件：** 正式计划、S2总计划和上方文档入口、content-foundation-results.md。消费任务1最终源码/锁文件，产出可追溯Windows/Web实际运行证据、所有适用检查结果和main交付状态。不存在平台运行证据时本任务未完成，不能仅靠CPU测试关闭B1。

- [x] **步骤 1：完成适用workspace验收。** 依贡献指南运行格式、`cargo check --workspace --all-targets --locked`、`cargo test --workspace --locked`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo doc --workspace --no-deps --locked`、`cargo deny check`；兼容probe feature补对应 `--all-features` 检查/测试/Clippy/doc，保留doctest。默认和all-features干净coverage分别执行/导出，核对当前checkout路径与分母，报告新增缓存/失败/Scene/main路径命中及Web/设备缺口，不用忽略关键代码制造通过。运行相关core回放和上述app基准，其他未重测场景明确记录。

- [ ] **步骤 2：Windows正常登记资源入口验收。** 构建 `cargo build -p pvz-app --locked`，由主线程隐藏启动 `target/debug/pvz-app.exe`；能可靠捕获目标窗口/输入就保持后台，必须聚焦时仅做必要交互并说明。不得用probe或smoke代替正常输入。

| 正常玩家路径 | 必须观察及记录 |
| --- | --- |
| 默认正常入口 | 草坪/卡片/植物/僵尸完整，普通僵尸手腿/身体随着时间变化且逻辑位置继续移动；向日葵idle继续变化，无附件出现、无旧静态重影。至少拍一只无遮挡Zombie同一cycle两个相位，截图包含时间与目标窗口身份。 |
| F → 无防守失败 | 进入现有开发失败场景，不种row0射手；观察正常Zombie动态直到失败，冻结后继续观察至少1秒，位置/动作均冻结。 |
| R → 防守胜利 | 失败后R重开，正常卡片/草坪点击在row0种射手；可在row1种向日葵，观察战斗→正常胜利。移除Zombie立即隐藏，无死亡伪动画/旧残影，胜利后动画冻结。 |
| P/空格暂停恢复 | 在存活Zombie无遮挡时暂停，跨至少一个47/12秒循环期观察同帧，再恢复，动作与移动继续；不补播暂停期间时间。 |
| 胜利/失败后R | 新局时钟清零、旧池隐藏，新Zombie出现后继续循环；再种植/收集/铲除仍可用。 |
| 窗口缩放 | 保持800×600逻辑坐标与输入映射，僵尸身体不越错误裁剪或跨行，资源不随resize重复上传。 |

时间截图若Zombie位置移动，可在正常暂停前记录两个未暂停相位（仅作为动态观察），暂停后的图对则必须静止；精确源帧端点由CPU测试证明，不把两张不同位置截图单独当换帧证明。必要的无遮挡图对同时核对局部手腿差异与身体锚点；后台捕获失败时保留记录，不冒充通过。

- [x] **步骤 3：Windows固定GPU冒烟回归。** 复用现有隐藏入口、AppExit传播和截图等待冻结合同，不新增B1专用CLI：`target/debug/pvz-app.exe --smoke 30 --slice-smoke --screenshot`，`--failure-smoke --smoke 400 --screenshot`；终局不足期限 `--failure-smoke --smoke 2` 应退出1；默认成功 `--slice-smoke --smoke 600 --screenshot` 应退出0。截图/日志按实际路径归档索引，避免把已终局无Zombie图当walk端点证据；确切源44/90由生产缓存和真实SceneCPU测试闭合。

- [ ] **步骤 4：Web构建与正常隐藏浏览器验收。** 当前冻结源码已有构建、正常操作、局部动态图对及清理记录；由于 Tab8 关闭前未查询诊断，步骤整体仍未完成。仓库维护后的新验证必须重新绑定实际平台包。

```sh
cargo build -p pvz-app --target wasm32-unknown-unknown --release --locked
wasm-bindgen --target web --out-name pvz_app --out-dir target/web target/wasm32-unknown-unknown/release/pvz-app.wasm
cargo build -p pvz-app --target wasm32-unknown-unknown --release --features probe --locked
wasm-bindgen --target web --out-name pvz_app --out-dir target/web-probe target/wasm32-unknown-unknown/release/pvz-app.wasm
python -m http.server 8000 --bind 127.0.0.1
```

主线程统一服务生命周期，绑定loopback，记录wasm-bindgen版本0.2.129及生成包状态。隐藏浏览器打开正常 `http://127.0.0.1:8000/web/`，点击启动按钮；不能用 `?probe=1` 验收资源walk。重做上表F无防守失败、R种row0射手胜利、双终局冻结、暂停恢复、再重开新Zombie、向日葵idle及无遮挡局部动态图对；检查浏览器warn/error和资源加载失败，保留历史shader问题的已知限制。probe构建仅验证兼容，不把其静态探针当walk通过。关闭本任务服务/浏览器；macOS/Linux构建回归来自对应CI，实际窗口未运行则明确未运行，Android/iOS仍待接入。实际补证登记于 `web-extra-observation.json`、`capture-files.json`、`web-captures.jsonl`；收集阳光与铲除、向日葵运行动态、16张额外截图和 `image-comparison.json` 已核对。`web-extra-sunflower-a/b.jpg` 当时已 Won 且受阳光遮挡，不作为向日葵动态证据；运行中动态仅采用 `web-extra-sunflower-running-a/b.jpg`。`compare_captures.py` 使用 bundled Python 退出0；Tab8已关闭、PID11512回环服务已停止，并核对无 pvz-app/cargo 进程。tab7 的 `web-final-observation.json` 记录 warn/error=[]，tab8 未查询诊断，不能外推为空。JPEG局部比较只支持本轮可观察冻结/变化结论，不等同未压缩 GPU 逐像素或原版行为证据。Web 构建、正常操作和清理证据已取得，但诊断日志缺项仍是本步骤必要待补项。

- [x] **步骤 5：写实际结果与同步入口。** B1记录源码状态、环境、target/feature、命令/退出码、覆盖/基准/缓存统计、截图与日志路径、输入路径、限制；阶段文字仅声明“普通僵尸walk开发增量完成”。S2-A勾选由它的交付证据决定，S2-B拆B1/B2，B2可靠事件到表现接口仍未实现；整个S2不因B1关闭。新正式计划链接在docs索引/AGENTS等已有导航中更新，不修改长期原则以降低要求。

- [ ] **步骤 6：主线程最终复核与Git/CI交付。** 核查生产变更仅resources/render、main仅测试；无core/runtime/content/资源变化、无旧Zombie静态消费者、无新公开无用API、无无关文件；检查编码/链接/资源清单、`git diff --check`与暂存差异，保留其他任务修改。适用检查与实际Windows/Web验收全部完成后提交 `feat(app): 接入普通僵尸动态行走表现`，直接main普通推送（用户持续授权），记录目的commit；只以该commit对应CI核验完成，不引用S2-A旧CI代替。核对远端main、工作区剩余改动和本任务服务清理；无分支/PR流程、无强推。

## 最终验收与自审清单

- [x] 进入门槛有S2-A目的commit与对应CI，正式计划已复核/提交；冻结源码的依赖接口未漂移。维护后继续验证需重新核对。
- [x] walk44..91、12fps资源事实与全局开发循环选择分开写明；19身体全段图像/可见性和源索引核对保留。
- [x] Zombie完整clip解析/映射/prepare一次、去重解码一次；错误归属Zombie；旧pose不存在，其他消费者继续有效。
- [x] 原创测试具体覆盖借用、源层序、空图/隐藏/短轨道/时间边界/错误、两只不同位置、池复用、材质换绑、移除尾槽、裁剪、actual main update。
- [x] Runtime与内容未改；同版本回放digest/事件顺序不变；暂停、胜利/失败、重开、新波次及向日葵回归闭合。该勾选只针对冻结源码的既有工程证据。
- [x] 新采样/1与10对象基准和真实缓存结构统计已执行，测量口径与限制明确，没有无依据时间或内存收益声明。维护后不复用其数值作为新基线。
- [ ] Windows/Web正常输入与资源walk动态证据、双终局及重开已取得；GPU冒烟与CPU测试分别报告，其余平台限定于实际构建/运行范围。
- [ ] 正式main提交/推送/该commit CI、编码、链接、资源边界和服务清理完成；所有执行结果由主线程实际核验后登记。
- [x] 原版walk/walk2选择、出生相位、插值/混合、切换与死亡时机仍未知，原版有效重复0/3；B1持续walk（包括碰到植物的僵尸）只是开发降级表现，不声明原版动作匹配。
- [x] 啃食不靠Damaged target或几何位置猜攻击者，死亡不靠不存在的当前位置猜锚点；B2先固定可靠输入与事件/回放合同，再另做可玩增量。

自审结果：所有B1需求均映射到任务1缓存/装配/真实消费者/测试/基准，任务2平台/文档/交付；没有占位步骤，没有未经消费者验证的通用框架或未来动作占位。帧/图像事实、CPU 测试和冻结源码工程结果按对应证据范围记录；本计划不把这些结果扩大为当前新源码的编译、GPU、平台运行或原版行为验收。Windows 正常窗口、Web 诊断、最终复核和 Git/CI 仍是关闭 B1 的必要条件。

## 2026-10-14 执行状态

任务1实现、118项app测试与限定修复复核已完成，无开放Critical/Important；只修改五个app文件，main仅测试，源码尚未提交。步骤1/2已取得缺 `ZombieWalk` 的3处E0433红测；步骤6的Scene测试并非全部先失败，保留实际顺序偏差，不声明全任务完整TDD。alpha缺省按 `unwrap_or(1.0)` 有效绘制值、恢复命令当批core可推进tick而动画相位不变，均有原始失败与修正记录。

冻结源码的任务2步骤1默认/all-features各222项workspace测试（含doctest）、适用check/Clippy/doc/fmt/deny/actionlint/native构建、两组干净覆盖率与三项walk基准已通过；相关core回放包含在workspace测试，未重跑未修改core/assets的旧时间基准。步骤3四条冻结源码 Windows GPU CLI 实际退出码为0/0/1/0。步骤5入口与结果文档已同步。步骤2仍待完成：Windows旧正常窗口JPEG记录缺完整无遮挡身体动态图对、失败冻结图对与resize，当前CUA无getApp/listWindows，无法补齐。步骤4的 default/probe 构建/生成及 Web 正常动态图对、跨循环暂停/恢复、双终局/重开、收集阳光、铲除、向日葵复验与DPR1 resize均有记录；`compare_captures.py` 使用 bundled Python 成功退出0并生成 `image-comparison.json`。Tab8已关闭且关闭前未查询 warn/error，Web 步骤4诊断子项仍待补，整个步骤未完成。步骤6和整批最终验收保持未完成；后续维护后的新验证尚在运行，不能复用冻结源码数值作为新结论。详细读数、索引和限制见[本批结果](../../behavior/content-foundation-results.md#2026-10-14-普通僵尸动态行走增量)。


## 2026-10-15 维护后阶段性收尾

本轮继续消费维护后的源码，完成无消费者接口清理、粒子属性树缓存清理、app 显示修复、Web 状态回归和文档整合。默认/all-features workspace 测试、check、Clippy、doc、fmt、deny、actionlint、干净覆盖率、资源解析/表现基准、native release、WASM default/probe 绑定及四条 CLI 冒烟均已通过；结果和原始输出见[维护后结果](../../behavior/content-foundation-results.md#2026-10-15-全仓审计与维护阶段性收尾)。

本节不关闭步骤2 Windows 正常窗口、步骤4真实浏览器诊断、设备音频、原版行为和六平台完整运行；这些仍需真实平台或原版证据，不能用维护后的 CPU/CLI/Node 测试替代。维护工作的提交、推送和合并按用户本轮授权单独记录，不把 B1 阶段标为完成。

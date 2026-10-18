# pvz-app

Bevy 0.20 应用，当前提供白天开发切片，以及纯视口/输入、资源装配、2D 仿射/裁剪和音频探针。默认切片使用登记资源，向日葵动态 idle 和普通僵尸开发 walk 已进入主线，其余对象继续静态构图；完整动作混合与原版粒子尚未接入。B1 遗留平台验收及啃食/死亡反馈由[第一阶段](../../docs/superpowers/plans/2026-10-17-phase-1-day-combat.md)承接；历史工程与运行证据见[维护结果](../../docs/behavior/content-foundation-results.md#2026-10-15-全仓审计与维护阶段性收尾)。800×600、模拟 60 tick/s、背景 x=220 裁剪和 marker 解释均为开发合同，不构成原版行为还原证据。

正常入口 `Runtime::day()` 消费编译时嵌入的原创 [development-day.json](../../content/levels/development-day.json)，native/Web 使用相同字节，schema 1、规则 2.0、依据 `development`、seed 7、初始阳光 150 和有序波次均由内容定义。默认波次无操作会由割草机完成胜利。按 `F` 可从同一正常入口进入明确标记的 [development-failure.json](../../content/levels/development-failure.json) 开发失败场景；该场景放任第二只僵尸会失败，种植豌豆射手仍可获胜。`F` 是当前阶段的单一开发入口，不是通用选关系统。`Runtime::from_day_json(&str)` 在解析前按 UTF-8 字节数拒绝超过 1048576 字节的输入，再复用 serde_json 和 core 纯类型装配；损坏 JSON、未知/遗漏/重复字段、未知版本和非法参数明确返回中文包装错误，保留原始原因，不回退默认规则。项目自带内容无效时正常入口明确失败；独立 `Runtime::default()` 继续用于原创空棋盘 probe。

帧推进读取 `game.clock().ticks_per_second()`，支持 schema 1 的 1..=1000 Hz，默认仍为 60 Hz。每帧最多消费 250ms，余量只来自运行中的有效帧；暂停和终局冻结模拟/动画且丢弃追赶时间，恢复不补停顿，重开保留内容时钟并清空动画及旧帧余量。频率只改变现实耗时对应的 tick 数，内容间隔与速度仍以 tick 为单位，不自动换算平衡参数。

## 运行与探针

所有命令从仓库根执行。默认读取 `assets/manifest.json` 登记的背景、六类对象所需的 Reanim/图像集合（94 张图像）、四类事件音效，以及原有 `seedlift.ogg`/`ZombiesOnYourLawn.ogg` 音频探针素材；先检查大小/SHA-256/文本规范，再解码。native 使用 `AssetRoot` 只读输入，Web 使用 gloo-net 同源 `/assets/` 下载并复用同一清单校验。没有安装目录或资源根覆盖参数。缺失、损坏、未登记图像引用与下载失败会显示中文错误，不会静默变成占位资源。

```sh
cargo run -p pvz-app --locked
cargo run -p pvz-app --locked -- --probe
cargo run -p pvz-app --locked -- --probe --smoke 120 --screenshot
cargo run -p pvz-app --locked -- --smoke 160 --audio-smoke --screenshot
cargo run -p pvz-app --locked -- --smoke 600 --slice-smoke --screenshot
```

`--probe` 只显示原创背景、网格与仿射夹具，不读取原版素材或音频。`--smoke` 创建不可见且不请求焦点的窗口，持续更新，在指定真实绘制帧数后进入退出流程。`--slice-smoke` 用正常游戏命令驱动默认切片，每帧推进 15 个开发模拟 tick；`--failure-smoke` 通过与 F 键相同的正常场景入口进入开发失败场景，在指定帧数结束时必须观察到 `Defeat`。`--screenshot` 等 GPU 回调完成后退出，probe 写入 `artifacts/viewport/probe.png`，登记默认切片写入 `artifacts/vertical-slice/registered.png`，失败场景写入 `artifacts/vertical-slice/failure.png`；这些文件均被忽略，保存或资源失败返回错误。资源加载等待上限为 30 秒，截图请求后的最终等待上限为 30 秒，使用 `Time<Real>` 的真实时钟截止时间；回调完成、保存失败和缺失回调分别产生一次成功或非零退出，不依赖模拟 tick、暂停状态或帧数。后台运行与截图验收统一协调。冒烟事件日志中的“帧末tick”是消费该帧全部事件后的模拟 tick，保留事件相对顺序，但不是逐事件发生时刻。

鼠标、触摸和 P/空格/R/E/M 控制按键通过同一 Bevy 有序 `WindowEvent` 适配，只消费主窗口并完整保留同批开始、移动、结束、取消及尺寸/DPI 变化的顺序。坐标先乘当时的窗口 DPI，再逆变换到逻辑点。视口外按下不产生游戏命令；捕获的指针越界、取消、失焦或视口变化会取消，多个指针状态相互独立。控制键只接收当前有焦点窗口的非 repeat 按下，释放不产生操作，同批多次按压及与指针的交错保留顺序；暂停切换考虑此前排队的暂停、恢复与重开意图。重开只清空其前面的铲除选择，其后的选择继续有效；重开后的草坪命令不会命中重开前的旧阳光实体。

正常切片将 UI/草坪按下映射为选卡、种植、收集、铲除、暂停或重开；P/空格暂停与恢复，R 重开，F 进入明确开发失败场景。伤害、阳光、冷却和胜负由 core 决定，表现只消费状态与事件。只有原创 probe 的空棋盘路径保留开发 `AddSun`；正常切片拒绝该命令。暂停冻结模拟与可冻结表现时钟，渲染帧计数继续推进，控制因果见 [开发切片合同](../../docs/behavior/development-slice.md)。

M 启停已加载的开发循环音乐；E 音效键仅在 probe/音频冒烟路径启用，原创 probe 没有加载音频。每帧按模拟执行后的最终暂停状态更新音乐的 `PlaybackSettings.paused`，尚未建立的 sink 因而按正确状态开始；现成 sink 通过 Bevy `AudioSink::pause/play` 保留实际游标。暂停、恢复与重开传播不依赖 sink 创建时机或单次事件，同批 P→M、M→P 均服从 core 的执行结果，终局拒绝暂停不会提前改变音乐。`--audio-smoke` 要求登记资源模式且至少 120 帧：提交一次音效和循环音乐，比较暂停期间两次游标，再确认恢复推进并停止。这个探针不等于人工听辨或完整曲尾循环验收。浏览器须通过用户手势解锁音频，并用实际游标验证，不能由 Web 构建替代。

Web 入口先编译 WebAssembly 模块，再等待用户点击“启动”按钮，在该手势内调用 wasm-bindgen 生成的异步 `init({ module_or_path: module })` 初始化应用和音频输出。自动启动曾实测 AudioContext 悬停，sink 存在而游标始终为零；当前入口的实际验证与限制见 [平台探针结果](../../docs/behavior/platform-probe-results.md)，构建命令见 [Web 说明](../../web/README.md)。

## 仿射与开发动画合同

纯 `PreparedAnimation` 在装配时继承缺省字段，保留显式空图片，源轨道顺序作为图层顺序；marker 取首个非负 `f` 连续区段，按资源 fps 离散循环，不反向驱动模拟。该采样能力已由向日葵 idle 和普通僵尸 walk 的真实 Scene 消费。PeaShooter 选择 `anim_full_idle`（79..104）的帧 79，包含头、嘴、叶与茎；`anim_idle`（4..29）只启用叶与茎。`kx` 控制 y 基向量、`ky` 控制 x 基向量，分别生成 `[-sy*sin(kx), sy*cos(kx)]` 与 `[sx*cos(ky), sx*sin(ky)]`。独立角度、缩放和平移使用原创夹具验证；原版完整插值、锚点、marker、sprite atlas 和 `f` 语义仍待测量。其他构图的来源与轨道选择见开发切片合同。

Bevy 世界坐标 Y 向上，纯逻辑坐标 Y 向下；适配层执行 `(x-400, 300-y)`。Camera viewport 使用 framebuffer 像素，正交投影覆盖 800×600 逻辑单位，直接在实际 framebuffer 绘制。纹理顶点经 CPU 凸多边形裁剪，同时插值 UV，再送入 `Mesh2d/ColorMaterial`；没有低分辨率渲染纹理作为中间输出。右侧红蓝原创四边形验证独立错切、图层遮挡和右侧裁剪。1400×600 背景使用 800×600 开发裁剪，不横向挤压。

`properties/resources.xml` 实查没有 PeaShooter 图像登记。`resources.rs` 当前保存 93 项经 Reanim 引用与清单逐项核对的显式图像映射，另加载豌豆图像，共 94 张；每个引用必须精确匹配唯一登记文件。运行时不使用大小写或文件名猜测，该映射不宣称复现原版资源管理系统。

普通僵尸 `ZombieWalk` 装配完整 `Zombie.reanim`，选资源 `anim_walk` 半开段 `44..91`、12fps，保留19条身体源轨道并排除附件；全映射/去重解码后图像集合仍为94张，没有旧 Zombie 静态副本。Scene 共享唯一缓存，按各僵尸当前位置借用采样并复用网格/材质槽。全局相位、离散采样、新出生接当前相位及碰植物仍持续 walk 均为开发合同；移除直接隐藏旧槽，没有死亡伪动画。暂停/双终局/重开和实际 main update 已有 CPU 回归；原批次验收范围见[本批结果](../../docs/behavior/content-foundation-results.md#2026-10-14-普通僵尸动态行走增量)，Windows 正常窗口与 Web 诊断缺项由[第一阶段集中实机任务](../../docs/superpowers/plans/2026-10-17-phase-1-day-combat.md#任务-5集中实机测试修复与调整)承接。

## 测试与性能入口

```sh
cargo test -p pvz-app --locked
cargo check -p pvz-app --all-targets --all-features --locked
cargo clippy -p pvz-app --all-targets --all-features --locked -- -D warnings
cargo bench -p pvz-app --bench presentation --locked
```

纯 lib 测试不创建窗口、GPU 或音频设备，默认并行；隔离临时目录验证缺失和脱敏错误，登记资源测试校验并实际解码六类构图所需集合。内容回归验证默认入口、实际参数消费、无回退错误边界、一 MiB 字节上限、30/120 Hz 推进、暂停/恢复、终局和重开。binary 回归在无设备的 Bevy World 中驱动真实 `update` 与播放请求组件，覆盖同批暂停/音乐顺序、sink 多帧尚未建立时的暂停、恢复、重开及终局拒绝；请求状态验证不代表后端已创建 sink 或实测游标。Criterion 覆盖 4K 输入映射、双轴仿射/裁剪、原创 8 轨道×128 帧和 12 fps 动画时间索引；这些表现场景的构造/解析位于计时区外。新增 schema 1/规则 2.0 原创白天 JSON 场景明确标注字节数，计时包含解析、校验与 Runtime 装配，嵌入输入且不计文件 I/O。新增普通僵尸原创42轨道×504帧夹具，测完整19身体借用采样以及1/10只的仿射裁剪；prepare、映射/上传、Bevy池与GPU不计入计时。测试覆盖源索引、空图/隐藏/短轨道、错误归属、真实两只僵尸坐标、池复用/材质换绑、移除尾槽、裁剪及暂停/双终局/重开回放。基准不读取或内嵌原版素材。GPU、设备输入与播放游标需要平台运行证据，CPU 基准不代表整体帧率。本批内容迁移的实际结果与待验范围见[版本化内容结果](../../docs/behavior/content-foundation-results.md)，历史平台/切片验收保留在[切片结果](../../docs/behavior/vertical-slice-results.md)。

## 平台与依赖边界

| 平台 | 当前接口路线 | 必须补充的目标证据 |
| --- | --- | --- |
| Windows | Bevy/winit/wgpu、固定仓库根只读资源、rodio/cpal | 构建、隐藏窗口截图、输入、DPI、设备音频 |
| macOS | winit/wgpu 平台后端和 cpal | 目标构建、窗口/DPI、音频、打包 |
| Linux | 当前显式 X11 feature、winit/wgpu 和 cpal | X11/音频系统依赖、目标构建与实际运行；Wayland 未注册 |
| Web | wasm-bindgen、WebGL2、gloo-net、浏览器音频 | 同源下载、浏览器启动、resize/DPI、鼠标/触摸、用户手势音频 |
| Android | 保持纯 lib 和资源字节接口，候选 winit/wgpu/cpal | Activity feature/入口、资源包、触摸、生命周期、构建与实机 |
| iOS | 保持纯 lib 和资源字节接口，候选 winit/wgpu/cpal | Xcode/macOS、应用入口与打包、触摸/生命周期、构建与实机 |

Bevy、image、lewton、gloo-net、clap 与 serde_json 的直接许可均为 MIT/Apache-2.0 系列，传递许可及公告由锁文件对应的仓库 `cargo deny` 门槛评估；serde_json 用于 native/Web 相同的纯内存内容解析，不引入文件或平台入口。原版资源与 SIL OFL 1.1 中文字体保持独立来源/许可。Bevy 的完整音频路径需目标平台设备，OGG 在进入后端前用 lewton 全量预解码。MO3 明确返回未支持诊断：将来需要独立的 MO3 解包和 tracker 模块播放器、许可与六平台评估，当前不把 MO3 交给 OGG 后端。可编译和运行支持状态分别记录，未测的平台不声明支持。

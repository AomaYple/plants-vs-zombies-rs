# pvz-app

Bevy 0.20 验证应用，当前消费者包括纯视口/输入合同、资源清单校验、开发动画、2D 仿射网格和音频探针。它验证跨平台基础设施；800×600、模拟 60 tick/s、背景 x=220 裁剪和动画 marker 解释均为开发合同，尚不是原版游戏行为验收。

## 运行与探针

所有命令从仓库根执行。默认读取 `assets/manifest.json` 登记的背景、PeaShooter 文本与 19 张 PNG、`seedlift.ogg` 音效和 `ZombiesOnYourLawn.ogg` 音乐；先检查大小/SHA-256/文本规范，再解码。native 使用 `AssetRoot` 只读输入，Web 使用 gloo-net 同源 `/assets/` 下载并复用同一清单校验。没有安装目录或资源根覆盖参数。缺失、损坏、未登记图像引用与下载失败会显示中文错误，不会静默变成占位资源。

```sh
cargo run -p pvz-app --locked
cargo run -p pvz-app --locked -- --probe
cargo run -p pvz-app --locked -- --probe --smoke 120 --screenshot
cargo run -p pvz-app --locked -- --smoke 160 --audio-smoke --screenshot
```

`--probe` 只显示原创背景、网格与仿射夹具，不读取原版素材或音频。`--smoke` 创建不可见且不请求焦点的窗口，持续更新，在指定真实绘制帧数后退出；`--screenshot` 等 GPU 截图回调完成，写入忽略目录 `artifacts/viewport/`，保存失败或资源失败返回错误。应用内加载等待上限为 30 秒。后台运行与截图验收由主代理统一协调。

鼠标与触摸坐标先乘窗口 DPI，再逆变换成逻辑点。视口外按下不产生游戏命令；捕获的指针越界、取消、失焦或视口变化会取消，多个指针状态相互独立。逻辑按下在未暂停时向 core 提交一个开发 `AddSun` 命令，模拟只消费 core 的状态/事件。P 或空格冻结模拟与动画，渲染计数继续推进。

E 播放一次音效，M 启停循环音乐。P/空格在音乐已播放时使用 Bevy `AudioSink::pause/play` 保留游标；UI 显示实际播放位置。`--audio-smoke` 要求登记资源模式且至少 120 帧：提交一次音效和循环音乐，比较暂停期间两次游标，再确认恢复推进并停止。这个探针验证音频后端和播放请求，不等于人工听辨声音质量，也不单独证明已经跨过曲目末尾的循环边界。浏览器还需要用户手势解锁音频，是否成功须用目标浏览器实测，不能由 Web 构建代替。

Web 入口先编译 WebAssembly 模块，再等待用户点击“启动”按钮，在该手势内调用 wasm-bindgen 生成的异步 `init({ module_or_path: module })` 初始化应用和音频输出。这样保留浏览器用户手势要求；直接自动启动应用可能让初始化的 AudioContext 悬停，此时音乐 sink 存在也可能始终为零。主代理已在实际浏览器测得播放至 26.19s、暂停保持、恢复至 59.81s；该证据只覆盖本次浏览器，其他浏览器和移动端仍需验收。

## 仿射与开发动画合同

纯 `PreparedAnimation` 在装配时继承缺省字段，保留显式空图片，源轨道顺序作为图层顺序；marker 取首个非负 `f` 连续区段，按资源 fps 离散循环，不反向驱动模拟。登记 PeaShooter 的开发默认选择 `anim_full_idle`（79..104），这个 XML 区段包含头、嘴、叶与茎；`anim_idle`（4..29）只启用叶与茎，不适合作为完整植物探针。`kx` 控制 y 基向量、`ky` 控制 x 基向量，分别生成 `[-sy*sin(kx), sy*cos(kx)]` 与 `[sx*cos(ky), sx*sin(ky)]`。独立角度、缩放和平移使用原创夹具验证。原版完整的插值、锚点、marker、sprite atlas 和 `f` 语义仍待原版测量。

Bevy 世界坐标 Y 向上，纯逻辑坐标 Y 向下；适配层执行 `(x-400, 300-y)`。Camera viewport 使用 framebuffer 像素，正交投影覆盖 800×600 逻辑单位，直接在实际 framebuffer 绘制。纹理顶点经 CPU 凸多边形裁剪，同时插值 UV，再送入 `Mesh2d/ColorMaterial`；没有低分辨率渲染纹理作为中间输出。右侧红蓝原创四边形验证独立错切、图层遮挡和右侧裁剪。1400×600 背景使用 800×600 开发裁剪，不横向挤压。

`properties/resources.xml` 实查没有 PeaShooter 图像登记，因此 `resources.rs` 保存 19 项经清单逐项核对的显式开发 ID 映射；每个动画引用必须精确匹配唯一登记文件。运行时不使用大小写或文件名猜测。这个映射尚不宣称复现原版资源管理系统。

## 测试与性能入口

```sh
cargo test -p pvz-app --locked
cargo check -p pvz-app --all-targets --all-features --locked
cargo clippy -p pvz-app --all-targets --all-features --locked -- -D warnings
cargo bench -p pvz-app --bench presentation --locked
```

纯 lib 测试不创建窗口、GPU 或音频设备，默认并行；隔离临时目录验证缺失和脱敏错误，登记资源测试校验并实际解码默认最小集合。Criterion 覆盖 4K 视口输入映射、两个角度的仿射/裁剪、原创 8 轨道×128 帧、12fps 开发动画的时间索引；原创稀疏夹具的构造/解析位于计时区外，基准不读取或嵌入原版素材。GPU 每帧网格更新、驱动、设备输入与播放游标需要平台运行证据，CPU 基准不代表整体帧率。

## 平台与依赖边界

| 平台 | 当前接口路线 | 必须补充的目标证据 |
| --- | --- | --- |
| Windows | Bevy/winit/wgpu、固定仓库根只读资源、rodio/cpal | 构建、隐藏窗口截图、输入、DPI、设备音频 |
| macOS | winit/wgpu 平台后端和 cpal | 目标构建、窗口/DPI、音频、打包 |
| Linux | 当前显式 X11 feature、winit/wgpu 和 cpal | X11/音频系统依赖、目标构建与实际运行；Wayland 未注册 |
| Web | wasm-bindgen、WebGL2、gloo-net、浏览器音频 | 同源下载、浏览器启动、resize/DPI、鼠标/触摸、用户手势音频 |
| Android | 保持纯 lib 和资源字节接口，候选 winit/wgpu/cpal | Activity feature/入口、资源包、触摸、生命周期、构建与实机 |
| iOS | 保持纯 lib 和资源字节接口，候选 winit/wgpu/cpal | Xcode/macOS、应用入口与打包、触摸/生命周期、构建与实机 |

Bevy、image、lewton、gloo-net 与 clap 的直接许可均为 MIT/Apache-2.0 系列，传递许可及公告由锁文件对应的仓库 `cargo deny` 门槛评估；原版资源与 SIL OFL 1.1 中文字体保持独立来源/许可。Bevy 的完整音频路径需目标平台设备，OGG 在进入后端前用 lewton 全量预解码。MO3 明确返回未支持诊断：将来需要独立的 MO3 解包和 tracker 模块播放器、许可与六平台评估，当前不把 MO3 交给 OGG 后端。可编译和运行支持状态分别记录，未测的平台不声明支持。

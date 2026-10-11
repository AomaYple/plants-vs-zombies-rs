# Web 开发入口

此入口消费 Bevy 应用和与锁文件匹配的 wasm-bindgen 生成包；不使用原版程序。默认模式读取仓库清单和登记资源，`?probe=1` 明确选择无原版资源的探针构建。

在仓库根目录执行：

```sh
cargo build -p pvz-app --target wasm32-unknown-unknown --release --locked
wasm-bindgen --target web --out-name pvz_app --out-dir target/web target/wasm32-unknown-unknown/release/pvz-app.wasm
cargo build -p pvz-app --target wasm32-unknown-unknown --release --features probe --locked
wasm-bindgen --target web --out-name pvz_app --out-dir target/web-probe target/wasm32-unknown-unknown/release/pvz-app.wasm
python -m http.server 8000 --bind 127.0.0.1
```

打开 `http://127.0.0.1:8000/web/`；无资源探针使用 `http://127.0.0.1:8000/web/?probe=1`。服务从仓库根目录提供资源与忽略的构建产物，仅监听本机回环地址；结束后关闭本次服务。代理验证优先使用隐藏浏览器，不切换维护者的前台窗口。

`wasm-bindgen-cli` 不是 Cargo 内建命令；安装版本必须与 `Cargo.lock` 中的 `wasm-bindgen` 精确匹配。生成的 JS/WASM 放在被忽略的 `target/`，不在源码中手写引擎 loader。升级依赖时重新生成并验证导入、音频及输入行为。

浏览器实际运行与 wasm 构建分别验收。音频需要用户手势；MO3 原版音乐暂未接入，不能将 OGG 探针播放成功当作 MO3 验收。

页面先下载并使用浏览器标准 `WebAssembly.compile` 准备模块；“启动开发应用”按钮在用户手势后调用 wasm-bindgen 生成的异步初始化接口。应用和 Bevy 音频设备在这时创建，避免页面加载时先创建悬停的 AudioContext。同步 `initSync` 实测受 Chromium 大模块主线程限制，因此不采用，也不要求修改浏览器参数。启动失败显式显示中文错误。实际验证见 [平台探针结果](../docs/behavior/platform-probe-results.md)。

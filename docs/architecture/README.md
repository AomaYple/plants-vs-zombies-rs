# 模块与依赖

本目录描述模块合同。当前已注册 `pvz-data`、`pvz-core` 和 `pvz-assets`，其余 package 按实施计划接入；实际成员以根 `Cargo.toml` 为准。运行时分层、行为还原和平台边界的长期约束见 [游戏开发原则](../principles/game-development.md)。

## 依赖方向

```text
pvz-app ---------> pvz-core -------> pvz-data
   |                                    ^
   +-------------> pvz-assets ----------+

pvz-asset-tool --> pvz-assets
```

箭头表示“依赖”。pvz-data 是独立的类型与数据定义层，没有引擎依赖。
pvz-core 与 pvz-assets 不互相依赖；资源解析结果由 app 装配，不在模拟里访问资源。
pvz-asset-tool 是宿主机 CLI，不加入 wasm 应用的依赖图。
serde 等依赖需在实际采用时说明用途；文件读取、格式解析和转换归 assets/tool，不能放入 data 或 core。

## crate 职责

| crate | 提供 | 不承担 |
| --- | --- | --- |
| pvz-data | 实体定义、关卡/波次 schema、内部 ID、版本和校验规则 | 文件 I/O、贴图、设备、原版文本内嵌 |
| pvz-core | 棋盘、实体、状态机、碰撞、规则更新、RNG、逻辑事件与回放 | 渲染、音频、资产加载、墙钟时间 |
| pvz-assets | 清单/Reanim/粒子解析、资源索引、CPU 数据和加载报告 | 游戏伤害、波次决策、GPU 纹理句柄 |
| pvz-app | 窗口、平台输入、资源装配、Macroquad 渲染/音频和存档 | 隐藏的第二套游戏规则 |
| pvz-asset-tool | 版本化资源检查、报告、转换、打包 | 运行原版、修改 core 规则、自动生成未登记的公开资源 |

## 数据流

平台输入 → 逻辑坐标映射 → 带顺序的游戏命令 → 固定 tick 模拟 → 状态/事件 → 动画与视听表现。
游戏事件包含种植成功、扣费、发射、伤害、产阳光、死亡、胜负；声音与特效消费事件，不反向决定伤害。
文件/包/浏览器字节输入 → assets 解析 → app 资源缓存 → 渲染和音频。
单元测试直接向 core 提交命令，无需窗口、资源包或系统路径。

## pvz-app 内部模块

- input：鼠标、触摸、取消与暂停映射，维护物理与逻辑坐标变换。
- render：有序绘制、Reanim 仿射矩阵、纹理/图集、粒子和裁剪。
- audio：音效去重、音乐循环与暂停恢复；单独验证 MO3 路线。
- platform：资源读取、存档介质、应用生命周期；宿主文件系统与 Web storage 分开实现。
- bootstrap：版本化资源装配、资源验证报告和加载失败界面。

独立 crate 的拆分条件是已有稳定接口、独立消费者或独立测试需要。不会仅为“以后可能”预建更多空 package。

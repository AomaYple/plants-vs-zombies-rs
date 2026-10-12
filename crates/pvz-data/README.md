# pvz-data

领域数据 crate：版本化内部 ID、schema 版本、规则版本、原版参考基线 ID 和白天关卡定义。

本 crate 只提供可序列化的纯数据类型，不读取文件、不访问资源、不创建窗口，也不包含游戏行为。标识符在构造和反序列化时都会校验格式；资源路径必须是相对的规范斜杠路径。

`DayLevelDefinition`、`DayRulesDefinition`、`DaySpawnDefinition` 和 `ContentBasis` 定义严格的内容格式：对象拒绝未知、遗漏和重复字段，整数按声明类型解析，不提供隐藏遗漏的默认值。`basis` 当前只接受 `development`。规则版本文本遵循现有版本语法；核心负责判断支持的规则版本、频率范围和游戏规则有效性，data 层不执行或重复验证游戏规则。

真实原创关卡位于 [development-day.json](../../content/levels/development-day.json)，来源与使用约定见 [内容说明](../../content/README.md)。集成测试直接读取该文件，锁定全部默认参数、有序波次、严格解析和稳定往返。

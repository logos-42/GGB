# 快速开始指南

## 立即开始测试多节点协同训练

### 步骤 1: 编译项目

```bash
cargo build --release
```

### 步骤 2: 启动多节点测试

**Windows:**
```powershell
.\scripts\test_multi_node.ps1 -Nodes 3 -Duration 300
```

这将：
- 启动 3 个节点
- 运行 5 分钟
- 自动收集统计数据到 `test_output/` 目录

### 步骤 3: 分析结果

```bash
cargo run --bin analyze_training -- --input test_output
```

### 步骤 4: 查看训练日志

训练过程中会输出：
- 节点发现信息
- 训练统计摘要（每 10 个 tick）
- 收敛度指标
- 网络拓扑状态

## 单节点运行

```bash
cargo run
```

或使用自定义配置：

```bash
# 模拟低端移动设备
$env:GGB_DEVICE_TYPE="low"
$env:GGB_NETWORK_TYPE="4g"
$env:GGB_BATTERY_LEVEL="0.5"
cargo run
```

## 查看实时统计

训练过程中，每 10 个 tick 会输出类似以下信息：

```
训练统计 [运行 120s, 12 ticks] | 连接: 3 节点 | 接收: 15 稀疏 + 2 密集 | 发送: 12 稀疏 + 1 密集 | 模型: v45 (0x1a2b3c4d...)
  收敛度: 0.623 | 参数变化: 0.000234 | 标准差: 0.045678
```

## 导出统计数据

使用 `--stats-output` 参数：

```bash
cargo run -- --stats-output stats.json
```

统计数据会每 30 秒自动导出到指定文件。

## 下一步

- 阅读 [测试指南](TESTING.md) 了解详细测试场景
- 查看 [Android 集成指南](../android/README.md) 了解移动端集成
- 查看 [iOS 集成指南](../ios/README.md) 了解 iOS 集成


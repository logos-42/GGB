# GGB 去中心化训练节点 - 运行指南

## 🚀 快速开始

### 1. 单节点运行（测试）
```bash
cargo run
```
- 默认随机地理位置
- 128维模型
- 自动检测设备能力

### 2. 指定节点ID（多节点协作）
```bash
# 第一台电脑
cargo run -- --node-id 0

# 第二台电脑  
cargo run -- --node-id 1

# 第三台电脑
cargo run -- --node-id 2
```

**自动配置**：
- 端口：9234 + node_id
- Bootstrap：自动连接其他节点的本地地址

### 3. 手动配置网络（不同电脑）
```bash
# 电脑A (192.168.1.100)
cargo run -- --node-id 0 --quic-port 9234 --bootstrap 192.168.1.101:9235 --bootstrap 192.168.1.102:9236

# 电脑B (192.168.1.101)
cargo run -- --node-id 1 --quic-port 9235 --bootstrap 192.168.1.100:9234 --bootstrap 192.168.1.102:9236

# 电脑C (192.168.1.102)
cargo run -- --node-id 2 --quic-port 9236 --bootstrap 192.168.1.100:9234 --bootstrap 192.168.1.101:9235
```

## ⚙️ 可用参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `--node-id <ID>` | 节点ID (0,1,2...) | `--node-id 0` |
| `--quic-port <PORT>` | QUIC端口 | `--quic-port 9234` |
| `--bootstrap <IP:PORT>` | Bootstrap节点地址 | `--bootstrap 192.168.1.100:9234` |
| `--model-dim <DIM>` | 模型维度 | `--model-dim 512` |
| `--stats-output <FILE>` | 统计输出文件 | `--stats-output stats.json` |

## 🌐 网络发现机制

### 自动发现（同一局域网）
- **mDNS**：零配置，自动发现邻近节点
- **QUIC连接**：低延迟P2P通信
- **Kademlia DHT**：分布式节点发现

### 手动配置（不同网络）
- 指定其他节点的公网IP和端口
- 配置防火墙端口转发
- 可选：使用DDNS服务

## 📊 监控运行状态

### 控制台输出
- `[QUIC] 成功连接到 <IP>:<PORT>` - 连接建立
- `[mDNS] 发现节点 <peer_id>` - 节点发现
- `拓扑更新：xxx => sim 0.xxx` - 训练协作开始
- 每10个tick显示训练统计信息

### 统计数据导出
```bash
cargo run -- --node-id 0 --stats-output node_stats.json
```

## 🧪 测试工具

### 多节点自动化测试
```powershell
# Windows
.\scripts\test_multi_node.ps1 -Nodes 3 -Duration 300

# Linux/Mac
bash scripts/test_multi_node.sh --nodes 3 --duration 300
```

### 训练结果分析
```bash
cargo run --bin analyze_training -- --input test_output
```

## 🔧 设备自适应

系统自动根据设备能力调整：
- **内存**：调整模型维度
- **网络**：WiFi允许密集快照，移动网络仅稀疏更新
- **电池**：低电量时自动降低训练频率
- **CPU**：根据核心数调整并行度

## 🎯 验证成功运行

看到以下输出表示运行正常：
1. `启动 GGS 节点 => peer: xxx, eth: xxx, sol: xxx`
2. `[QUIC] 成功连接到 xxx` 或 `[mDNS] 发现节点 xxx`
3. 定期显示训练统计信息（收敛度、参数变化等）

## 🛠️ 故障排除

- **编译失败**：确保安装了Rust和系统依赖
- **连接失败**：检查防火墙设置和端口可用性
- **发现延迟**：mDNS和DHT需要几秒到几十秒建立连接
- **性能问题**：调整模型维度或训练频率
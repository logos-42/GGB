# P2P 模型分发使用指南

本指南介绍如何使用基于 iroh 的 P2P 模型分发系统，将已经切分好的模型分片分发给另一台电脑。

## 🚀 快速开始

### 前置条件

1. **确保模型已切分**：
   ```bash
   # 检查模型分片目录
   ls -la ./test_models/test_models/simple_split/
   ```

2. **编译项目**：
   ```bash
   cargo build --release --example p2p_model_distribution_demo
   ```

### 基本使用

#### 方法1: 运行完整演示（推荐）

```bash
# 自动运行发送端和接收端的完整测试
cargo run --release --example p2p_model_distribution_demo -- full \
    --demo-dir "./demo_output" \
    --shard-dir "./test_models/test_models/simple_split" \
    --sender-port 9235 \
    --receiver-port 9236
```

#### 方法2: 手动启动两端

**步骤1: 启动接收端（在目标电脑上）**
```bash
# 创建接收目录
mkdir -p ./received_models

# 启动接收端
cargo run --release --example p2p_model_distribution_demo -- receive \
    --node-id "receiver_node" \
    --output-dir "./received_models" \
    --port 9236 \
    --auto-accept
```

**步骤2: 启动发送端（在源电脑上）**
```bash
# 发送模型分片
cargo run --release --example p2p_model_distribution_demo -- send \
    --node-id "sender_node" \
    --target-peer "receiver_node" \
    --shard-dir "./test_models/test_models/simple_split" \
    --chunk-size 1048576 \
    --port 9235
```

## 📋 详细参数说明

### 发送端参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--node-id` | 发送端节点ID | `demo_sender` |
| `--target-peer` | 目标接收端节点ID | 必需 |
| `--shard-dir` | 模型分片目录 | `./test_models/test_models/simple_split` |
| `--chunk-size` | 传输块大小（字节） | `1048576` (1MB) |
| `--port` | 监听端口 | `9235` |

### 接收端参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--node-id` | 接收端节点ID | `demo_receiver` |
| `--output-dir` | 接收文件输出目录 | `./received_models` |
| `--port` | 监听端口 | `9236` |
| `--auto-accept` | 自动接受传输请求 | `true` |
| `--max-concurrent` | 最大并发传输数 | `5` |

## 🔧 高级配置

### 自定义传输协议

可以通过修改 `TransferProtocolConfig` 来自定义传输行为：

```rust
let config = TransferProtocolConfig {
    max_chunk_size: 2 * 1024 * 1024, // 2MB 块大小
    max_retries: 5,                   // 最大重试次数
    timeout_seconds: 60,               // 超时时间
    enable_compression: true,         // 启用压缩
    enable_encryption: true,           // 启用加密
    checksum_algorithm: ChecksumAlgorithm::SHA256,
    resume_support: true,              // 支持断点续传
};
```

### 网络配置

如果需要通过 NAT 或防火墙，可以配置 bootstrap 节点：

```bash
# 使用 bootstrap 节点
cargo run --release --example p2p_model_distribution_demo -- send \
    --bootstrap "bootstrap_node:port" \
    [其他参数...]
```

## 🧪 测试验证

### 运行自动化测试

**Linux/Mac:**
```bash
chmod +x scripts/test_p2p_distribution.sh
./scripts/test_p2p_distribution.sh
```

**Windows:**
```powershell
.\scripts\test_p2p_distribution.ps1
```

### 文件完整性验证

```bash
# 验证特定文件的完整性
cargo run --release --example p2p_model_distribution_demo -- test-integrity \
    --file-path "./test_models/test_models/simple_split/node_001.json" \
    --algorithm sha256
```

## 📊 监控和调试

### 查看传输日志

发送端日志：
```bash
tail -f test_output/p2p_test_*/sender.log
```

接收端日志：
```bash
tail -f test_output/p2p_test_*/receiver.log
```

### 传输状态监控

系统会自动输出传输进度：
```
📊 传输进度: 25.0% (5/20)
📊 传输进度: 50.0% (10/20)
📊 传输进度: 75.0% (15/20)
✅ 传输完成: file_id_xyz
```

## 🔒 安全特性

### 文件完整性校验

- **SHA256 哈希校验**：确保文件传输完整性
- **块级验证**：每个数据块都有独立的哈希校验
- **最终验证**：文件组装后进行完整性验证

### 加密传输（可选）

```rust
// 启用端到端加密
let config = TransferProtocolConfig {
    enable_encryption: true,
    // ... 其他配置
};
```

## 🚨 故障排除

### 常见问题

1. **连接失败**
   ```bash
   # 检查端口是否被占用
   netstat -an | grep 9235
   netstat -an | grep 9236
   ```

2. **传输速度慢**
   ```bash
   # 调整块大小
   --chunk-size 2097152  # 2MB
   ```

3. **内存不足**
   ```bash
   # 减少并发数
   --max-concurrent 2
   ```

### 调试模式

启用详细日志：
```bash
RUST_LOG=debug cargo run --release --example p2p_model_distribution_demo [命令...]
```

## 📈 性能优化

### 网络优化

1. **调整块大小**：
   - 快速网络：2MB 或更大
   - 慢速网络：512KB 或更小

2. **并发传输**：
   - 高性能设备：10-20 个并发
   - 低性能设备：2-5 个并发

### 存储优化

1. **SSD 存储**：使用 SSD 提高写入速度
2. **内存缓存**：适当增加块大小减少 I/O

## 🌐 网络部署

### 局域网部署

在同一局域网内，直接使用 IP 地址连接：

```bash
# 发送端
cargo run --release --example p2p_model_distribution_demo -- send \
    --target-peer "192.168.1.100:9236" \
    [其他参数...]
```

### 广域网部署

需要配置 NAT 穿透或使用中继节点：

```bash
# 使用中继节点
cargo run --release --example p2p_model_distribution_demo -- send \
    --bootstrap "relay.example.com:8080" \
    [其他参数...]
```

## 📚 API 参考

### 核心组件

- **P2PModelDistributor**：核心分发器
- **P2PModelSender**：发送端实现
- **P2PModelReceiver**：接收端实现
- **FileTransferProtocol**：传输协议
- **FileIntegrity**：文件完整性管理

### 消息类型

```rust
pub enum FileTransferMessage {
    FileRequest { ... },      // 文件传输请求
    FileResponse { ... },     // 文件传输响应
    FileChunk { ... },        // 文件数据块
    FileComplete { ... },     // 传输完成
    ProgressReport { ... },   // 进度报告
    TransferError { ... },    // 传输错误
}
```

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request 来改进 P2P 模型分发系统！

### 开发环境设置

```bash
# 安装开发依赖
cargo install cargo-watch

# 运行开发模式
cargo watch -x "run --example p2p_model_distribution_demo"
```

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](../../LICENSE) 文件。

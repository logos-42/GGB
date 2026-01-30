# iroh P2P 本地通信指南

本指南提供了多个iroh P2P通信的实现示例，专门解决本地环境中的连接问题。

## 🚀 快速开始

### 方法1: 自动化测试（推荐）

运行完整的测试套件：

```powershell
# 运行所有测试
.\scripts\run_iroh_tests.ps1

# 运行特定测试
.\scripts\run_iroh_tests.ps1 -TestType simple
.\scripts\run_iroh_tests.ps1 -TestType robust
.\scripts\run_iroh_tests.ps1 -TestType demo

# 自定义消息
.\scripts\run_iroh_tests.ps1 -Message "你好，iroh！"
```

### 方法2: 手动测试

#### 简单版本测试

1. **启动接收端**（终端1）：
```bash
cargo run --example iroh_simple_local -- receive
```

2. **发送消息**（终端2）：
```bash
# 复制接收端显示的节点ID，替换下面的<节点ID>
cargo run --example iroh_simple_local -- send --target <节点ID>
```

#### 健壮版本测试

1. **启动接收端**（终端1）：
```bash
cargo run --example iroh_robust_local -- receive --port 11206
```

2. **发送消息**（终端2）：
```bash
cargo run --example iroh_robust_local -- send --target <节点ID> --port 11206 --message "Hello robust iroh!"
```

#### 演示版本测试

1. **启动接收端**（终端1）：
```bash
cargo run --example iroh_local_demo -- receive --port 11204
```

2. **发送消息**（终端2）：
```bash
cargo run --example iroh_local_demo -- send --target <节点ID> --addr 127.0.0.1:11204 --message "Hello demo!"
```

## 📁 示例文件说明

### 1. `iroh_simple_local.rs`
- **特点**: 最简化的实现
- **端口**: 固定使用11205
- **适用**: 快速测试和学习
- **优点**: 代码简洁，易于理解

### 2. `iroh_robust_local.rs`
- **特点**: 包含详细错误处理和重试机制
- **端口**: 可配置（默认11206）
- **适用**: 生产环境或不稳定网络
- **优点**: 健壮性强，调试信息丰富

### 3. `iroh_local_demo.rs`
- **特点**: 完整的演示版本，支持双向通信
- **端口**: 可配置（默认11204）
- **适用**: 完整功能演示
- **优点**: 功能完整，包含回复机制

## 🔧 配置说明

### 依赖配置

确保 `Cargo.toml` 中包含正确的iroh依赖：

```toml
[dependencies]
iroh = { version = "0.95", features = ["discovery-local-network"] }
tokio = { version = "1", features = ["rt", "time", "sync"] }
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 网络配置

- **本地环回**: 所有示例都使用 `127.0.0.1` (localhost)
- **端口范围**: 11204-11206，避免与其他服务冲突
- **发现机制**: 启用本地网络发现 (`discovery_local_network()`)

## 🐛 故障排除

### 常见问题

1. **连接超时**
   - 确保接收端完全启动后再发送消息
   - 检查防火墙设置
   - 尝试增加重试次数

2. **节点ID无效**
   - 确保完整复制节点ID
   - 检查ID格式是否正确（z32编码）

3. **端口占用**
   - 使用不同的端口号
   - 检查是否有其他程序占用端口

4. **构建失败**
   - 更新Rust版本：`rustup update`
   - 清理构建缓存：`cargo clean`

### 调试模式

使用健壮版本的调试模式获取详细信息：

```bash
cargo run --example iroh_robust_local -- --debug receive --port 11206
cargo run --example iroh_robust_local -- --debug send --target <节点ID> --port 11206
```

## 📊 性能测试

### 基准测试

测试不同消息大小的传输性能：

```bash
# 小消息
cargo run --example iroh_robust_local -- send --target <节点ID> --message "小消息测试"

# 大消息
cargo run --example iroh_robust_local -- send --target <节点ID> --message "$(python -c 'print("A" * 1000)')"
```

### 连接稳定性测试

使用重试机制测试连接稳定性：

```bash
cargo run --example iroh_robust_local -- send --target <节点ID> --retries 10
```

## 🔄 扩展开发

### 添加新功能

1. **文件传输**: 扩展消息处理支持二进制数据
2. **多节点通信**: 支持一对多或多对多通信
3. **加密通信**: 添加端到端加密
4. **持久连接**: 保持长连接进行多次通信

### 集成到项目

将iroh P2P功能集成到现有项目：

```rust
use iroh::{Endpoint, EndpointAddr, PublicKey};

// 创建P2P通信模块
pub struct P2PManager {
    endpoint: Endpoint,
}

impl P2PManager {
    pub async fn new() -> Result<Self> {
        let endpoint = Endpoint::builder()
            .alpns(vec![b"my-app".to_vec()])
            .discovery_local_network()
            .bind()
            .await?;
        
        Ok(Self { endpoint })
    }
    
    pub async fn send_message(&self, target: &str, message: &str) -> Result<()> {
        // 实现消息发送逻辑
        todo!()
    }
}
```

## 📚 参考资料

- [iroh官方文档](https://docs.rs/iroh/)
- [iroh GitHub仓库](https://github.com/n0-computer/iroh)
- [QUIC协议介绍](https://quicwg.org/)
- [P2P网络原理](https://en.wikipedia.org/wiki/Peer-to-peer)

## 🤝 贡献

如果你发现问题或有改进建议，请：

1. 创建Issue描述问题
2. 提交Pull Request
3. 更新文档和测试

---

**注意**: 这些示例专门针对本地开发和测试环境。在生产环境中使用时，请考虑安全性、网络配置和错误处理等因素。
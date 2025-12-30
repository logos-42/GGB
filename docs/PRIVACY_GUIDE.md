# GGB 隐私保护指南

## 概述

本文档介绍如何使用 GGB 的隐私保护功能，包括 IP 隐藏、中继网络、流量混淆和身份保护。

## 核心功能

### 1. IP 隐藏
- **功能**: 完全隐藏节点的真实 IP 地址
- **实现**: 通过中继网络转发所有流量
- **配置**: `hide_ip = true`

### 2. 中继网络
- **功能**: 使用中继节点转发连接
- **实现**: libp2p 中继协议
- **配置**: `use_relay = true`, `relay_nodes = [...]`

### 3. 流量混淆
- **功能**: 添加随机填充数据，混淆流量特征
- **实现**: 随机长度填充 + 定期更换模式
- **配置**: 自动启用（当 `hide_ip = true` 时）

### 4. 身份保护
- **功能**: 定期更换 PeerId，防止长期追踪
- **实现**: 临时身份生成 + 历史管理
- **配置**: 自动启用（当 `hide_ip = true` 时）

## 快速开始

### 步骤 1: 创建配置文件

```bash
# 复制示例配置
cp config/privacy_example.toml config/security.toml

# 编辑配置文件
# 1. 设置 hide_ip = true
# 2. 添加可用的中继节点地址
# 3. 根据需求调整其他参数
```

### 步骤 2: 配置中继节点

在 `config/security.toml` 中添加中继节点：

```toml
[security]
hide_ip = true
use_relay = true
relay_nodes = [
    "/ip4/中继IP/tcp/端口/p2p/中继PeerId",
    "/ip4/另一个中继IP/tcp/端口/p2p/另一个中继PeerId",
]
```

### 步骤 3: 启动隐私保护节点

**Linux/Mac:**
```bash
chmod +x scripts/start_private_node.sh
./scripts/start_private_node.sh
```

**Windows:**
```powershell
.\scripts\start_private_node.ps1
```

### 步骤 4: 验证隐私保护

检查启动日志：
```
[安全] 启用中继网络隐藏IP
[安全] 禁用公共DHT保护IP隐私
[中继] 尝试连接到中继节点: ...
[中继] 已添加中继节点: ...
[身份保护] 生成新的临时PeerId: ...
```

## 配置详解

### 安全配置 (`[security]`)

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `hide_ip` | bool | `false` | 是否隐藏 IP 地址 |
| `use_relay` | bool | `false` | 是否使用中继网络 |
| `relay_nodes` | 数组 | `[]` | 中继节点地址列表 |
| `private_network_key` | string | `null` | 私有网络密钥（可选） |
| `max_hops` | u8 | `3` | 最大中继跳数 (1-5) |
| `enable_autonat` | bool | `true` | 启用自动 NAT 穿透 |
| `enable_dcutr` | bool | `true` | 启用直接连接升级 |

### 通信配置 (`[comms]`)

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `enable_dht` | bool | `true` | 启用公共 DHT（隐藏 IP 时应禁用） |
| `listen_addr` | string | 自动 | 监听地址（使用中继时可设为本地） |
| `topic` | string | `"ggb-training"` | 通信主题 |

## 隐私模式对比

### 公共模式（默认）
```toml
[security]
hide_ip = false
use_relay = false
enable_dcutr = true

[comms]
enable_dht = true
```
- **优点**: 连接速度快，延迟低
- **缺点**: IP 地址暴露，可被追踪
- **适用**: 测试环境、信任的网络

### 隐私模式
```toml
[security]
hide_ip = true
use_relay = true
enable_dcutr = false

[comms]
enable_dht = false
```
- **优点**: IP 完全隐藏，隐私性强
- **缺点**: 连接速度较慢，依赖中继节点
- **适用**: 生产环境、公共网络

### 混合模式
```toml
[security]
hide_ip = true
use_relay = true
enable_dcutr = true  # 尝试直接连接

[comms]
enable_dht = false
```
- **优点**: 平衡隐私和性能
- **缺点**: DCUtR 可能暴露 IP
- **适用**: 需要平衡的场景

## 中继节点管理

### 公共中继节点
目前可用的公共中继节点有限，建议：

1. **自建中继节点**: 在云服务器上运行 GGB 中继
2. **社区中继**: 加入 GGB 社区获取可靠中继节点
3. **临时中继**: 在信任的设备间互相作为中继

### 自建中继节点

```bash
# 在中继服务器上
cargo run --release -- \
  --config relay_config.toml \
  --relay-mode true
```

中继配置示例 (`relay_config.toml`):
```toml
[security]
hide_ip = false  # 中继节点需要公网 IP
use_relay = false

[comms]
listen_addr = "/ip4/0.0.0.0/tcp/9001"
enable_dht = true
max_connections = 100  # 允许更多连接
```

## 隐私验证

### 检查 IP 暴露
```rust
use GGB::security::PrivacyChecker;

let checker = PrivacyChecker::new(config);
let addr = "/ip4/192.168.1.100/tcp/9001".parse().unwrap();
if checker.is_address_exposing_ip(&addr) {
    println!("警告: 地址暴露了 IP!");
}
```

### 测试隐私保护
```bash
# 运行隐私测试
cargo test --test security_test

# 运行演示
cargo run --example privacy_demo
```

## 故障排除

### 常见问题

1. **无法连接到中继节点**
   - 检查中继节点地址格式
   - 确认中继节点在线
   - 检查防火墙设置

2. **连接速度慢**
   - 增加中继节点数量
   - 选择地理位置近的中继
   - 调整 `max_hops`（值越小越快）

3. **IP 仍然暴露**
   - 确认 `hide_ip = true`
   - 确认 `enable_dht = false`
   - 检查日志中的警告信息

### 监控日志

关键日志信息：
- `[安全] 启用中继网络隐藏IP` - 隐私保护已启用
- `[安全] 禁用公共DHT保护IP隐私` - DHT 已禁用
- `[中继] 已添加中继节点` - 中继连接成功
- `[身份保护] 生成新的临时PeerId` - 身份保护工作正常
- `[警告]` - 需要关注的隐私问题

## 最佳实践

### 1. 多层保护
```toml
# 启用所有隐私功能
hide_ip = true
use_relay = true
private_network_key = "your-secret-key"
max_hops = 3
```

### 2. 定期更换
- 每小时更换一次 PeerId
- 每5分钟更换流量混淆模式
- 定期更换中继节点

### 3. 监控告警
```bash
# 监控隐私相关日志
tail -f node.log | grep -E "(安全|中继|身份保护|警告)"
```

### 4. 备份配置
- 备份中继节点列表
- 备份私有网络密钥
- 定期导出配置

## 安全注意事项

### 必须启用
- ✅ `hide_ip = true` (隐藏 IP)
- ✅ `use_relay = true` (使用中继)
- ✅ `enable_dht = false` (禁用公共 DHT)

### 建议禁用
- ⚠ `enable_dcutr = false` (防止直接连接)
- ⚠ 避免使用固定 PeerId

### 风险提示
1. **中继节点可信度**: 中继节点可能监控流量
2. **时序分析**: 流量模式可能被分析
3. **元数据泄露**: 连接时间、频率等可能泄露信息

## 高级配置

### 自定义流量混淆
```rust
use GGB::security::TrafficObfuscator;

let mut obfuscator = TrafficObfuscator::new(config);
obfuscator.set_padding_sizes(vec![256, 512, 1024]); // 自定义填充大小
obfuscator.set_rotation_interval(Duration::from_secs(600)); // 每10分钟更换
```

### 自定义身份保护
```rust
use GGB::security::IdentityProtector;

let protector = IdentityProtector::new(config);
protector.set_rotation_interval(Duration::from_secs(1800)); // 每30分钟更换
```

## 支持与反馈

如有问题或建议：
1. 查看日志文件 `node.log`
2. 运行测试 `cargo test --test security_test`
3. 提交 Issue 到项目仓库
4. 加入社区讨论

---

**记住**: 没有绝对的隐私，但通过合理配置可以显著降低风险。定期审查和更新安全配置是保持隐私的关键。

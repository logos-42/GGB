# GGB 故障排查指南

## 常见问题

### 节点发现问题

#### 症状
- 节点启动后 `connected_peers` 始终为 0
- 没有收到任何心跳或探测消息
- 日志中没有 `[mDNS] 发现节点` 消息

#### 解决方案

**Windows 系统：**

1. **启用 mDNS 服务**
   - 打开"服务"管理器
   - 找到"Bonjour"服务，确保已启动
   - 如果没有，安装 Apple Bonjour 服务

2. **配置防火墙**
   - 允许程序通过防火墙
   - 允许 UDP 5353 端口（mDNS）
   - 允许程序访问专用网络

3. **使用 DHT 作为备选**
   - 在配置中启用 DHT：`enable_dht: true`
   - 配置 bootstrap 节点文件

**Linux/Mac：**

1. 检查 `avahi-daemon` 是否运行（Linux）
2. 检查防火墙规则
3. 确认网络接口支持多播

**通用方案：**

使用手动 bootstrap 节点：

```rust
CommsConfig {
    enable_dht: true,
    bootstrap_peers_file: Some("bootstrap_peers.txt".into()),
    // ...
}
```

### 模型加载失败

#### 症状
```
Error: model file not found
Error: 模型维度不匹配
Error: 模型包含无效值
```

#### 解决方案

1. **检查文件路径**
   ```rust
   // 使用绝对路径或相对于工作目录的路径
   model_path: Some("examples/sample_model.npy".into())
   ```

2. **验证模型文件**
   ```rust
   use GGB::inference::validate_model_file;
   validate_model_file("model.npy", Some(256))?;
   ```

3. **创建示例模型**
   ```bash
   python scripts/create_sample_model.py 256 examples/sample_model.npy
   ```

### 内存不足

#### 症状
- 程序崩溃或 OOM 错误
- Top-K 值自动减少
- 性能下降

#### 解决方案

1. **降低模型维度**
   ```rust
   InferenceConfig {
       model_dim: 128,  // 从 256 降低到 128
       // ...
   }
   ```

2. **减少邻居数量**
   ```rust
   TopologyConfig {
       max_neighbors: 4,  // 从 8 降低到 4
       // ...
   }
   ```

3. **检查设备能力**
   ```rust
   let caps = device_manager.get();
   let recommended_dim = caps.recommended_model_dim();
   ```

### 网络连接问题

#### 症状
- QUIC 连接失败
- 消息发送失败
- 频繁的 failover

#### 解决方案

1. **检查网络类型**
   - WiFi 允许密集快照
   - 移动网络仅稀疏更新

2. **调整带宽预算**
   ```rust
   BandwidthBudgetConfig {
       sparse_per_window: 6,  // 降低稀疏更新频率
       dense_bytes_per_window: 128 * 1024,  // 降低密集快照大小
       // ...
   }
   ```

3. **检查 QUIC 端口**
   - 确保端口未被占用
   - 检查防火墙规则

### Android 集成问题

#### 症状
- `System.loadLibrary("ggb")` 失败
- JNI 调用崩溃
- 设备检测返回默认值

#### 解决方案

1. **检查库文件位置**
   - 确保 `.so` 文件在 `src/main/jniLibs/<abi>/` 目录
   - 检查 ABI 是否匹配

2. **检查 JNI 绑定**
   - 确认 `ggb_jni.cpp` 已编译
   - 检查 `CMakeLists.txt` 配置

3. **检查权限**
   - 确认已添加网络和电池权限
   - 检查运行时权限请求

### iOS 集成问题

#### 症状
- Framework 链接失败
- Swift 调用崩溃
- 设备检测失败

#### 解决方案

1. **检查 Framework**
   - 确认 XCFramework 包含所有架构
   - 检查 Framework Search Paths

2. **检查头文件**
   - 确认 `GGB.h` 已添加到项目
   - 检查 Bridging Header 配置

3. **检查权限**
   - 确认 Info.plist 包含网络使用说明
   - 检查电池监控权限

### Base 网络 RPC 问题

#### 症状
- RPC 调用失败
- 质押查询返回错误
- 交易发送失败

#### 解决方案

1. **检查 RPC 端点**
   ```rust
   // 使用公共 RPC 或配置自己的节点
   let client = BaseNetworkClient::new("https://mainnet.base.org".to_string());
   ```

2. **检查网络连接**
   - 确认可以访问 RPC 端点
   - 检查代理设置

3. **启用区块链功能**
   ```bash
   cargo build --release --features blockchain
   ```

## 调试技巧

### 启用详细日志

```rust
// 在代码中添加更多 println! 或使用日志库
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
```

### 网络诊断

```bash
# 检查端口是否监听
netstat -an | grep 9234

# 检查 mDNS
# Windows: 检查 Bonjour 服务
# Linux: systemctl status avahi-daemon
# Mac: 检查 mDNSResponder
```

### 性能分析

使用统计输出分析性能：

```bash
cargo run --release -- --stats-output stats.json
cargo run --bin analyze_training -- --input test_output
```

## 获取帮助

如果问题仍未解决：

1. 查看日志文件：`test_output/node_*.log`
2. 检查统计数据：`test_output/node_*_stats.json`
3. 提交 Issue 到 GitHub 仓库


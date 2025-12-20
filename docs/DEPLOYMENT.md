# GGB 部署指南

## 概述

本文档介绍如何部署 GGB 去中心化训练节点到不同环境。

## 桌面/服务器部署

### 系统要求

- Rust 1.70+
- 网络连接（用于 P2P 通信）
- 至少 512MB 可用内存

### 安装步骤

1. **克隆仓库**
```bash
git clone git@github.com:logos-42/GGB.git
cd GGB
```

2. **编译项目**
```bash
cargo build --release
```

3. **运行节点**
```bash
cargo run --release
```

### 配置选项

通过环境变量配置：

```bash
export GGB_DEVICE_TYPE=high        # low/mid/high
export GGB_NETWORK_TYPE=wifi       # wifi/4g/5g
export GGB_BATTERY_LEVEL=0.8
export GGB_BATTERY_CHARGING=true
```

通过命令行参数：

```bash
cargo run --release -- --model-dim 512 --quic-port 9234
```

## Android 部署

### 前置要求

- Android NDK
- Android Studio
- Rust Android 工具链

### 构建步骤

1. **安装 Rust Android 工具链**
```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

2. **构建 Rust 库**
```bash
# Windows
.\scripts\build_android.ps1

# Linux/Mac
bash scripts/build_android.sh
```

3. **集成到 Android 项目**

将生成的 `.so` 文件复制到 `android/src/main/jniLibs/` 目录。

在 `build.gradle` 中配置：

```gradle
android {
    ndk {
        abiFilters 'armeabi-v7a', 'arm64-v8a', 'x86', 'x86_64'
    }
}
```

4. **使用 Java API**

```java
import com.ggb.GgbNode;

GgbNode node = new GgbNode(context);
node.updateDeviceCapabilities();
String capabilities = node.getCapabilities();
```

### 权限配置

在 `AndroidManifest.xml` 中添加：

```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.BATTERY_STATS" />
```

## iOS 部署

### 前置要求

- Xcode
- Rust iOS 工具链
- CocoaPods (可选)

### 构建步骤

1. **安装 Rust iOS 工具链**
```bash
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios-sim
```

2. **构建 XCFramework**
```bash
bash scripts/build_ios.sh
```

3. **集成到 Xcode 项目**

- 将 `GGB.xcframework` 拖入 Xcode 项目
- 在 Build Settings 中配置 Framework Search Paths
- 链接 `GGB.framework`

4. **使用 Swift API**

```swift
import GGB

let node = GgbNode()
node.updateDeviceCapabilities()
if let capabilities = node.getCapabilities() {
    print(capabilities)
}
```

### 权限配置

在 `Info.plist` 中添加：

```xml
<key>NSLocalNetworkUsageDescription</key>
<string>用于 P2P 节点发现和通信</string>
```

## 网络配置

### 防火墙设置

- **TCP**: 允许监听端口（默认随机）
- **UDP**: 允许 mDNS (5353) 和 QUIC 端口（默认 9234+）
- **mDNS**: Windows 需要启用 Bonjour 服务

### Bootstrap 节点

创建 `bootstrap_peers.txt` 文件，每行一个节点地址：

```
/ip4/192.168.1.100/tcp/9001
/ip4/192.168.1.101/tcp/9002
```

在配置中指定：

```rust
CommsConfig {
    bootstrap_peers_file: Some("bootstrap_peers.txt".into()),
    // ...
}
```

## Base 网络集成

### 配置 RPC 端点

```rust
use GGB::blockchain::BaseNetworkClient;

let client = BaseNetworkClient::new("https://mainnet.base.org".to_string());
```

### 启用区块链功能

编译时启用 `blockchain` feature：

```bash
cargo build --release --features blockchain
```

## 监控和日志

### 训练统计

节点会自动输出训练统计信息：

```
训练统计 [运行 120s, 12 ticks] | 连接: 3 节点 | 接收: 15 稀疏 + 2 密集 | 发送: 12 稀疏 + 1 密集
  收敛度: 0.623 | 参数变化: 0.000234 | 标准差: 0.045678
```

### 导出统计数据

```bash
cargo run --release -- --stats-output stats.json
```

## 故障排查

### 节点无法发现彼此

1. 检查防火墙设置
2. 确认在同一网络
3. 检查 mDNS 是否启用
4. 使用 DHT bootstrap 节点

### 模型加载失败

1. 验证模型文件格式（必须是 .npy）
2. 检查模型维度是否匹配
3. 使用 `validate_model_file()` 验证

### 内存不足

1. 降低模型维度
2. 减少邻居数量
3. 启用内存压力检测

详细故障排查请参考 [TROUBLESHOOTING.md](TROUBLESHOOTING.md)


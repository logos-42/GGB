# iOS 集成指南

## 概述

本目录包含 iOS 平台的集成代码和配置。

## 目录结构

```
ios/
├── GGB/
│   ├── GGB.swift          # Swift 包装类
│   └── GGB.h              # Objective-C 头文件
├── GGB.xcodeproj/         # Xcode 项目
└── README.md
```

## 构建步骤

1. **安装 Rust iOS 工具链**
   ```bash
   rustup target add aarch64-apple-ios
   rustup target add armv7-apple-ios
   rustup target add x86_64-apple-ios
   rustup target add i386-apple-ios
   ```

2. **构建 Rust 库**
   ```bash
   # 在项目根目录
   cargo build --target aarch64-apple-ios --release
   cargo build --target x86_64-apple-ios --release  # 用于模拟器
   ```

3. **创建 XCFramework**
   ```bash
   # 使用 cargo-xcode 或手动创建
   # 将编译好的库打包为 .xcframework
   ```

4. **集成到 Xcode 项目**
   - 将 `.xcframework` 添加到 Xcode 项目
   - 在 Build Settings 中配置 Framework Search Paths
   - 链接 `GGB.framework`

## 使用示例

```swift
import GGB

// 创建节点实例
let node = GgbNode()

// 获取设备能力
if let capabilities = node.getCapabilities() {
    print("设备能力: \(capabilities)")
}

// 更新网络类型
node.updateNetworkType(.wifi)

// 更新电池状态
node.updateBattery(level: 0.75, isCharging: true)

// 启动训练
node.start()
```

## 注意事项

- 需要 iOS 12.0+
- 需要网络权限和后台运行权限
- 建议在后台任务中运行节点


# Android 集成指南

## 概述

本目录包含 Android 平台的集成代码和配置。

## 目录结构

```
android/
├── build.gradle          # Android 构建配置
├── src/
│   └── main/
│       ├── java/
│       │   └── com/
│       │       └── ggs/
│       │           └── GgsNode.java    # Java 包装类
│       └── res/
└── README.md
```

## 构建步骤

1. **安装 Android NDK**
   ```bash
   # 通过 Android Studio SDK Manager 安装 NDK
   # 或下载: https://developer.android.com/ndk/downloads
   ```

2. **配置环境变量**
   ```bash
   export ANDROID_NDK_HOME=/path/to/android-ndk
   export ANDROID_HOME=/path/to/android-sdk
   ```

3. **构建 Rust 库**
   ```bash
   # 在项目根目录
   cargo build --target aarch64-linux-android --release
   cargo build --target armv7-linux-androideabi --release
   cargo build --target i686-linux-android --release
   cargo build --target x86_64-linux-android --release
   ```

4. **集成到 Android 项目**
   - 将编译好的 `.so` 文件复制到 `android/src/main/jniLibs/`
   - 在 `build.gradle` 中配置 JNI 库路径

## 使用示例

```java
import com.ggs.GgsNode;

// 创建节点实例
GgsNode node = new GgsNode();

// 获取设备能力
String capabilities = node.getCapabilities();
System.out.println("设备能力: " + capabilities);

// 更新网络类型
node.updateNetworkType("wifi");

// 更新电池状态
node.updateBattery(0.75f, true);

// 启动训练
node.start();
```

## 注意事项

- 需要 Android API Level 21+ (Android 5.0+)
- 需要网络权限和后台运行权限
- 建议在后台服务中运行节点


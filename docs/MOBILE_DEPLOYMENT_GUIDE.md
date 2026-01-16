# Tauri 移动端产品化部署指南

## 项目移动端状态概述

当前项目已具备移动端集成基础：

### ✅ 已完成的工作

1. **FFI接口层** (`src/network/ffi.rs`)
   - 完整的C兼容接口，支持Android和iOS
   - 提供设备信息回调机制
   - 支持实时更新网络类型、电池状态等

2. **Android集成** (`android/`)
   - JNI库配置
   - Java包装类 (GgbNode.java)
   - Gradle构建配置
   - 支持armeabi-v7a, arm64-v8a, x86, x86_64

3. **iOS集成** (`ios/`)
   - Swift包装类 (GGB.swift)
   - Objective-C头文件
   - Xcode项目配置

4. **设备检测适配**
   - 移动端可以通过回调提供真实设备信息
   - 自动适应移动设备的GPU、内存、电池检测

### 📋 移动端产品化步骤

## 一、环境准备

### 1.1 安装Tauri移动端工具

```bash
# 安装Tauri CLI（如果还没安装）
cargo install tauri-cli

# 安装移动端依赖
npm install -g @tauri-apps/cli

# Android环境
cargo install cargo-ndk

# iOS环境（Mac only）
sudo xcode-select --install
```

### 1.2 配置环境变量（Android）

```bash
# Windows PowerShell
$env:ANDROID_HOME = "C:\Users\YourName\AppData\Local\Android\Sdk"
$env:ANDROID_NDK_HOME = "C:\Users\YourName\AppData\Local\Android\Sdk\ndk\25.1.8937393"
$env:PATH += ";$env:ANDROID_HOME\platform-tools"

# Linux/macOS
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/25.1.8937393
export PATH=$PATH:$ANDROID_HOME/platform-tools
```

### 1.3 安装Rust移动端工具链

```bash
# Android目标
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# iOS目标（Mac only）
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
```

## 二、配置Tauri移动端支持

### 2.1 更新Tauri配置

编辑 `src-tauri/tauri.conf.json`，添加移动端配置：

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Williw",
  "version": "0.1.1",
  "identifier": "com.williw.mobile",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "security": {
      "csp": "default-src 'self'; connect-src ipc: http://ipc.localhost"
    },
    "windows": [
      {
        "title": "Williw",
        "width": 1150,
        "height": 700
      }
    ]
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/williw.png"
    ],
    "android": {
      "minSdkVersion": 21,
      "targetSdkVersion": 34,
      "ndkVersion": "25.1.8937393"
    },
    "ios": {
      "developmentTeam": "YOUR_TEAM_ID",
      "minimumSystemVersion": "12.0"
    }
  }
}
```

### 2.2 初始化移动端项目

```bash
# 在项目根目录

# 初始化Android项目
tauri android init

# 初始化iOS项目（Mac only）
tauri ios init
```

## 三、构建移动端应用

### 3.1 Android构建步骤

```bash
# 1. 构建Rust库（所有支持的架构）
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 --release -- build --release

# 2. 将.so文件复制到Android项目
cp target/aarch64-linux-android/release/libwilliw.so android/src/main/jniLibs/arm64-v8a/
cp target/armv7-linux-androideabi/release/libwilliw.so android/src/main/jniLibs/armeabi-v7a/
cp target/x86_64-linux-android/release/libwilliw.so android/src/main/jniLibs/x86_64/
cp target/i686-linux-android/release/libwilliw.so android/src/main/jniLibs/x86/

# 3. 构建Android应用
tauri android build

# 或开发模式
tauri android dev
```

### 3.2 iOS构建步骤（Mac only）

```bash
# 1. 构建Rust库（真机和模拟器）
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release  # 模拟器

# 2. 创建XCFramework
# 使用cargo-xcode或手动创建xcframework

# 3. 构建iOS应用
tauri ios build

# 或开发模式
tauri ios dev
```

## 四、移动端UI适配

### 4.1 响应式设计

```typescript
// src/App.tsx
import { useState, useEffect } from 'react';
import { getDeviceInfo } from './services/device';

function App() {
  const [isMobile, setIsMobile] = useState(false);
  
  useEffect(() => {
    // 检测设备类型
    const checkDevice = async () => {
      const info = await getDeviceInfo();
      setIsMobile(info.device_type === 'phone' || info.device_type === 'tablet');
    };
    checkDevice();
  }, []);
  
  return (
    <div className={isMobile ? 'app-mobile' : 'app-desktop'}>
      {/* 移动端优化UI */}
    </div>
  );
}
```

### 4.2 移动端组件优化

```typescript
// src/components/TrainingSwitch.tsx
import { isTauri } from '@tauri-apps/api/core';
import { platform } from '@tauri-apps/plugin-platform';

export const TrainingSwitch = () => {
  const [isMobile, setIsMobile] = useState(false);
  
  useEffect(() => {
    const init = async () => {
      if (isTauri()) {
        const plat = await platform();
        setIsMobile(plat === 'android' || plat === 'ios');
      }
    };
    init();
  }, []);
  
  // 移动端显示简化界面
  if (isMobile) {
    return <MobileTrainingSwitch />;
  }
  
  return <DesktopTrainingSwitch />;
};
```

## 五、移动端权限配置

### 5.1 Android权限

编辑 `src-tauri/gen/android/app/src/main/AndroidManifest.xml`：

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
    <uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
    <uses-permission android:name="android.permission.BATTERY_STATS" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
    <uses-permission android:name="android.permission.REQUEST_IGNORE_BATTERY_OPTIMIZATIONS" />
    
    <!-- 如果需要后台训练 -->
    <uses-permission android:name="android.permission.WAKE_LOCK" />
</manifest>
```

### 5.2 iOS权限

编辑 `src-tauri/gen/ios/Info.plist`：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSAppTransportSecurity</key>
    <dict>
        <key>NSAllowsArbitraryLoads</key>
        <true/>
    </dict>
    <key>UIBackgroundModes</key>
    <array>
        <string>fetch</string>
        <string>processing</string>
    </array>
</dict>
</plist>
```

## 六、移动端设备检测适配

### 6.1 移动端的设备检测特点

```rust
// src/device/platform/android.rs (需要创建)
//! Android 设备检测实现

pub fn detect_gpu_apis() -> Vec<GpuComputeApi> {
    // Android 设备通常使用 Vulkan 或 OpenCL
    let mut apis = Vec::new();
    
    // 检查 OpenCL
    if check_library_exists("libOpenCL.so") {
        apis.push(GpuComputeApi::OpenCL);
    }
    
    // 检查 Vulkan
    if check_library_exists("libvulkan.so") {
        apis.push(GpuComputeApi::Vulkan);
    }
    
    apis
}

pub fn detect_network_type() -> NetworkType {
    // Android 使用 ConnectivityManager
    // 这里简化处理，实际应由 Java 层提供
    NetworkType::Unknown
}

pub fn detect_battery() -> (Option<f32>, bool) {
    // Android 电池信息通常由 Java 层提供
    (None, false)
}
```

```rust
// src/device/platform/ios.rs (需要创建)
//! iOS 设备检测实现

pub fn detect_gpu_apis() -> Vec<GpuComputeApi> {
    // iOS 使用 Metal
    vec![GpuComputeApi::Metal]
}

pub fn detect_network_type() -> NetworkType {
    // iOS 使用 NWPathMonitor
    // 这里简化处理，实际应由 Swift 层提供
    NetworkType::Unknown
}

pub fn detect_battery() -> (Option<f32>, bool) {
    // iOS 电池信息通常由 Swift 层提供
    (None, false)
}
```

### 6.2 更新平台检测模块

```rust
// src/device/platform/mod.rs

pub fn detect_gpu_apis() -> Vec<GpuComputeApi> {
    #[cfg(target_os = "android")]
    {
        android::detect_gpu_apis()
    }
    #[cfg(target_os = "ios")]
    {
        ios::detect_gpu_apis()
    }
    #[cfg(target_os = "windows")]
    {
        windows::detect_gpu_apis()
    }
    // ... 其他平台
}
```

## 七、移动端功能优化

### 7.1 电池优化

```typescript
// 检测电池状态，自动调整训练强度
import { getBatteryInfo } from '@tauri-apps/plugin-battery';

const adjustTrainingForBattery = async () => {
  const battery = await getBatteryInfo();
  
  if (battery.level < 0.2 && !battery.charging) {
    // 电量低于20%且未充电，降低训练强度
    await setTrainingIntensity('low');
  } else if (battery.level > 0.8 || battery.charging) {
    // 电量充足或正在充电，可以使用高性能模式
    await setTrainingIntensity('high');
  }
};
```

### 7.2 网络感知

```typescript
// 根据网络类型调整数据传输
import { getNetworkStatus } from '@tauri-apps/plugin-network';

const adjustForNetwork = async () => {
  const network = await getNetworkStatus();
  
  if (network.type === 'wifi') {
    // WiFi环境下可以传输更多数据
    await setSyncMode('full');
  } else if (network.type === 'cellular') {
    // 移动网络下限制数据传输
    await setSyncMode('minimal');
  }
};
```

## 八、测试与调试

### 8.1 移动端测试

```bash
# Android模拟器
tauri android dev

# iOS模拟器（Mac only）
tauri ios dev

# 真机调试（Android）
tauri android dev --target device

# 真机调试（iOS）
tauri ios dev --target device
```

### 8.2 日志查看

```bash
# Android日志
adb logcat | grep Rust

# iOS日志
# 在Xcode中查看控制台输出
```

## 九、发布与分发

### 9.1 Android发布

```bash
# 生成发布版APK
tauri android build --apk

# 生成AAB（Google Play）
tauri android build --aab

# 输出在：
# src-tauri/gen/android/app/build/outputs/apk/release/
# src-tauri/gen/android/app/build/outputs/bundle/release/
```

### 9.2 iOS发布（Mac only）

```bash
# 生成发布版
tauri ios build

# 使用Xcode打包和签名
# 打开 src-tauri/gen/ios/Williw.xcodeproj
```

### 9.3 应用商店提交

#### Android (Google Play)

1. 准备应用清单：
   - 应用图标（512x512）
   - 特色图形（1024x500）
   - 截图（至少2张）
   - 应用描述和关键词

2. 生成签名密钥：
```bash
keytool -genkey -v -keystore williw-release.keystore -alias williw -keyalg RSA -keysize 2048 -validity 10000
```

3. 在 `src-tauri/gen/android/keystore.properties` 中配置签名：
```properties
storePassword=your_store_password
keyPassword=your_key_password
keyAlias=williw
storeFile=../williw-release.keystore
```

#### iOS (App Store)

1. 准备应用资源：
   - 应用图标（多种尺寸）
   - 截图（不同设备尺寸）
   - 应用预览视频（可选）

2. 在Apple Developer Console中：
   - 创建App ID
   - 创建证书和Provisioning Profile
   - 配置App Store Connect

3. 在Xcode中配置签名

## 十、移动端最佳实践

### 10.1 性能优化

1. **内存管理**
   - 移动设备内存有限，建议限制模型大小
   - 使用 `recommended_model_dim()` 获取适合设备的模型维度
   - 定期清理缓存

2. **电量优化**
   - 监控电池状态，电量低时暂停训练
   - 未充电时降低训练频率
   - 使用批量处理减少唤醒次数

3. **网络优化**
   - 移动网络下限制数据传输量
   - WiFi环境下可以同步更多数据
   - 实现断点续传功能

### 10.2 用户体验

1. **后台运行**
   - Android: 使用Foreground Service
   - iOS: 使用Background Tasks
   - 显示通知让用户知道应用正在运行

2. **权限管理**
   - 在需要时请求权限
   - 解释为什么需要这些权限
   - 处理权限被拒绝的情况

3. **UI适配**
   - 适配不同屏幕尺寸
   - 触摸友好的界面元素
   - 简化的移动端操作流程

### 10.3 安全性

1. **数据加密**
   - 本地存储的数据加密
   - 网络传输使用TLS
   - 敏感信息使用硬件安全模块

2. **隐私保护**
   - 遵循GDPR和CCPA
   - 匿名化用户数据
   - 提供数据删除选项

## 十一、故障排除

### 常见问题

#### 问题1: Android构建失败 - NDK未找到

**解决方案**：
```bash
# 检查NDK路径
echo $ANDROID_NDK_HOME

# 在Cargo.toml中添加ndk路径
[env]
ANDROID_NDK_HOME = "/path/to/ndk"
```

#### 问题2: iOS构建失败 - 签名错误

**解决方案**：
```bash
# 检查Xcode配置
xcode-select -p

# 在Xcode中配置团队
# 1. 打开src-tauri/gen/ios/Williw.xcodeproj
# 2. 选择项目 -> Signing & Capabilities
# 3. 配置Team和Bundle Identifier
```

#### 问题3: 移动端设备检测不准确

**解决方案**：
- Android: 确保实现DeviceInfoCallback
- iOS: 确保在Swift层提供真实设备信息
- 使用回调机制而非自动检测

## 总结

本项目已经具备移动端集成的基础：

✅ **FFI接口** - 完整的C兼容接口
✅ **Android支持** - JNI集成，Gradle配置
✅ **iOS支持** - Swift/Objective-C包装
✅ **设备检测适配** - 移动端回调机制

**下一步**:
1. 配置Tauri移动端环境
2. 优化移动端UI
3. 配置应用权限
4. 实现电池和网络感知功能
5. 测试和发布

按照本指南操作，可以将项目成功转化为移动端产品应用！

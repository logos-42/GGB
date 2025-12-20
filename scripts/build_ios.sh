#!/bin/bash
# iOS 构建脚本
# 构建 Rust 库并打包为 XCFramework

set -e

# 配置
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IOS_DIR="$PROJECT_ROOT/ios"
TARGET_DIR="$PROJECT_ROOT/target"
XCFRAMEWORK_DIR="$IOS_DIR/GGB.xcframework"

# iOS 目标架构
IOS_TARGETS=("aarch64-apple-ios" "x86_64-apple-ios")
IOS_SIMULATOR_TARGETS=("aarch64-apple-ios-sim" "x86_64-apple-ios-sim")

# 检查 Rust iOS 工具链
echo "检查 Rust iOS 工具链..."
for target in "${IOS_TARGETS[@]}" "${IOS_SIMULATOR_TARGETS[@]}"; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo "安装 $target 工具链..."
        rustup target add "$target"
    fi
done

# 清理旧的构建
echo "清理旧的构建..."
rm -rf "$XCFRAMEWORK_DIR"
rm -rf "$IOS_DIR/build"

# 构建 Rust 库
echo "构建 Rust 库..."
cd "$PROJECT_ROOT"

# 构建真机库
for target in "${IOS_TARGETS[@]}"; do
    echo "构建 $target..."
    cargo build --target "$target" --release --features ffi
done

# 构建模拟器库
for target in "${IOS_SIMULATOR_TARGETS[@]}"; do
    echo "构建 $target..."
    cargo build --target "$target" --release --features ffi
done

# 创建 XCFramework
echo "创建 XCFramework..."

# 创建临时目录
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# 为每个架构创建 framework 结构
FRAMEWORKS=()

# 真机 frameworks
for target in "${IOS_TARGETS[@]}"; do
    ARCH_NAME=$(echo "$target" | sed 's/-apple-ios//')
    FRAMEWORK_NAME="GGB_${ARCH_NAME}.framework"
    FRAMEWORK_PATH="$TEMP_DIR/$FRAMEWORK_NAME"
    
    mkdir -p "$FRAMEWORK_PATH/Headers"
    mkdir -p "$FRAMEWORK_PATH/Modules"
    
    # 复制库文件
    if [ -f "$TARGET_DIR/$target/release/libggb.a" ]; then
        cp "$TARGET_DIR/$target/release/libggb.a" "$FRAMEWORK_PATH/GGB"
    elif [ -f "$TARGET_DIR/$target/release/libGGB.a" ]; then
        cp "$TARGET_DIR/$target/release/libGGB.a" "$FRAMEWORK_PATH/GGB"
    fi
    
    # 复制头文件
    cp "$IOS_DIR/GGB.h" "$FRAMEWORK_PATH/Headers/"
    
    # 创建 modulemap
    cat > "$FRAMEWORK_PATH/Modules/module.modulemap" <<EOF
framework module GGB {
    header "GGB.h"
    export *
}
EOF
    
    # 创建 Info.plist
    cat > "$FRAMEWORK_PATH/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleIdentifier</key>
    <string>com.ggb</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>GGB</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>MinimumOSVersion</key>
    <string>12.0</string>
</dict>
</plist>
EOF
    
    FRAMEWORKS+=("-framework" "$FRAMEWORK_PATH")
done

# 使用 xcodebuild 创建 XCFramework
xcodebuild -create-xcframework \
    "${FRAMEWORKS[@]}" \
    -output "$XCFRAMEWORK_DIR"

echo "iOS XCFramework 构建完成！"
echo "Framework 位置: $XCFRAMEWORK_DIR"


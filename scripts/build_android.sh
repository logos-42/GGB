#!/bin/bash
# Android 构建脚本
# 构建 Rust 库并集成到 Android 项目

set -e

# 配置
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ANDROID_DIR="$PROJECT_ROOT/android"
TARGET_DIR="$PROJECT_ROOT/target"
ANDROID_ABIS=("aarch64-linux-android" "armv7-linux-androideabi" "i686-linux-android" "x86_64-linux-android")
ANDROID_ABI_NAMES=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")

# 检查 Rust 工具链
echo "检查 Rust Android 工具链..."
for target in "${ANDROID_ABIS[@]}"; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo "安装 $target 工具链..."
        rustup target add "$target"
    fi
done

# 构建 Rust 库
echo "构建 Rust 库..."
cd "$PROJECT_ROOT"

for i in "${!ANDROID_ABIS[@]}"; do
    target="${ANDROID_ABIS[$i]}"
    abi_name="${ANDROID_ABI_NAMES[$i]}"
    
    echo "构建 $target..."
    cargo build --target "$target" --release --features ffi
    
    # 复制库文件到 Android 项目
    mkdir -p "$ANDROID_DIR/src/main/jniLibs/$abi_name"
    
    # 查找生成的库文件
    if [ -f "$TARGET_DIR/$target/release/libggb.so" ]; then
        cp "$TARGET_DIR/$target/release/libggb.so" "$ANDROID_DIR/src/main/jniLibs/$abi_name/"
        echo "已复制 libggb.so 到 $abi_name"
    elif [ -f "$TARGET_DIR/$target/release/libGGB.so" ]; then
        cp "$TARGET_DIR/$target/release/libGGB.so" "$ANDROID_DIR/src/main/jniLibs/$abi_name/libggb.so"
        echo "已复制 libGGB.so 到 $abi_name (重命名为 libggb.so)"
    else
        echo "警告: 未找到 $target 的库文件"
    fi
done

echo "Android 构建完成！"
echo "库文件位置: $ANDROID_DIR/src/main/jniLibs/"


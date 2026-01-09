#!/bin/bash

echo "🔧 开始构建 WASM..."

# 切换到 wasm 目录
cd "$(dirname "$0")/../wasm"

# 清理旧的构建产物
if [ -d pkg ]; then
    echo "🧹 清理旧的构建产物..."
    rm -rf pkg
fi

# 创建 pkg 目录
mkdir -p pkg

# 构建 WASM
echo "🚀 使用 wasm-pack 构建..."
wasm-pack build --target web --out-dir pkg

if [ $? -ne 0 ]; then
    echo "❌ WASM 构建失败"
    exit 1
fi

echo "✅ WASM 构建完成！"

# 列出构建产物
echo ""
echo "📦 构建产物:"
ls -lh pkg/

#!/bin/bash
# WASM构建脚本

set -e

echo "🔧 构建WASM目标..."

# 安装wasm-bindgen-cli如果不存在
if ! command -v wasm-bindgen &> /dev/null; then
    echo "📦 安装wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# 创建输出目录
mkdir -p wasm/pkg

# 构建WASM目标
echo "🚀 构建WASM..."
cargo build --target wasm32-unknown-unknown --release --features wasm

# 生成绑定
echo "🔗 生成WASM绑定..."
wasm-bindgen \
    --target web \
    --out-dir wasm/pkg \
    --out-name ggb_wasm \
    target/wasm32-unknown-unknown/release/ggb.wasm

echo "✅ WASM构建完成！"
echo "📁 输出目录: wasm/pkg/"
echo "📄 主要文件:"
echo "   - ggb_wasm.js"
echo "   - ggb_wasm_bg.wasm"
echo "   - ggb_wasm.d.ts"

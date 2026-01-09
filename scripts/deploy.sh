#!/bin/bash

echo "========================================"
echo "  Williw Workers 部署脚本"
echo "========================================"
echo ""

# 切换到脚本所在目录
cd "$(dirname "$0")/.."

# 检查 wrangler
echo "[1/4] 检查 Cloudflare Workers CLI..."
if ! command -v wrangler &> /dev/null; then
    echo "❌ wrangler 未安装"
    echo "请运行: npm install -g wrangler"
    exit 1
fi
echo "✅ wrangler 已安装"
echo ""

# 检查 wasm-pack
echo "[2/4] 检查 wasm-pack..."
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 安装 wasm-pack..."
    cargo install wasm-pack
fi
echo "✅ wasm-pack 已安装"
echo ""

# 构建 WASM
echo "[3/4] 构建 WASM 模块..."
bash scripts/build_wasm.sh
if [ $? -ne 0 ]; then
    echo "❌ WASM 构建失败"
    exit 1
fi
echo ""

# 部署到 Cloudflare Workers
echo "[4/4] 部署到 Cloudflare Workers..."
echo "🚀 开始部署..."
cd workers-config
wrangler deploy
if [ $? -ne 0 ]; then
    echo "❌ 部署失败"
    cd ..
    exit 1
fi
cd ..
echo ""

echo "========================================"
echo "  ✅ 部署完成！"
echo "========================================"
echo ""
echo "📊 部署信息:"
echo "  - Worker 名称: williw"
echo "  - 账户: yuanjieliu65@gmail.com"
echo ""
echo "🌐 测试端点:"
echo "  - 健康检查: https://williw.workers.dev/health"
echo "  - API: https://williw.workers.dev/api/*"
echo ""

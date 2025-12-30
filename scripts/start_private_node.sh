#!/bin/bash
# 启动隐私保护节点脚本
# 使用方法: ./scripts/start_private_node.sh [配置文件]

set -e

CONFIG_FILE="${1:-config/security.toml}"

echo "=== 启动 GGB 隐私保护节点 ==="
echo "配置文件: $CONFIG_FILE"

# 检查配置文件是否存在
if [ ! -f "$CONFIG_FILE" ]; then
    echo "错误: 配置文件 $CONFIG_FILE 不存在"
    echo ""
    echo "请创建配置文件或使用示例配置:"
    echo "  cp config/privacy_example.toml config/security.toml"
    echo "  # 编辑 config/security.toml 配置中继节点"
    exit 1
fi

# 检查中继节点配置
if grep -q "relay_nodes = \[\]" "$CONFIG_FILE"; then
    echo "警告: 配置文件中未设置中继节点"
    echo "隐私保护需要至少一个可用的中继节点"
    echo "请编辑 $CONFIG_FILE 添加中继节点地址"
    read -p "是否继续? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 设置环境变量
export RUST_LOG="info,ggb=debug"
export GGB_PRIVACY_MODE="enabled"

echo ""
echo "启动参数:"
echo "  - 配置文件: $CONFIG_FILE"
echo "  - 日志级别: $RUST_LOG"
echo "  - 隐私模式: $GGB_PRIVACY_MODE"
echo ""

# 运行节点
echo "启动节点..."
cargo run --release -- \
    --config "$CONFIG_FILE" \
    --log-level info

echo ""
echo "节点已停止"
echo "=== 完成 ==="

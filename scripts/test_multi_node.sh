#!/bin/bash
# GGS 多节点测试脚本 (Linux/Mac)
# 使用方法: ./scripts/test_multi_node.sh --nodes 3 --duration 300

set -e

# 默认参数
NODES=3
DURATION=300
MODEL_DIM=256
OUTPUT_DIR="test_output"

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--nodes)
            NODES="$2"
            shift 2
            ;;
        -d|--duration)
            DURATION="$2"
            shift 2
            ;;
        -m|--model-dim)
            MODEL_DIM="$2"
            shift 2
            ;;
        -o|--output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "未知参数: $1"
            exit 1
            ;;
    esac
done

echo "=== GGS 多节点协同训练测试 ==="
echo "节点数量: $NODES"
echo "训练时长: $DURATION 秒"
echo "模型维度: $MODEL_DIM"
echo "输出目录: $OUTPUT_DIR"
echo ""

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 启动多个节点
PIDS=()
for ((i=0; i<NODES; i++)); do
    NODE_ID=$i
    STATS_FILE="$OUTPUT_DIR/node_${NODE_ID}_stats.json"
    LOG_FILE="$OUTPUT_DIR/node_${NODE_ID}.log"
    ERROR_LOG="$OUTPUT_DIR/node_${NODE_ID}_error.log"
    
    # 设置设备类型
    case $((i % 3)) in
        0) DEVICE_TYPE="low" ;;
        1) DEVICE_TYPE="mid" ;;
        *) DEVICE_TYPE="high" ;;
    esac
    
    echo "启动节点 $NODE_ID (设备类型: $DEVICE_TYPE)..."
    
    # 启动节点进程（后台运行）
    GGS_DEVICE_TYPE=$DEVICE_TYPE RUST_LOG=info \
        cargo run --release -- \
        --node-id $NODE_ID \
        --stats-output "$STATS_FILE" \
        > "$LOG_FILE" 2> "$ERROR_LOG" &
    
    PIDS+=($!)
    
    # 错开启动时间
    sleep 0.5
done

echo ""
echo "所有节点已启动，开始训练..."
echo "等待 $DURATION 秒后自动停止..."
echo ""

# 等待指定时间
sleep $DURATION

echo ""
echo "训练时间到，正在停止所有节点..."

# 停止所有进程
for pid in "${PIDS[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
    fi
done

# 等待所有进程结束
for pid in "${PIDS[@]}"; do
    wait "$pid" 2>/dev/null || true
done

echo ""
echo "=== 测试完成 ==="
echo "统计数据已保存到: $OUTPUT_DIR"
echo ""
echo "可以使用以下命令分析结果："
echo "  cargo run --bin analyze_training -- --input $OUTPUT_DIR"


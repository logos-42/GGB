#!/usr/bin/env python3
"""
创建示例模型文件
用于测试和演示
"""

import numpy as np
import sys
import os

def create_sample_model(dim=256, output_path="examples/sample_model.npy"):
    """创建示例模型文件"""
    # 创建随机初始化的模型参数
    # 使用正态分布，均值 0，标准差 0.1
    arr = np.random.randn(dim).astype(np.float32) * 0.1
    
    # 确保输出目录存在
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    # 保存为 .npy 文件
    np.save(output_path, arr)
    
    print(f"已创建示例模型文件: {output_path}")
    print(f"模型维度: {dim}")
    print(f"参数范围: [{arr.min():.6f}, {arr.max():.6f}]")
    print(f"参数均值: {arr.mean():.6f}")
    print(f"参数标准差: {arr.std():.6f}")

if __name__ == "__main__":
    dim = 256
    if len(sys.argv) > 1:
        dim = int(sys.argv[1])
    
    output_path = "examples/sample_model.npy"
    if len(sys.argv) > 2:
        output_path = sys.argv[2]
    
    create_sample_model(dim, output_path)


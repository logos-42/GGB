# GGS 去中心化训练节点

面向 Geo-Similarity-Weighted Gossip (GGS) 的 Rust 节点实现，集成真实推理张量、QUIC Gossip 通道、地理 + 嵌入双指标拓扑、以及 Web3 签名 / 质押 / 信誉系统，可直接部署到 Base 网络环境。

## 功能概览

- **推理引擎 (`src/inference.rs`)**
  - 从 `.npy` 模型参数加载，维护 TensorSnapshot、SparseUpdate，并带 residual 误差反馈。
  - 支持 Top-K 稀疏更新、密集快照、local training tick；可输出模型 hash & 维度。
  - **新增**：内存压力检测，自动调整 Top-K 值以降低内存使用。

- **通信层 (`src/comms.rs`)**
  - 基于 `libp2p gossipsub + mDNS` 的控制平面。
  - QUIC (`quinn`) 数据平面，带带宽预算（稀疏次数 / 密集字节）和 failover 回落。
  - **新增**：网络类型检测（WiFi/4G/5G），根据网络类型动态调整带宽和传输策略。
  - **新增**：QUIC 连接健康检查和自动重连机制。

- **拓扑模块 (`src/topology.rs`)**
  - Geo + embedding 双指标评分，维护主邻居 + 备份池，支持 failover / mark unreachable。
  - 为日志提供 `PeerSnapshot`（相似度、地理亲和、嵌入维度、位置）。
  - **新增**：根据设备能力自动调整邻居数量。

- **共识与 Web3 (`src/consensus.rs`, `src/crypto.rs`)**
  - 以太坊 (k256) + Solana (ed25519) 双签名；stake/reputation 计分。
  - 心跳 / 稀疏 / 密集消息统一签名与验证，并按活动自动调整信誉。

- **设备适配模块 (`src/device.rs`)** 🆕
  - 设备能力检测：内存、CPU、网络类型、电池状态。
  - 自适应配置：根据设备能力自动调整模型维度、带宽预算、邻居数量。
  - 电池感知调度：根据电量自动调整训练频率。
  - 网络自适应：WiFi 允许密集快照，移动网络仅稀疏更新。

- **FFI 接口 (`src/ffi.rs`)** 🆕
  - C 兼容的 FFI 接口，供 Android/iOS 移动端调用。
  - 支持设备能力查询、网络状态更新、电池状态更新等功能。

## 快速开始

```bash
cargo check          # 仅编译检查
cargo run            # 运行节点，默认随机Geo位置 & 128维模型
```

启动日志中将输出本地 peer id、ETH/SOL 地址、模型维度、设备能力信息，以及拓扑评分详情。默认 Gossip 主题为 `ggs-training`，可在 `CommsConfig` 自定义监听地址 / QUIC 端口 / 带宽预算。

### 移动设备适配

系统现在支持移动设备自适应配置：

- **环境变量配置**（用于测试）：
  ```bash
  # 设置设备类型
  export GGS_DEVICE_TYPE=low    # low/mid/high
  export GGS_NETWORK_TYPE=wifi   # wifi/4g/5g
  export GGS_BATTERY_LEVEL=0.75  # 0.0-1.0
  export GGS_BATTERY_CHARGING=true
  
  cargo run
  ```

- **自动适配功能**：
  - 根据设备内存自动调整模型维度
  - 根据网络类型（WiFi/4G/5G）调整带宽预算
  - 根据电池电量自动调整训练频率
  - 移动网络下自动禁用密集快照传输

## 测试与验证

### 多节点测试

快速启动多节点测试：

**Windows:**
```powershell
.\scripts\test_multi_node.ps1 -Nodes 3 -Duration 300
```

**Linux/Mac:**
```bash
bash scripts/test_multi_node.sh --nodes 3 --duration 300
```

### 分析训练结果

```bash
cargo run --bin analyze_training -- --input test_output
```

### 训练监控

系统会自动输出训练统计信息：
- 每 10 个 tick 输出统计摘要
- 显示收敛度指标、参数变化、标准差
- 支持导出 JSON 格式统计数据

详细测试指南请参考 [docs/TESTING.md](docs/TESTING.md)

## 移动端集成

### Android

Android 集成代码位于 `android/` 目录，包含：
- Java 包装类 (`GgsNode.java`)
- 设备能力检测（网络、电池）
- JNI 绑定配置

详细说明请参考 [android/README.md](android/README.md)

### iOS

iOS 集成代码位于 `ios/` 目录，包含：
- Swift 包装类 (`GGS.swift`)
- 设备能力检测（网络、电池）
- XCFramework 构建配置

详细说明请参考 [ios/README.md](ios/README.md)

## 自定义与扩展

- 在 `Cargo.toml` 中加入所需的推理库（如 Candle、ONNX Runtime）后，扩展 `InferenceEngine` 的加载逻辑即可。
- 若要使用实际的链上 RPC，可在 `ConsensusEngine` 中替换当前内存 staking 账本。
- 通过 `TopologyConfig` 的参数可调邻居数量、备份池大小、地理缩放等策略。

## 目录结构

```
├── src/
│   ├── main.rs          # Node 入口，驱动所有模块
│   ├── comms.rs         # Gossip + QUIC 通信层（含网络适配）
│   ├── inference.rs     # 推理张量与更新逻辑（含资源监控和收敛度）
│   ├── topology.rs      # 拓扑评分与 failover
│   ├── consensus.rs     # 签名、质押、信誉
│   ├── crypto.rs        # ETH/SOL 密钥管理
│   ├── device.rs        # 设备能力检测与自适应配置 🆕
│   ├── stats.rs         # 训练统计与监控 🆕
│   └── ffi.rs           # FFI 接口（移动端集成）🆕
├── examples/
│   └── multi_node_test.rs  # 多节点测试工具 🆕
├── tools/
│   └── analyze_training.rs  # 训练结果分析工具 🆕
├── scripts/
│   └── test_multi_node.ps1  # Windows 测试脚本 🆕
├── android/              # Android 集成代码 🆕
├── ios/                  # iOS 集成代码 🆕
├── tests/                # 集成测试 🆕
├── docs/                 # 文档 🆕
├── Cargo.toml
└── README.md
```

## 贡献与远程

仓库：`git@github.com:logos-42/GGS.git`  
当前默认分支为 `master`，欢迎在此基础上提交 PR 或扩展模块（例如 WebRTC、治理合约集成等）。



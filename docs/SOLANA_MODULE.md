# Solana 区块链集成模块

## 概述

本模块提供了与 Solana 区块链交互的完整功能，包括：
- 节点算力贡献记录
- 收益分配管理
- 智能合约交互
- 交易签名和广播

## 文件结构

```
src/solana/
├── mod.rs        # 模块入口，定义配置和基本类型
├── types.rs      # 数据类型定义（节点信息、贡献记录、收益等）
├── compute.rs    # 算力贡献跟踪和计算
├── rewards.rs    # 收益分配管理
└── client.rs     # Solana 区块链客户端
```

## 主要功能

### 1. 节点管理
- 注册节点到区块链
- 更新节点状态
- 查询节点信息

### 2. 算力贡献
- 跟踪节点计算任务
- 记录资源使用情况（GPU、CPU、内存、网络）
- 计算算力评分
- 查询贡献统计

### 3. 收益管理
- 计算节点收益
- 生成结算计划
- 执行收益分配
- 查询钱包余额

## 使用示例

### 初始化客户端

```rust
use williw::solana::{SolanaClient, SolanaConfig};

let config = SolanaConfig::devnet("your_program_id");
let client = SolanaClient::new(config, "node_id_123".to_string());
```

### 跟踪算力贡献

```rust
// 开始任务
let tracker = client.get_compute_tracker();
{
    let mut tracker = tracker.write();
    tracker.start_task("task_123".to_string())?;
}

// 完成任务
{
    let mut tracker = tracker.write();
    let contribution = tracker.complete_task(
        10000,     // samples_processed
        100,        // batches_processed
        500,        // network_upload_mb
        200,        // network_download_mb
    )?;
}

// 上报贡献到区块链
client.report_compute_contribution(contribution).await?;
```

### 收益分配

```rust
// 计算待结算收益
let manager = client.get_reward_manager();
{
    let manager = manager.read();
    let pending = manager.calculate_pending_rewards(&contributions);
}

// 生成结算计划
let balances = client.get_wallet_balance(&address).await?;
let plan = client.generate_settlement_plan(vec![balances]).await?;

// 执行结算
let results = client.execute_settlement(&plan).await?;
```

## 算力评分计算

算力评分基于以下指标：

- 计算时长（30% 权重）
- 处理样本数（25% 权重）
- GPU 使用率（25% 权重）
- CPU 使用率（10% 权重）
- 网络流量（10% 权重）

## 贡献等级

- **Beginner**（初级）：平均评分 < 0.5 或贡献数 < 10
- **Regular**（常规）：平均评分 >= 0.5 且贡献数 >= 10，奖励倍率 1.1x
- **Medium**（中级）：平均评分 >= 1.5 且贡献数 >= 20，奖励倍率 1.25x
- **High**（高级）：平均评分 >= 3.0 且贡献数 >= 50，奖励倍率 1.5x
- **Elite**（精英）：平均评分 >= 5.0 且贡献数 >= 100，奖励倍率 2.0x

## 待完成功能

以下功能标记为 TODO，需要实现实际的 Solana 区块链交互：

1. **节点注册**（`register_node`）
   - 实际调用 Solana 智能合约的注册函数
   - 需要编写智能合约程序

2. **状态更新**（`update_node_status`）
   - 实际更新链上节点状态

3. **贡献上报**（`report_compute_contribution`）
   - 实际调用智能合约记录贡献

4. **收益分配**（`distribute_rewards`）
   - 实际执行 SOL 转账交易

5. **状态查询**（`get_contract_state`、`get_node_info`）
   - 从区块链读取账户数据

## 智能合约接口

需要创建对应的 Solana 程序（智能合约），提供以下指令：

```rust
// 账户结构
pub struct NodeAccount {
    pub owner: Pubkey,
    pub status: u8,
    pub total_compute_score: u64,
    pub total_rewards: u64,
}

pub struct ContributionAccount {
    pub node: Pubkey,
    pub task_id: String,
    pub compute_score: u64,
    pub timestamp: i64,
}

pub struct TreasuryAccount {
    pub admin: Pubkey,
    pub total_distributed: u64,
    pub balance: u64,
}

// 指令
pub enum ProgramInstruction {
    RegisterNode,
    UpdateNodeStatus { status: u8 },
    RecordContribution { score: u64, task_id: String },
    DistributeRewards { amount: u64 },
    WithdrawRewards,
}
```

## 注意事项

1. **测试网络**：当前默认使用 devnet，生产环境需切换到 mainnet
2. **私钥安全**：生产环境中私钥应安全存储，避免硬编码
3. **Gas 费用**：每次交易都需要支付 SOL 作为 gas 费
4. **并发控制**：算力跟踪器使用了 `RwLock`，注意线程安全

## 依赖项

已添加到 `Cargo.toml`：

```toml
solana-sdk = "1.18"
solana-client = "1.18"
solana-account-decoder = "1.18"
chrono = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }
log = "0.4"
```

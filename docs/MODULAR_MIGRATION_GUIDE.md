# 智能合约拆分迁移指南

## 📋 概述

本文档描述了如何从单一合约迁移到拆分后的模块化合约架构。

## 🏗️ 新架构概览

### 拆分后的合约模块

1. **共享类型库** (`shared-types`)
   - 包含所有合约共享的数据类型和工具函数
   - 不需要单独部署，作为依赖库使用

2. **节点管理合约** (`node-management`)
   - 节点注册、状态更新、验证、罚没
   - 程序ID: `NODE_MANAGEMENT_PROGRAM_ID`

3. **贡献跟踪合约** (`contribution-tracking`)
   - 算力贡献记录、验证、奖励计算
   - 程序ID: `CONTRIBUTION_TRACKING_PROGRAM_ID`

4. **收益管理合约** (`reward-management`)
   - 收益分配、质押、资金池管理
   - 程序ID: `REWARD_MANAGEMENT_PROGRAM_ID`

5. **治理合约** (`governance`)
   - 多签管理、提案投票、参数更新
   - 程序ID: `GOVERNANCE_PROGRAM_ID`

## 🔄 迁移步骤

### 1. 环境准备

```bash
# 更新 Anchor CLI
npm install -g @coral-xyz/anchor@latest

# 检查 Solana CLI
solana --version
```

### 2. 部署新合约

#### 使用脚本部署（推荐）

**Linux/Mac:**
```bash
cd decentralized-training-contract
chmod +x scripts/deploy-modular.sh
./scripts/deploy-modular.sh
```

**Windows:**
```powershell
cd decentralized-training-contract
.\scripts\deploy-modular.ps1
```

#### 手动部署

```bash
# 构建所有合约
anchor build --config Anchor-modular.toml

# 按顺序部署
anchor deploy node-management --config Anchor-modular.toml
anchor deploy contribution-tracking --config Anchor-modular.toml
anchor deploy reward-management --config Anchor-modular.toml
anchor deploy governance --config Anchor-modular.toml
```

### 3. 更新程序ID

部署后，需要更新以下文件中的程序ID：

1. **客户端配置** (`src/solana/mod.rs`)
2. **Anchor 配置文件** (`Anchor-modular.toml`)
3. **环境变量**

### 4. 数据迁移

从旧合约迁移数据到新合约：

```bash
# 使用数据迁移工具
anchor run migrate-data -- --old-program-id OLD_ID --new-program-ids "NODE_MANAGEMENT_ID,CONTRIBUTION_TRACKING_ID,REWARD_MANAGEMENT_ID,GOVERNANCE_ID"
```

## 🔧 客户端 SDK 更新

### 新的客户端结构

```rust
// 拆分后的客户端
pub struct ModularSolanaClient {
    pub node_management: NodeManagementClient,
    pub contribution_tracking: ContributionTrackingClient,
    pub reward_management: RewardManagementClient,
    pub governance: GovernanceClient,
}
```

### 使用示例

```rust
// 创建模块化客户端
let client = ModularSolanaClient::new(config)?;

// 注册节点
let node_result = client.node_management.register_node(node_info).await?;

// 记录贡献
let contribution_result = client.contribution_tracking.record_contribution(contribution).await?;

// 分配收益
let reward_result = client.reward_management.distribute_rewards(node_id, amount).await?;
```

## 📊 合约间通信

### CPI (跨程序调用)

新合约通过 CPI 进行通信：

```rust
// 在收益管理合约中调用节点管理合约
let cpi_context = CpiContext::new(
    node_management_program.to_account_info(),
    UpdateNodeStatus {
        node_account: node_account.to_account_info(),
        state: node_management_state.to_account_info(),
        authority: authority.to_account_info(),
    },
);

node_management::cpi::update_node_status(cpi_context, node_id, new_status)?;
```

### 共享状态

通过 PDA (程序派生地址) 共享状态：

```rust
// 查找节点账户 PDA
let (node_account_pda, _) = Pubkey::find_program_address(
    &[b"node", node_id.as_ref()],
    &node_management_program_id
);
```

## 🧪 测试

### 运行测试

```bash
# 测试所有模块
anchor test --config Anchor-modular.toml

# 测试特定模块
anchor test node-management --config Anchor-modular.toml
anchor test contribution-tracking --config Anchor-modular.toml
anchor test reward-management --config Anchor-modular.toml
anchor test governance --config Anchor-modular.toml
```

### 集成测试

```bash
# 运行完整集成测试
anchor test integration --config Anchor-modular.toml
```

## 📈 性能优化

### Gas 费用优化

1. **批量操作**: 使用批量函数减少交易数量
2. **账户复用**: 复用现有账户减少创建成本
3. **指令合并**: 合并相关指令到单个交易

### 存储优化

1. **数据压缩**: 使用更紧凑的数据结构
2. **延迟删除**: 标记删除而非立即删除
3. **分片存储**: 大数据集分片存储

## 🔒 安全考虑

### 权限管理

1. **多签验证**: 重要操作需要多签确认
2. **时间锁**: 关键操作添加时间延迟
3. **角色分离**: 不同合约使用不同的管理员

### 升级策略

1. **渐进升级**: 逐个模块升级
2. **回滚机制**: 支持快速回滚
3. **兼容性测试**: 升级前充分测试

## 🚨 故障排除

### 常见问题

1. **程序ID不匹配**
   ```
   Error: Program ID mismatch
   ```
   解决方案：检查配置文件中的程序ID

2. **CPI 调用失败**
   ```
   Error: Cross-program invocation failed
   ```
   解决方案：验证目标程序是否正确部署

3. **账户权限错误**
   ```
   Error: An account required by the instruction is missing
   ```
   解决方案：检查账户列表和权限设置

### 调试工具

```bash
# 查看程序日志
solana logs PROGRAM_ID

# 检查账户状态
solana account ACCOUNT_ID

# 模拟交易
solana confirm --simulate TRANSACTION_SIGNATURE
```

## 📚 参考资料

- [Anchor 框架文档](https://anchor-lang.com/)
- [Solana 开发者文档](https://docs.solana.com/)
- [跨程序调用指南](https://docs.solana.com/developing/programming-model/calling-between-programs)

## 🤝 支持

如果在迁移过程中遇到问题，请：

1. 查看本文档的故障排除部分
2. 检查 GitHub Issues
3. 联系开发团队

---

*本指南将随着合约拆分工作的进展持续更新。*

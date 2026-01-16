# 智能合约重构指南

## 📋 概述

本指南说明如何将原有的单一智能合约重构为模块化架构。

## 🔄 重构策略

### 原合约处理

由于合约尚未部署，我们采用**直接重构**策略：

1. **保留原合约**作为参考 (`old-contract/`)
2. **创建新的模块化合约**
3. **完全替换**原有架构
4. **更新客户端**以使用新接口

### 📁 目录结构变化

**重构前:**
```
programs/
└── decentralized-training-contract/
    ├── src/lib.rs (单一合约)
    └── Cargo.toml
```

**重构后:**
```
programs/
├── shared/
│   └── types/ (共享类型库)
├── node-management/ (节点管理)
├── contribution-tracking/ (贡献跟踪)
├── reward-management/ (收益管理)
├── governance/ (治理)
└── old-contract/ (原合约备份)
```

## 🚀 部署步骤

### 1. 构建新合约

```bash
# 构建所有模块化合约
anchor build --config Anchor-modular.toml
```

### 2. 部署新合约

```bash
# 使用自动化脚本部署
.\scripts\deploy-modular.ps1

# 或手动部署
anchor deploy node-management --config Anchor-modular.toml
anchor deploy contribution-tracking --config Anchor-modular.toml
anchor deploy reward-management --config Anchor-modular.toml
anchor deploy governance --config Anchor-modular.toml
```

### 3. 更新程序ID

部署后，需要更新以下文件中的程序ID：

1. **客户端配置** (`src/solana/modular_client.rs`)
2. **环境变量**
3. **配置文件**

## 🔄 客户端迁移

### 原客户端代码

```rust
// 原来的单一客户端
let client = SolanaClient::new(config, node_id)?;
let result = client.register_node(node_info).await?;
```

### 新客户端代码

```rust
// 新的模块化客户端
let program_ids = ProgramIds {
    node_management: "NODE_MANAGEMENT_PUBKEY".parse()?,
    contribution_tracking: "CONTRIBUTION_TRACKING_PUBKEY".parse()?,
    reward_management: "REWARD_MANAGEMENT_PUBKEY".parse()?,
    governance: "GOVERNANCE_PUBKEY".parse()?,
};

let client = ModularSolanaClient::new(
    rpc_url,
    program_ids,
    node_id,
    payer_keypair_base58,
)?;

let result = client.register_node(node_info).await?;
```

## 📊 功能映射

| 原合约函数 | 新合约位置 | 新函数名 |
|-------------|-------------|----------|
| `register_node` | `node-management` | `register_node` |
| `record_contribution` | `contribution-tracking` | `record_contribution` |
| `distribute_rewards` | `reward-management` | `distribute_rewards` |
| `create_multisig` | `governance` | `create_multisig` |
| `stake_tokens` | `reward-management` | `stake_tokens` |
| `verify_contribution` | `contribution-tracking` | `verify_contribution` |

## 🔧 配置更新

### Anchor 配置

**原配置** (`Anchor.toml`):
```toml
[programs.devnet]
decentralized_training_contract = "4SLjWwRYgRRdr4i5pgfjcbZEswXZRDcZ31BT1gipYdPq"
```

**新配置** (`Anchor-modular.toml`):
```toml
[programs.devnet]
node_management = "NODE_MANAGEMENT_PROGRAM_ID"
contribution_tracking = "CONTRIBUTION_TRACKING_PROGRAM_ID"
reward_management = "REWARD_MANAGEMENT_PROGRAM_ID"
governance = "GOVERNANCE_PROGRAM_ID"
```

### 客户端配置

**原配置**:
```rust
pub struct SolanaConfig {
    pub rpc_url: String,
    pub program_id: String, // 单一程序ID
    // ...
}
```

**新配置**:
```rust
pub struct ProgramIds {
    pub node_management: Pubkey,
    pub contribution_tracking: Pubkey,
    pub reward_management: Pubkey,
    pub governance: Pubkey,
}
```

## 🧪 测试策略

### 1. 单元测试

```bash
# 测试各个模块
anchor test node-management --config Anchor-modular.toml
anchor test contribution-tracking --config Anchor-modular.toml
anchor test reward-management --config Anchor-modular.toml
anchor test governance --config Anchor-modular.toml
```

### 2. 集成测试

```bash
# 测试模块间交互
anchor test integration --config Anchor-modular.toml
```

### 3. 性能测试

```bash
# 对比新旧架构性能
anchor test --benchmark --config Anchor-modular.toml
```

## 📈 优势对比

### 重构前 (单一合约)

**优点:**
- 部署简单
- 调用方便
- 状态共享容易

**缺点:**
- 代码复杂度高
- 升级困难
- Gas 费用高
- 安全风险集中

### 重构后 (模块化)

**优点:**
- 代码模块化
- 易于维护
- 独立升级
- Gas 优化
- 安全隔离

**缺点:**
- 部署复杂
- 跨合约调用开销
- 状态同步复杂

## 🚨 注意事项

### 1. 程序ID管理

- 记录所有新合约的程序ID
- 更新所有配置文件
- 备份原程序ID

### 2. 状态迁移

由于是新部署，无需状态迁移。但如果将来需要迁移：

```bash
# 使用数据迁移工具
anchor run migrate-data -- \
  --from-program OLD_PROGRAM_ID \
  --to-programs "NODE_MANAGEMENT_ID,CONTRIBUTION_TRACKING_ID,REWARD_MANAGEMENT_ID,GOVERNANCE_ID"
```

### 3. 向后兼容

- 保持API接口兼容性
- 提供迁移文档
- 支持渐进式迁移

## 🔄 回滚计划

如果重构后出现问题：

1. **暂停新合约**使用
2. **重新部署原合约**
3. **回滚客户端**代码
4. **分析问题**并修复

## 📚 参考资料

- [模块化架构设计](./MODULAR_MIGRATION_GUIDE.md)
- [Anchor 框架文档](https://anchor-lang.com/)
- [Solana 开发指南](https://docs.solana.com/)

---

*重构完成后，请删除本指南和 `old-contract/` 目录。*

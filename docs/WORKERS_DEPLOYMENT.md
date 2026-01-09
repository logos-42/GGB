# Workers 部署指南

## 概述

本指南说明如何部署去中心化训练系统到 Cloudflare Workers，包括：
- 数据库支持（D1, KV, Durable Objects）
- 时间适配（Workers 时间钟）
- 长期进程支持

## 数据库架构

### 1. D1 数据库（SQLite 兼容）

D1 用于结构化数据存储，适合需要 SQL 查询的场景。

```sql
-- 节点表
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    public_key TEXT NOT NULL,
    registered_at INTEGER NOT NULL,
    last_active_at INTEGER NOT NULL,
    status TEXT NOT NULL,
    capabilities TEXT NOT NULL, -- JSON 格式
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_nodes_status ON nodes(status);
CREATE INDEX idx_nodes_last_active ON nodes(last_active_at);

-- 贡献表
CREATE TABLE contributions (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    compute_score REAL NOT NULL,
    duration_seconds INTEGER NOT NULL,
    samples_processed INTEGER NOT NULL,
    gpu_usage REAL,
    cpu_usage REAL,
    memory_mb INTEGER,
    network_mb INTEGER,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id)
);

CREATE INDEX idx_contributions_node ON contributions(node_id);
CREATE INDEX idx_contributions_timestamp ON contributions(timestamp);

-- 收益表
CREATE TABLE rewards (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    amount_lamports INTEGER NOT NULL,
    status TEXT NOT NULL,
    distributed_at INTEGER NOT NULL,
    transaction_signature TEXT,
    FOREIGN KEY (node_id) REFERENCES nodes(id)
);

CREATE INDEX idx_rewards_node ON rewards(node_id);
CREATE INDEX idx_rewards_status ON rewards(status);
```

### 2. KV 存储

KV 用于简单的键值对存储，适合：
- 配置数据
- 缓存
- 快速查找

### 3. Durable Objects

Durable Objects 提供长期进程能力：
- `NodeRegistry` - 节点注册表
- `TaskScheduler` - 任务调度器
- `ProofVerifier` - 零知识证明验证器

## 时间适配

Cloudflare Workers 不支持系统时间，需要使用时间钟方法：

```rust
use williw::workers::{WorkersClock, TimestampUtils};

// 创建时间钟
let clock = WorkersClock::new();

// 从请求头设置时间
let timestamp = TimestampUtils::extract_from_headers(&headers);
if let Some(ts) = timestamp {
    clock.set_timestamp(ts);
}

// 获取当前时间
let now_ms = clock.timestamp_ms();
let now_secs = clock.timestamp_secs();

// 检查超时
let is_timeout = clock.is_timeout(start_time, timeout_ms);
```

### 请求头时间

Workers 请求通常包含时间信息：
- `Date` - RFC 2822 格式的时间
- `X-Request-Start-Time` - 毫秒时间戳（自定义）

```javascript
// 在 JavaScript 中设置请求头
fetch(url, {
    headers: {
        'X-Request-Start-Time': Date.now().toString()
    }
});
```

## 部署步骤

### 1. 安装 Wrangler CLI

```bash
npm install -g @cloudflare/wrangler
```

### 2. 登录

```bash
wrangler login
```

### 3. 配置环境

编辑 `workers-config/wrangler.toml`：

```toml
name = "williw"
type = "rust"
account_id = "your-account-id"
zone_id = "your-zone-id"

[build]
command = "cargo build --target wasm32-unknown-unknown --release --features workers"
upload = { format = "wasm" }
dir = "wasm/pkg"
main = "./williw_wasm.js"

[[d1_databases]]
binding = "DB"
database_name = "williw_db"
database_id = "your-database-id"

[kv_namespaces]
{ binding = "NODES_STORE", id = "your-namespace-id" }

[durable_objects]
{ name = "NodeRegistry", class_name = "NodeRegistry" }
{ name = "TaskScheduler", class_name = "TaskScheduler" }

[triggers]
crons = ["*/5 * * * *"]
```

### 4. 创建 D1 数据库

```bash
# 创建数据库
wrangler d1 create williw_db

# 执行 SQL 脚本
wrangler d1 execute DB --file=./scripts/schema.sql

# 或者交互式执行
wrangler d1 execute DB --command="SELECT * FROM nodes"
```

### 5. 构建 WASM

```bash
cd wasm
./build-wasm.sh
# 或在 Windows 上
powershell ./build-wasm.ps1
```

### 6. 部署到 Workers

```bash
# 部署所有资源
wrangler deploy

# 只部署 Worker
wrangler deploy williw-edge-server

# 部署 KV 命名空间
wrangler kv:namespace create --binding=NODES_STORE
wrangler kv:namespace create --binding=TASKS_STORE
wrangler kv:namespace create --binding=PROOFS_STORE
```

## API 端点

### 节点管理

```
POST /api/nodes/register
Body: NodeInfo
Response: { status: "OK" }

POST /api/nodes/heartbeat?node_id=xxx
Response: { status: "OK" }

GET /api/nodes/list?active=true
Response: [NodeInfo]

GET /api/nodes/node?node_id=xxx
Response: NodeInfo
```

### 任务调度

```
POST /api/tasks/add
Body: Task
Response: { status: "OK", task_id: "xxx" }

POST /api/tasks/cancel?task_id=xxx
Response: { status: "OK" }

GET /api/tasks/list
Response: [Task]
```

### 算力贡献

```
POST /api/contributions/record
Body: ComputeContribution
Response: { status: "OK", contribution_id: "xxx" }

GET /api/contributions/stats?node_id=xxx
Response: ComputeStats
```

### 收益管理

```
POST /api/rewards/distribute
Body: [RewardDistribution]
Response: [{ transaction_signature: "xxx", success: true }]

GET /api/rewards/balance?address=xxx
Response: NodeWalletBalance
```

## 监控和维护

### 定时任务

配置了每 5 分钟执行的定时任务：

```toml
[triggers]
crons = ["*/5 * * * *"]
```

任务包括：
1. 清理过期节点
2. 清理过期 KV 条目
3. 记录统计信息
4. 健康检查

### 日志查看

```bash
# 查看 Worker 日志
wrangler tail

# 查看 D1 查询日志
wrangler d1 execute DB --command="SELECT * FROM logs ORDER BY created_at DESC LIMIT 100"
```

### 健康检查

```
GET /health
Response: {
    status: "healthy",
    timestamp: "2024-01-01T00:00:00Z",
    version: "1.0.0",
    features: {
        wasm: true,
        zk_proof: true,
        databases: true,
        d1: true,
        kv: true,
        durable_objects: true
    }
}
```

## 性能优化

### 1. KV 缓存策略

```javascript
// 设置带 TTL 的缓存
await env.NODES_STORE.put(
    `node:${nodeId}`,
    JSON.stringify(nodeInfo),
    { expirationTtl: 300 } // 5 分钟
);
```

### 2. D1 批量操作

```javascript
// 批量插入
const stmt = env.DB.prepare(
    `INSERT INTO contributions (id, node_id, task_id, compute_score, ...)
     VALUES (?, ?, ?, ...)`
);
await env.DB.batch([
    stmt.bind(...values1),
    stmt.bind(...values2),
    stmt.bind(...values3),
]);
```

### 3. Durable Object 连接复用

```javascript
// 复用 Durable Object 连接
const nodeRegistry = await env.NodeRegistry.get(id);
// 多次请求使用同一个实例
```

## 安全考虑

1. **认证**：所有 API 端点需要认证令牌
2. **速率限制**：使用 KV 实现速率限制
3. **数据验证**：验证所有输入数据
4. **加密**：敏感数据传输前加密

## 故障排查

### 常见问题

1. **WASM 加载失败**
   - 检查 `wasm/pkg` 目录是否存在
   - 验证构建输出是否正确

2. **D1 查询错误**
   - 检查数据库 ID 是否正确
   - 验证 SQL 语法

3. **时间不准确**
   - 确保请求头包含时间信息
   - 使用 `TimestampUtils::extract_from_headers()`

4. **Durable Object 超时**
   - 检查 `storage.put()` 调用是否正确
   - 减少 `waitUntil()` 队列大小

## 环境变量

```toml
[vars]
NODE_ENV = "production"
LOG_LEVEL = "info"
MAX_NODES = "1000"
ZK_PROOF_ENABLED = "true"
BASE_REWARD_LAMPORTS = "1000000"
MIN_SETTLEMENT_LAMPORTS = "10000000"

# 敏感变量（使用 wrangler secret）
# wrangler secret put SOLANA_PRIVATE_KEY
```

## 下一步

1. 实现 Solana 智能合约
2. 集成 Workers 与 Solana 网络
3. 添加实时监控
4. 优化 D1 查询性能

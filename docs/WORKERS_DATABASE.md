# Workers 数据库和时间适配模块

## 概述

本模块提供 Cloudflare Workers 环境的完整适配，包括：
1. **时间适配** - Workers 不支持系统时间，需要时间钟方法
2. **数据库支持** - D1, KV, Durable Objects 接口
3. **长期进程** - Durable Objects 实现有状态持久化

## 文件结构

```
src/workers/
├── mod.rs              # 模块入口
├── time.rs             # 时间钟适配器
├── db.rs               # 数据库接口定义
├── kv.rs               # KV 存储实现
├── d1.rs              # D1 数据库适配器
└── durable_objects.rs   # Durable Objects 实现

scripts/
└── schema.sql          # D1 数据库架构

workers-config/
└── wrangler.toml      # Workers 配置
```

## 时间适配（重要）

### 问题

Cloudflare Workers **不支持系统时间**，因此不能使用：
- `std::time::SystemTime::now()`
- `chrono::Utc::now()`
- 其他系统时钟

### 解决方案

使用 **WorkersClock** 时间钟接口：

```rust
use williw::workers::{WorkersClock, TimestampUtils};

// 创建时间钟
let clock = WorkersClock::new();

// 从请求头设置时间（重要！）
let headers = request.headers;
let timestamp = TimestampUtils::extract_from_headers(&headers);
if let Some(ts) = timestamp {
    clock.set_timestamp(ts);
}

// 获取当前时间
let now_ms = clock.timestamp_ms();
let now_secs = clock.timestamp_secs();

// 检查超时
let is_timeout = clock.is_timeout(start_time, 5000);
```

### 请求头时间

Workers 请求应包含时间信息：

**JavaScript 端:**
```javascript
fetch(url, {
    headers: {
        'X-Request-Start-Time': Date.now().toString()
    }
});
```

**Rust Workers 端:**
```rust
// 从请求头提取时间
use williw::workers::TimestampUtils;

let timestamp = TimestampUtils::extract_from_headers(&headers);
```

### 时间工具函数

```rust
use williw::workers::TimestampUtils;

// ISO 8601 格式化
let iso_str = TimestampUtils::format_iso8601(1704067200000);

// 解析 ISO 8601
let ts = TimestampUtils::parse_iso8601("2024-01-01T00:00:00Z")?;

// HTTP Date 解析
let ts = TimestampUtils::parse_http_date("Wed, 21 Oct 2015 07:28:00 GMT")?;
```

## 数据库支持

### 1. KV 存储

用于简单的键值对存储：

```rust
use williw::workers::{InMemoryKV, KVStorage};

let kv = InMemoryKV::new(1000); // 最大 1000 条目

// 设置值（带 TTL）
kv.put("node:123", data, Some(300)).await?; // 5 分钟过期

// 获取值
if let Some(value) = kv.get("node:123").await? {
    // 处理数据
}

// 列出键
let result = kv.list("node:", Some(100), None).await?;
for key in result.keys {
    println!("{}", key);
}

// 删除值
kv.delete("node:123").await?;

// 批量操作
kv.set_batch(&[("key1", data1), ("key2", data2)], None).await?;
let values = kv.get_batch(&["key1", "key2"]).await?;
```

### 2. D1 数据库

用于结构化数据，支持 SQL：

**初始化：**
```sql
-- 执行 schema.sql
wrangler d1 execute DB --file=./scripts/schema.sql
```

**使用示例：**
```rust
use williw::workers::D1Adapter;

let db = D1Adapter::new("DB".to_string());

// 执行查询
let results = db.query(
    "SELECT * FROM nodes WHERE status = ?",
    &[serde_json::json!("active")]
).await?;

// 执行语句
let affected = db.execute(
    "UPDATE nodes SET last_active_at = ? WHERE id = ?",
    &[serde_json::json!(now_ms), serde_json::json!(node_id)]
).await?;
```

### 3. Durable Objects

用于长期进程和共享状态：

```rust
use williw::workers::{NodeRegistry, NodeInfo, NodeStatus};

// 创建节点注册表
let registry = NodeRegistry::new("node-registry".to_string());

// 注册节点
let node = NodeInfo {
    node_id: "node-123".to_string(),
    address: "127.0.0.1:8080".to_string(),
    public_key: "public_key".to_string(),
    registered_at: 1704067200000,
    last_active_at: 1704067200000,
    status: NodeStatus::Active,
    capabilities: DeviceCapabilities { /* ... */ },
};
registry.register_node(node).await?;

// 更新心跳
registry.update_heartbeat("node-123").await?;

// 列出活跃节点
let active_nodes = registry.list_active_nodes().await?;
```

## 数据库架构

### D1 表结构

**nodes 表:**
```sql
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    public_key TEXT NOT NULL,
    registered_at INTEGER NOT NULL,
    last_active_at INTEGER NOT NULL,
    status TEXT NOT NULL,
    capabilities TEXT NOT NULL,           -- JSON
    total_compute_score REAL NOT NULL,
    total_rewards_lamports INTEGER NOT NULL,
    contribution_count INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

**contributions 表:**
```sql
CREATE TABLE contributions (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    compute_score REAL NOT NULL,
    duration_seconds INTEGER NOT NULL,
    samples_processed INTEGER NOT NULL,
    gpu_usage_percent REAL,
    memory_used_mb INTEGER,
    network_upload_mb INTEGER,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id)
);
```

**rewards 表:**
```sql
CREATE TABLE rewards (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    amount_lamports INTEGER NOT NULL,
    status TEXT NOT NULL,
    distributed_at INTEGER NOT NULL,
    transaction_signature TEXT,
    FOREIGN KEY (node_id) REFERENCES nodes(id)
);
```

**wallets 表:**
```sql
CREATE TABLE wallets (
    node_id TEXT PRIMARY KEY,
    wallet_address TEXT NOT NULL,
    sol_balance_lamports INTEGER NOT NULL,
    pending_rewards_lamports INTEGER NOT NULL,
    total_rewards_distributed_lamports INTEGER NOT NULL,
    last_updated_at INTEGER NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id)
);
```

## 部署配置

### wrangler.toml

```toml
name = "williw"
type = "rust"

# D1 数据库
[[d1_databases]]
binding = "DB"
database_name = "williw_db"
database_id = "your-database-id"

# KV 存储
[kv_namespaces]
{ binding = "NODES_STORE", id = "nodes-store-id" }
{ binding = "TASKS_STORE", id = "tasks-store-id" }
{ binding = "PROOFS_STORE", id = "proofs-store-id" }

# Durable Objects
[durable_objects]
{ name = "NodeRegistry", class_name = "NodeRegistry" }
{ name = "TaskScheduler", class_name = "TaskScheduler" }

# 定时任务
[triggers]
crons = ["*/5 * * * *"]  # 每 5 分钟
```

### 环境变量

```toml
[vars]
NODE_ENV = "production"
LOG_LEVEL = "info"
MAX_NODES = "1000"
ZK_PROOF_ENABLED = "true"
BASE_REWARD_LAMPORTS = "1000000"      # 0.001 SOL
MIN_SETTLEMENT_LAMPORTS = "10000000"  # 0.01 SOL
```

## API 端点

### 节点管理

```
POST   /api/nodes/register           注册节点
POST   /api/nodes/heartbeat          更新心跳
GET    /api/nodes/list               列出节点
GET    /api/nodes/node?node_id=xxx   获取节点信息
PUT    /api/nodes/status            更新节点状态
DELETE /api/nodes/node?node_id=xxx   删除节点
```

### 贡献管理

```
POST   /api/contributions/record      记录贡献
GET    /api/contributions/stats?node_id=xxx  获取统计
GET    /api/contributions/list?node_id=xxx   列出贡献
```

### 收益管理

```
POST   /api/rewards/distribute       分配收益
GET    /api/rewards/balance?address=xxx   查询余额
GET    /api/rewards/pending?node_id=xxx    查询待结算
POST   /api/rewards/settle         结算收益
```

### 任务调度

```
POST   /api/tasks/add                添加任务
GET    /api/tasks/next               获取下一个任务
POST   /api/tasks/cancel?task_id=xxx  取消任务
GET    /api/tasks/list               列出任务
```

## 使用示例

### 完整的节点生命周期

```rust
use williw::workers::{WorkersClock, TimestampUtils, InMemoryKV, NodeRegistry};

// 1. 设置时间钟
let clock = WorkersClock::new();
let headers = extract_request_headers(&request);
let timestamp = TimestampUtils::extract_from_headers(&headers);
if let Some(ts) = timestamp {
    clock.set_timestamp(ts);
}

// 2. 注册节点
let registry = NodeRegistry::new("node-registry".to_string());
let node = create_node_info();
registry.register_node(node).await?;

// 3. 定期更新心跳
registry.update_heartbeat("node-123").await?;

// 4. 记录贡献
let contribution = create_contribution();
report_contribution(contribution).await?;

// 5. 查询收益
let balance = query_wallet_balance(&wallet_address).await?;
```

### 定时任务维护

```javascript
// workers/worker.js
addEventListener('scheduled', async (event) => {
    // 1. 清理过期节点
    await cleanup_expired_nodes();

    // 2. 清理过期 KV 条目
    await cleanup_expired_kv_entries();

    // 3. 更新统计信息
    await update_statistics();

    // 4. 健康检查
    await health_check();
});
```

## 性能优化

### 1. KV 缓存策略

```rust
// 使用 TTL 自动清理
kv.put("node:123", data, Some(300)).await?; // 5 分钟

// 定期清理
kv.cleanup_expired();
```

### 2. D1 批量操作

```rust
// 批量插入
let stmt = "INSERT INTO contributions (...) VALUES (?, ?, ...)";
let values = [
    &[data1, data2, data3],
    &[data4, data5, data6],
];
db.execute_batch(stmt, &values).await?;
```

### 3. Durable Object 状态持久化

```rust
// 定期持久化
registry.persist().await?;

// 加载时恢复
registry.load().await?;
```

## 注意事项

1. **时间依赖**：所有时间操作必须通过 `WorkersClock`
2. **数据库选择**：
   - KV: 简单键值对，频繁访问
   - D1: 结构化数据，需要 SQL 查询
   - Durable Objects: 长期进程，共享状态
3. **缓存策略**：合理使用 TTL 减少数据库查询
4. **错误处理**：处理网络超时和数据库不可用情况
5. **并发控制**：Durable Objects 有访问限制，避免并发过多

## 故障排查

### 问题：时间不准确

**症状：** 超时检测失败，时间戳错误

**解决：**
```rust
// 确保从请求头设置时间
let timestamp = TimestampUtils::extract_from_headers(&headers);
clock.set_timestamp(timestamp.unwrap_or_else(|| TimestampUtils::now_millis()));
```

### 问题：KV 缓存溢出

**症状：** `KV 存储已满` 错误

**解决：**
```rust
// 增加缓存大小
let kv = InMemoryKV::new(10000); // 10000 条目

// 或启用自动清理
kv.cleanup_expired();
```

### 问题：D1 查询慢

**症状：** 响应时间长

**解决：**
```sql
-- 添加索引
CREATE INDEX idx_nodes_status ON nodes(status);
CREATE INDEX idx_contributions_timestamp ON contributions(timestamp);

-- 使用 LIMIT
SELECT * FROM nodes WHERE status = ? LIMIT 100;
```

## 下一步

1. 实现真实的 Workers KV 绑定
2. 集成 D1 数据库查询
3. 添加错误重试机制
4. 实现监控和告警
5. 优化缓存策略

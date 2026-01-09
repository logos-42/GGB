# Cloudflare Workers 部署指南

## 前置条件

确保你已经安装了以下工具：

1. **Node.js 和 npm** - JavaScript 包管理器
2. **Rust 和 Cargo** - Rust 工具链
3. **wasm-pack** - WebAssembly 打包工具
4. **Wrangler** - Cloudflare Workers CLI

### 安装工具

```bash
# 1. 安装 Node.js (如果未安装)
# 访问 https://nodejs.org/ 下载并安装

# 2. 安装 Rust (如果未安装)
# 访问 https://rustup.rs/ 下载并安装

# 3. 安装 wasm-pack
cargo install wasm-pack

# 4. 安装 wrangler
npm install -g wrangler

# 5. 登录 Cloudflare
wrangler login
```

---

## 快速部署

### Windows

运行部署脚本：

```cmd
deploy_workers.bat
```

### Linux/macOS

运行部署脚本：

```bash
chmod +x deploy_workers.sh
./deploy_workers.sh
```

---

## 手动部署步骤

如果自动部署脚本失败，可以按照以下步骤手动部署：

### 步骤 1: 构建 WASM

```bash
cd wasm

# 清理旧的构建产物
rm -rf pkg  # Linux/macOS
# 或
rmdir /s /q pkg  # Windows

# 使用 wasm-pack 构建
wasm-pack build --target web --out-dir pkg

# 检查构建产物
ls pkg/
# 应该看到：
# - williw_wasm.js
# - williw_wasm_bg.wasm
# - williw_wasm.d.ts
# - package.json
```

### 步骤 2: 配置 Wrangler

编辑 `workers-config/wrangler.toml`，确保配置正确：

```toml
name = "williw"
type = "rust"
account_id = "your-account-id"  # 替换为你的账户 ID
workers_dev = true

[build]
command = "cargo build --target wasm32-unknown-unknown --release --features workers"
upload = { format = "wasm" }

[build.upload]
dir = "wasm/pkg"
main = "./williw_wasm.js"

# KV 命名空间
[kv_namespaces]
{ binding = "NODES_STORE", id = "your-nodes-store-id" }
{ binding = "TASKS_STORE", id = "your-tasks-store-id" }
{ binding = "PROOFS_STORE", id = "your-proofs-store-id" }

# D1 数据库
[[d1_databases]]
binding = "DB"
database_name = "williw_db"
database_id = "your-database-id"

# Durable Objects
[durable_objects]
{ name = "NodeRegistry", class_name = "NodeRegistry" }
{ name = "TaskScheduler", class_name = "TaskScheduler" }
{ name = "ProofVerifier", class_name = "ProofVerifier" }
```

### 步骤 3: 创建资源

#### 创建 KV 命名空间

```bash
wrangler kv:namespace create "NODES_STORE" --preview
wrangler kv:namespace create "TASKS_STORE" --preview
wrangler kv:namespace create "PROOFS_STORE" --preview
```

复制输出的 ID，更新到 `wrangler.toml` 中。

#### 创建 D1 数据库

```bash
# 创建数据库
wrangler d1 create williw_db

# 执行数据库架构
wrangler d1 execute williw_db --file=./scripts/schema.sql
```

复制输出的 database_id，更新到 `wrangler.toml` 中。

### 步骤 4: 部署

```bash
cd workers-config

# 部署到开发环境
wrangler deploy

# 或部署到生产环境
wrangler deploy --env production
```

---

## 测试部署

### 健康检查

```bash
curl https://williw.<your-account-id>.workers.dev/health
```

期望响应：

```json
{
  "status": "healthy",
  "timestamp": "2024-01-09T10:00:00Z",
  "version": "1.0.0",
  "features": {
    "wasm": true,
    "zk_proof": true,
    "algorithms": true,
    "edge_server": true
  }
}
```

### API 测试

```bash
# 测试状态端点
curl https://williw.<your-account-id>.workers.dev/api/status

# 测试节点注册
curl -X POST https://williw.<your-account-id>.workers.dev/api/nodes \
  -H "Content-Type: application/json" \
  -d '{"node_id": "node-123", "public_key": "abc123"}'
```

---

## 查看日志

实时查看 Worker 日志：

```bash
wrangler tail
```

---

## 常见问题

### 1. wasm-pack 构建失败

**错误**: `error: linker link.exe not found`

**解决**: 确保安装了 MSVC 工具链：

```bash
rustup target add wasm32-unknown-unknown
```

### 2. wrangler 部署失败：找不到 wasm 文件

**错误**: `error: Could not find file wasm/pkg/williw_wasm.js`

**解决**: 确保 wasm-pack 构建成功，并检查文件名是否正确。

### 3. KV/D1 资源未配置

**错误**: `error: KV namespace not found`

**解决**: 按照"创建资源"步骤创建相应的资源，并更新 `wrangler.toml`。

### 4. Worker 初始化失败

**错误**: `WASM initialization failed`

**解决**: 检查 wasm 文件是否完整，确保 `williw_wasm_bg.wasm` 存在。

---

## 资源管理

### 查看 Worker

```bash
wrangler workers list
```

### 删除 Worker

```bash
wrangler delete williw
```

### 查看详细信息

```bash
wrangler tail
```

---

## 性能优化

### 1. 启用缓存

在 `wrangler.toml` 中添加：

```toml
[build]
command = "cargo build --target wasm32-unknown-unknown --release --features workers"
upload = { format = "wasm" }

[build.upload]
dir = "wasm/pkg"
main = "./williw_wasm.js"
mode = "dist"
```

### 2. 压缩 WASM

WASM 文件已经通过 `opt-level = "z"` 和 `lto = true` 优化。

### 3. 使用 Edge Runtime

Workers 默认运行在 Edge Runtime，无需额外配置。

---

## 监控和调试

### 查看日志

```bash
# 实时查看
wrangler tail

# 过滤错误
wrangler tail --format pretty | grep ERROR

# 保存到文件
wrangler tail > workers.log
```

### 性能监控

访问 Cloudflare Dashboard：
https://dash.cloudflare.com/

查看 Analytics 和 Metrics。

---

## 更新部署

每次更新代码后：

1. 重新构建 WASM
2. 重新部署

```bash
cd wasm
wasm-pack build --target web --out-dir pkg
cd ../workers-config
wrangler deploy
```

---

## 环境变量

在 `wrangler.toml` 中配置：

```toml
[vars]
NODE_ENV = "production"
LOG_LEVEL = "info"
MAX_NODES = "1000"
ZK_PROOF_ENABLED = "true"
```

或在命令行中设置：

```bash
wrangler deploy --env production
```

---

## 支持

如有问题，请查看：
- Cloudflare Workers 文档: https://developers.cloudflare.com/workers/
- Wrangler CLI 文档: https://developers.cloudflare.com/workers/wrangler/
- Rust WASM 文档: https://rustwasm.github.io/docs/wasm-pack/

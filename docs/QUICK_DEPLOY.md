# Workers 快速部署指南

## 当前状态

✅ **已完成的配置**:
- wrangler 已安装并登录 (yuanjieliu65@gmail.com)
- 账户 ID: a13e8fd1b7246c7105fbbab04f5d9b8d
- wrangler.toml 已更新正确的账户 ID

## 快速部署步骤

### 方法 1: 使用一键部署脚本

#### Windows
```cmd
deploy_workers.bat
```

#### Linux/macOS
```bash
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```

### 方法 2: 手动部署

#### 步骤 1: 构建 WASM

```bash
cd wasm

# Windows
rmdir /s /q pkg
mkdir pkg
wasm-pack build --target web --out-dir pkg

# Linux/macOS
rm -rf pkg
mkdir pkg
wasm-pack build --target web --out-dir pkg
```

#### 步骤 2: 部署

```bash
cd workers-config
wrangler deploy
```

---

## 预期的构建产物

构建成功后，`wasm/pkg/` 目录应包含：

- `williw_wasm.js` - JavaScript 绑定
- `williw_wasm_bg.wasm` - WebAssembly 二进制
- `williw_wasm.d.ts` - TypeScript 类型定义
- `package.json` - NPM 包配置

---

## 部署成功后

### 测试健康检查

```bash
curl https://williw.workers.dev/health
```

### 查看日志

```bash
wrangler tail
```

### 访问 Worker

- URL: https://williw.workers.dev
- 健康检查: https://williw.workers.dev/health

---

## 常见问题

### 问题 1: wasm-pack 构建失败

**错误**: `error: linking with link.exe failed`

**解决**: 安装 Microsoft C++ Build Tools 或使用预编译的二进制

### 问题 2: 找不到 williw_wasm.js

**错误**: `Could not find file wasm/pkg/williw_wasm.js`

**解决**:
1. 检查构建产物文件名
2. 如果文件名是 `ggb_wasm.js`，需要修改 `wasm/Cargo.toml`:
   ```toml
   [lib]
   name = "williw_wasm"
   ```

### 问题 3: 部署失败 - KV 命名空间不存在

**错误**: `KV namespace not found`

**解决**: 创建 KV 命名空间
```bash
wrangler kv:namespace create "NODES_STORE" --preview
wrangler kv:namespace create "TASKS_STORE" --preview
wrangler kv:namespace create "PROOFS_STORE" --preview
```

更新 `wrangler.toml` 中的 KV ID。

### 问题 4: D1 数据库不存在

**错误**: `D1 database not found`

**解决**: 创建 D1 数据库
```bash
wrangler d1 create williw_db
wrangler d1 execute williw_db --file=./scripts/schema.sql
```

更新 `wrangler.toml` 中的 database_id。

---

## 资源创建命令

### KV 命名空间
```bash
wrangler kv:namespace create "NODES_STORE"
wrangler kv:namespace create "TASKS_STORE"
wrangler kv:namespace create "PROOFS_STORE"
```

### D1 数据库
```bash
wrangler d1 create williw_db
wrangler d1 execute williw_db --file=./scripts/schema.sql
```

### 获取账户信息
```bash
wrangler whoami
```

---

## 更新部署

修改代码后重新部署:

```bash
# 重新构建
cd wasm
wasm-pack build --target web --out-dir pkg

# 重新部署
cd ../workers-config
wrangler deploy
```

---

## 监控和管理

### 查看日志
```bash
wrangler tail
```

### 查看所有 Workers
```bash
wrangler workers list
```

### 删除 Worker
```bash
wrangler delete williw
```

---

## 配置文件

主要配置文件:
- `workers-config/wrangler.toml` - Worker 配置
- `wasm/Cargo.toml` - WASM 包配置
- `workers/worker.js` - Worker 入口

---

## 下一步

部署成功后，可以:
1. 测试健康检查端点
2. 实现完整的 API 接口
3. 配置 Durable Objects
4. 集成 Solana 区块链
5. 设置监控和告警

---

## 需要帮助?

查看详细文档:
- `docs/WORKERS_DEPLOY_GUIDE.md` - 完整部署指南
- `docs/WORKERS_DATABASE.md` - 数据库配置
- `docs/SOLANA_MODULE.md` - Solana 集成

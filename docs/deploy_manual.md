# Workers 部署手动指南

## 📋 当前状态

✅ **已完成**:
- wrangler 已安装并登录
- 账户: yuanjieliu65@gmail.com
- 账户 ID: a13e8fd1b7246c7105fbbab04f5d9b8d
- wrangler.toml 已配置

---

## 🚀 方法 1: 快速部署（推荐）

### 步骤 1: 构建 WASM

打开命令行，运行：

```bash
cd d:/AI/去中心化训练/wasm
rmdir /s /q pkg
mkdir pkg
wasm-pack build --target web --out-dir pkg
```

如果构建成功，会看到：

```
✅ WASM 构建完成！
📦 构建产物:
  - williw_wasm.js
  - williw_wasm_bg.wasm
  - williw_wasm.d.ts
```

### 步骤 2: 部署

```bash
cd d:/AI/去中心化训练/workers-config
wrangler deploy
```

如果部署成功，会看到：

```
✨ Successfully published your Worker to
  https://williw.workers.dev
```

### 步骤 3: 测试

```bash
# 测试健康检查
curl https://williw.workers.dev/health

# 查看日志
wrangler tail
```

---

## 🔧 方法 2: 手动部署（如果方法 1 失败）

### 步骤 1: 准备 Worker

如果 WASM 构建失败，可以先部署一个简单的 Worker：

1. 创建简单的 Worker 脚本：

```javascript
// workers-config/worker-simple.js
export default {
  async fetch(request) {
    return new Response(JSON.stringify({
      status: "healthy",
      message: "Williw Worker is running!",
      timestamp: new Date().toISOString()
    }), {
      headers: { 'Content-Type': 'application/json' }
    });
  }
};
```

2. 修改 wrangler.toml：

```toml
name = "williw"
type = "javascript"
account_id = "a13e8fd1b7246c7105fbbab04f5d9b8d"
workers_dev = true

[build.upload]
format = "modules"
main = "./worker-simple.js"
```

3. 部署：

```bash
cd workers-config
wrangler deploy
```

### 步骤 2: 创建必要的资源

#### 创建 KV 命名空间

```bash
# 每次运行后，复制输出的 ID 到 wrangler.toml
wrangler kv:namespace create "NODES_STORE"
wrangler kv:namespace create "TASKS_STORE"
wrangler kv:namespace create "PROOFS_STORE"
```

更新 `wrangler.toml`:

```toml
[kv_namespaces]
{ binding = "NODES_STORE", id = "<复制的ID>" }
{ binding = "TASKS_STORE", id = "<复制的ID>" }
{ binding = "PROOFS_STORE", id = "<复制的ID>" }
```

#### 创建 D1 数据库

```bash
# 创建数据库
wrangler d1 create williw_db

# 复制输出的 database_id

# 执行架构
wrangler d1 execute williw_db --file=../scripts/schema.sql
```

更新 `wrangler.toml`:

```toml
[[d1_databases]]
binding = "DB"
database_name = "williw_db"
database_id = "<复制的ID>"
```

### 步骤 3: 重新部署

```bash
wrangler deploy
```

---

## 📝 部署检查清单

- [ ] wrangler 已登录
- [ ] KV 命名空间已创建并配置
- [ ] D1 数据库已创建并初始化
- [ ] WASM 模块已构建（或使用简化版 Worker）
- [ ] wrangler.toml 配置正确
- [ ] 部署成功
- [ ] 健康检查通过
- [ ] 日志正常输出

---

## 🔍 常见问题

### Q1: wasm-pack 构建失败

**问题**: `error: linking with link.exe failed`

**解决方案 A**: 安装 Microsoft C++ Build Tools
1. 下载: https://visualstudio.microsoft.com/downloads/
2. 安装 "Desktop development with C++"

**解决方案 B**: 使用简化版 Worker（见方法 2）

### Q2: KV 命名空间不存在

**问题**: `error: KV namespace not found`

**解决**:
```bash
wrangler kv:namespace create "NODES_STORE"
wrangler kv:namespace create "TASKS_STORE"
wrangler kv:namespace create "PROOFS_STORE"
```

复制输出的 ID，更新到 `wrangler.toml`。

### Q3: D1 数据库不存在

**问题**: `error: D1 database not found`

**解决**:
```bash
wrangler d1 create williw_db
wrangler d1 execute williw_db --file=../scripts/schema.sql
```

### Q4: 权限不足

**问题**: `error: insufficient permissions`

**解决**:
```bash
wrangler logout
wrangler login
```

### Q5: 找不到 williw_wasm.js

**问题**: `Could not find file wasm/pkg/williw_wasm.js`

**解决方案 A**: 检查构建产物
```bash
cd wasm/pkg
dir
```

如果文件名不同（如 `ggb_wasm.js`），修改 `wrangler.toml`:

```toml
[build.upload]
main = "./ggb_wasm.js"  # 改为实际文件名
```

**解决方案 B**: 使用简化版 Worker（见方法 2）

---

## 📊 部署后测试

### 健康检查

```bash
curl https://williw.workers.dev/health
```

期望输出:
```json
{
  "status": "healthy",
  "message": "Williw Worker is running!",
  "timestamp": "2024-01-09T10:00:00Z"
}
```

### 查看日志

```bash
wrangler tail
```

### 访问 Dashboard

https://dash.cloudflare.com/

查看:
- Workers 列表
- 日志和分析
- KV 存储
- D1 数据库

---

## 🎯 下一步

部署成功后，可以：

1. **完善 API**
   - 实现节点注册
   - 实现任务调度
   - 实现算力跟踪

2. **集成 Solana**
   - 实现节点收益分配
   - 实现算力贡献记录

3. **优化性能**
   - 启用缓存
   - 优化 WASM 大小
   - 配置 CDN

4. **监控**
   - 设置告警
   - 配置日志分析
   - 性能监控

---

## 📚 相关文档

- [Workers 完整部署指南](./docs/WORKERS_DEPLOY_GUIDE.md)
- [Workers 数据库配置](./docs/WORKERS_DATABASE.md)
- [Solana 模块文档](./docs/SOLANA_MODULE.md)
- [快速部署指南](./docs/QUICK_DEPLOY.md)

---

## 💻 有用的命令

```bash
# 构建 WASM
cd wasm
wasm-pack build --target web --out-dir pkg

# 部署
cd ../workers-config
wrangler deploy

# 查看日志
wrangler tail

# 查看 Workers
wrangler workers list

# 删除 Worker
wrangler delete williw

# 查看 KV 内容
wrangler kv:key list --binding=NODES_STORE

# 查询 D1
wrangler d1 execute williw_db --command="SELECT * FROM nodes LIMIT 5"
```

---

## 🆘 需要帮助？

1. 查看 Cloudflare Workers 文档: https://developers.cloudflare.com/workers/
2. 查看 Wrangler 文档: https://developers.cloudflare.com/workers/wrangler/
3. 检查日志: `wrangler tail`
4. 查看错误信息并搜索解决方案

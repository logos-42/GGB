# P2P 前端集成指南

本指南详细说明如何将 P2P 模型分发系统集成到前端桌面应用中，实现节点 ID 显示、复制和添加功能。

## 🎯 功能概述

### 核心功能
- ✅ **自动启动 P2P 服务** - 应用启动时自动初始化 iroh 节点
- ✅ **节点 ID 显示** - 在前端界面显示本地节点 ID
- ✅ **一键复制** - 支持将节点 ID 复制到剪贴板
- ✅ **节点添加** - 支持手动输入或从剪贴板添加远程节点
- ✅ **连接状态监控** - 实时显示连接状态和统计信息
- ✅ **WebAssembly 集成** - 支持 WASM 前端与 Rust 后端交互

### 技术架构
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   前端界面      │◄──►│  WebAssembly     │◄──►│  Rust 后端      │
│   (HTML/JS)     │    │   接口层         │    │  P2P 管理器     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   iroh 网络     │
                       │   P2P 通信      │
                       └─────────────────┘
```

## 🚀 快速开始

### 1. 添加依赖

在 `Cargo.toml` 中添加必要的依赖：

```toml
[dependencies]
# P2P 相关
iroh = "0.8"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# WebAssembly 支持 (可选)
wasm-bindgen = "0.2"
web-sys = "0.3"
js-sys = "0.3"
wasm-bindgen-futures = "0.4"

# 时间和 UUID
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

### 2. 在主应用中集成

```rust
use williw::comms::p2p_app_integration::{quick_start, P2PAppFactory};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 方式1：快速启动（推荐）
    quick_start().await?;
    
    // 方式2：自定义配置
    // let app = P2PAppFactory::create_custom(
    //     "我的应用".to_string(),
    //     "1.0.0".to_string(),
    // );
    // app.start().await?;
    // app.run().await?;
    
    Ok(())
}
```

### 3. 前端界面集成

#### HTML 页面示例

```html
<!DOCTYPE html>
<html>
<head>
    <title>P2P 节点管理</title>
    <!-- 引入样式 -->
    <link rel="stylesheet" href="p2p_manager.css">
</head>
<body>
    <!-- 本地节点信息 -->
    <div class="card">
        <h2>本地节点</h2>
        <div class="node-id-container">
            <div class="node-id" id="localNodeId">正在加载...</div>
            <button onclick="copyNodeId()">📋 复制节点 ID</button>
        </div>
    </div>

    <!-- 远程节点管理 -->
    <div class="card">
        <h2>远程节点</h2>
        <div class="add-node-form">
            <input type="text" id="nodeIdInput" placeholder="输入节点 ID">
            <button onclick="addRemoteNode()">➕ 添加节点</button>
            <button onclick="addNodeFromClipboard()">📋 从剪贴板添加</button>
        </div>
        <div id="nodeList"></div>
    </div>

    <!-- WebAssembly 模块 -->
    <script>
        // 初始化 WebAssembly 接口
        let p2pInterface = null;

        async function initializeP2P() {
            try {
                // 加载 WebAssembly 模块
                p2pInterface = new P2PWebInterface();
                await p2pInterface.initialize();
                
                // 获取本地节点 ID
                const nodeId = await p2pInterface.get_local_node_id();
                document.getElementById('localNodeId').textContent = nodeId;
                
                console.log('P2P 服务已就绪');
            } catch (error) {
                console.error('P2P 初始化失败:', error);
            }
        }

        // 复制节点 ID
        async function copyNodeId() {
            try {
                await p2pInterface.copy_node_id();
                await navigator.clipboard.writeText(
                    document.getElementById('localNodeId').textContent
                );
                alert('节点 ID 已复制到剪贴板');
            } catch (error) {
                console.error('复制失败:', error);
            }
        }

        // 添加远程节点
        async function addRemoteNode() {
            const nodeId = document.getElementById('nodeIdInput').value.trim();
            if (!nodeId) {
                alert('请输入节点 ID');
                return;
            }

            try {
                await p2pInterface.add_remote_node(nodeId, []);
                document.getElementById('nodeIdInput').value = '';
                updateNodeList();
                alert('节点添加成功');
            } catch (error) {
                console.error('添加节点失败:', error);
                alert('添加节点失败: ' + error.message);
            }
        }

        // 页面加载时初始化
        document.addEventListener('DOMContentLoaded', initializeP2P);
    </script>
</body>
</html>
```

## 📋 API 参考

### Rust 后端 API

#### P2PFrontendManager

```rust
use williw::comms::p2p_frontend_manager::P2PFrontendManager;

// 创建管理器
let manager = P2PFrontendManager::new().await?;

// 获取本地节点 ID
let node_id = manager.local_node_id();

// 获取本地节点信息
let local_info = manager.get_local_node_info().await?;

// 添加远程节点
manager.add_remote_node(
    "12D3KooW...".to_string(),
    vec!["/ip4/127.0.0.1/tcp/9236".to_string()],
).await?;

// 移除节点
manager.remove_node("12D3KooW...").await?;

// 复制节点 ID
manager.copy_node_id().await?;

// 获取连接统计
let stats = manager.get_connection_stats().await?;
```

#### FFI 函数

```c
// 获取本地节点 ID
char* p2p_get_local_node_id();

// 复制节点 ID
bool p2p_copy_node_id();

// 添加远程节点
bool p2p_add_remote_node(const char* node_id);

// 从剪贴板添加节点
bool p2p_add_node_from_clipboard();

// 获取前端状态 JSON
char* p2p_get_frontend_state();

// 释放字符串内存
void p2p_free_string(char* ptr);
```

### JavaScript API

#### WebAssembly 接口

```javascript
// 创建 Web 接口
const p2pInterface = new P2PWebInterface();

// 初始化
await p2pInterface.initialize();

// 获取本地节点 ID
const nodeId = await p2pInterface.get_local_node_id();

// 复制节点 ID
await p2pInterface.copy_node_id();

// 添加远程节点
await p2pInterface.add_remote_node(nodeId, addresses);

// 移除节点
await p2pInterface.remove_node(nodeId);

// 获取连接统计
const stats = await p2pInterface.get_connection_stats();
```

## 🔧 高级配置

### 自定义节点配置

```rust
use williw::comms::p2p_frontend_manager::P2PFrontendManager;

// 创建自定义配置的管理器
let mut manager = P2PFrontendManager::new().await?;

// 启动 P2P 服务
manager.start_p2p_service().await?;

// 添加引导节点
manager.add_remote_node(
    "12D3KooWBootstrapNode".to_string(),
    vec![
        "/ip4/104.131.131.82/tcp/4001/p2p/12D3KooWBootstrapNode".to_string(),
    ],
).await?;
```

### WebAssembly 编译

```bash
# 编译 WebAssembly 目标
cargo build --target wasm32-unknown-unknown --release

# 绑定 JavaScript
wasm-bindgen --target web --out-dir pkg --no-typescript \
    target/wasm32-unknown-unknown/release/williw.wasm
```

## 🎨 前端定制

### 样式定制

```css
/* 自定义节点 ID 显示样式 */
.node-id {
    font-family: 'Courier New', monospace;
    font-size: 0.9rem;
    background: #f8f9fa;
    border: 2px solid #e9ecef;
    border-radius: 8px;
    padding: 12px;
    word-break: break-all;
}

/* 自定义按钮样式 */
.btn-primary {
    background: linear-gradient(135deg, #667eea, #764ba2);
    color: white;
    border: none;
    border-radius: 8px;
    padding: 10px 20px;
    cursor: pointer;
    transition: all 0.3s ease;
}

.btn-primary:hover {
    transform: translateY(-2px);
    box-shadow: 0 5px 15px rgba(102, 126, 234, 0.4);
}
```

### 主题定制

```javascript
// 支持深色主题
const prefersDarkScheme = window.matchMedia('(prefers-color-scheme: dark)');

if (prefersDarkScheme.matches) {
    document.body.classList.add('dark-theme');
}

// 主题切换
function toggleTheme() {
    document.body.classList.toggle('dark-theme');
}
```

## 🐛 故障排除

### 常见问题

#### 1. WebAssembly 模块加载失败
```
错误: WebAssembly 模块加载失败
```

**解决方案:**
- 确保 `wasm-bindgen` 版本兼容
- 检查 `wasm32-unknown-unknown` 目标是否安装
- 验证 WebAssembly 文件路径正确

#### 2. FFI 函数调用失败
```
错误: 无法调用 FFI 函数
```

**解决方案:**
- 确保 FFI 函数使用 `#[no_mangle]` 标记
- 检查函数签名是否匹配
- 验证字符串内存管理正确

#### 3. 节点连接失败
```
错误: 节点连接超时
```

**解决方案:**
- 检查网络连接
- 验证节点 ID 格式正确
- 确认防火墙设置允许 P2P 连接

### 调试技巧

#### 启用详细日志
```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();
```

#### 前端调试
```javascript
// 启用详细控制台日志
console.log('P2P 状态:', await p2pInterface.get_connection_stats());

// 监听错误
window.addEventListener('error', (event) => {
    console.error('全局错误:', event.error);
});
```

## 📚 示例项目

### 完整示例结构
```
project/
├── src/
│   ├── main.rs                 # 主应用入口
│   └── comms/
│       ├── p2p_frontend_manager.rs
│       ├── p2p_web_integration.rs
│       └── p2p_app_integration.rs
├── frontend/
│   ├── p2p_manager.html       # 前端界面
│   ├── p2p_manager.css         # 样式文件
│   └── p2p_manager.js          # JavaScript 逻辑
├── pkg/                        # WebAssembly 输出
│   ├── williw.js
│   ├── williw_bg.js
│   └── williw.wasm
└── Cargo.toml
```

### 运行示例
```bash
# 1. 编译 Rust 代码
cargo build --release

# 2. 编译 WebAssembly
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/williw.wasm

# 3. 启动本地服务器
python -m http.server 8080

# 4. 访问前端界面
# http://localhost:8080/frontend/p2p_manager.html
```

## 🔮 未来规划

### 计划功能
- [ ] **节点发现** - 自动发现网络中的其他节点
- [ ] **文件传输** - 集成完整的 P2P 文件传输功能
- [ ] **加密通信** - 端到端加密支持
- [ ] **移动端支持** - React Native/Flutter 集成
- [ ] **网络拓扑可视化** - 图形化显示网络连接

### 性能优化
- [ ] **连接池管理** - 优化节点连接管理
- [ ] **缓存机制** - 减少重复计算和查询
- [ ] **批量操作** - 支持批量添加/删除节点
- [ ] **懒加载** - 按需加载节点信息

## 📞 支持

如果遇到问题或需要帮助，请：

1. 查看本文档的故障排除部分
2. 检查项目的 GitHub Issues
3. 联系开发团队获取技术支持

---

**注意**: 本指南基于 v1.0.0 版本编写，某些功能可能需要更新到最新版本才能使用。

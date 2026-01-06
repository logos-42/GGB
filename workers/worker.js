// Cloudflare Workers入口点

import { WorkersApp } from '../pkg/ggb_wasm.js';

// 初始化WASM模块
let wasmApp = null;

// 初始化函数
async function initWasm() {
    try {
        // 加载WASM模块
        const wasm = await import('../pkg/ggb_wasm_bg.wasm');
        
        // 创建配置
        const config = {
            edge_server: {
                name: "ggb-edge-server",
                max_nodes: 1000,
                heartbeat_timeout_secs: 60,
                default_matching_strategy: "Hybrid",
                enable_geo_matching: true,
                enable_performance_matching: true,
            },
            algorithms: {
                default_algorithm: "Hybrid",
                max_iterations: 100,
                convergence_threshold: 0.000001,
                enable_parallel: true,
            },
            network: {
                enable_websocket: true,
                max_connections: 100,
                connection_timeout_ms: 5000,
                enable_compression: true,
                enable_encryption: true,
            },
            storage: {
                enable_kv: true,
                enable_durable_objects: true,
                max_cache_size: 1000,
                cache_expiration_secs: 300,
            },
            enable_zk_proof: true,
            max_concurrent_requests: 100,
            request_timeout_ms: 30000,
        };
        
        // 创建应用实例
        wasmApp = new WorkersApp(config);
        
        console.log("WASM应用初始化成功");
        return true;
    } catch (error) {
        console.error("WASM初始化失败:", error);
        return false;
    }
}

// 处理请求
async function handleRequest(request) {
    // 如果WASM未初始化，尝试初始化
    if (!wasmApp) {
        const initialized = await initWasm();
        if (!initialized) {
            return new Response("WASM初始化失败", { status: 500 });
        }
    }
    
    try {
        // 将Fetch API的Request转换为WASM兼容的格式
        const wasmRequest = await convertRequest(request);
        
        // 调用WASM处理请求
        const wasmResponse = await wasmApp.handle_request(wasmRequest);
        
        // 将WASM响应转换为Fetch API的Response
        return convertResponse(wasmResponse);
    } catch (error) {
        console.error("请求处理失败:", error);
        return new Response(`内部服务器错误: ${error.message}`, { status: 500 });
    }
}

// 转换请求格式
async function convertRequest(request) {
    const url = new URL(request.url);
    
    return {
        method: request.method,
        path: url.pathname,
        query: Object.fromEntries(url.searchParams),
        headers: Object.fromEntries(request.headers),
        body: request.method !== 'GET' && request.method !== 'HEAD' 
            ? new Uint8Array(await request.arrayBuffer())
            : null,
    };
}

// 转换响应格式
function convertResponse(wasmResponse) {
    const headers = new Headers(wasmResponse.headers || {});
    
    return new Response(wasmResponse.body, {
        status: wasmResponse.status || 200,
        headers: headers,
    });
}

// 处理WebSocket连接
async function handleWebSocket(request) {
    try {
        // 获取WebSocket连接
        const webSocketPair = new WebSocketPair();
        const [client, server] = Object.values(webSocketPair);
        
        // 接受连接
        server.accept();
        
        // 设置消息处理器
        server.addEventListener('message', async (event) => {
            try {
                // 处理WebSocket消息
                await handleWebSocketMessage(server, event.data);
            } catch (error) {
                console.error("WebSocket消息处理失败:", error);
                server.close(1011, "内部错误");
            }
        });
        
        // 设置关闭处理器
        server.addEventListener('close', (event) => {
            console.log(`WebSocket连接关闭: code=${event.code}, reason=${event.reason}`);
        });
        
        // 设置错误处理器
        server.addEventListener('error', (error) => {
            console.error("WebSocket错误:", error);
        });
        
        return new Response(null, {
            status: 101,
            webSocket: client,
        });
    } catch (error) {
        console.error("WebSocket处理失败:", error);
        return new Response("WebSocket连接失败", { status: 500 });
    }
}

// 处理WebSocket消息
async function handleWebSocketMessage(webSocket, data) {
    // 这里实现WebSocket消息处理逻辑
    // 暂时简单回显
    if (typeof data === 'string') {
        webSocket.send(`收到消息: ${data}`);
    } else {
        webSocket.send("收到二进制消息");
    }
}

// 主事件处理器
addEventListener('fetch', (event) => {
    const request = event.request;
    const url = new URL(request.url);
    
    // 检查是否为WebSocket升级请求
    if (request.headers.get('Upgrade') === 'websocket') {
        event.respondWith(handleWebSocket(request));
        return;
    }
    
    // 处理HTTP请求
    event.respondWith(handleRequest(request));
});

// 定时任务（每5分钟执行一次）
addEventListener('scheduled', (event) => {
    event.waitUntil(handleScheduledEvent(event));
});

// 处理定时事件
async function handleScheduledEvent(event) {
    try {
        console.log("执行定时任务:", event.cron);
        
        // 清理过期节点
        if (wasmApp) {
            await wasmApp.cleanup_expired_nodes();
        }
        
        // 记录统计信息
        await logStatistics();
        
        console.log("定时任务执行完成");
    } catch (error) {
        console.error("定时任务执行失败:", error);
    }
}

// 记录统计信息
async function logStatistics() {
    if (!wasmApp) {
        return;
    }
    
    try {
        const stats = await wasmApp.get_stats();
        console.log("系统统计:", JSON.stringify(stats, null, 2));
        
        // 可以在这里将统计信息发送到监控服务
        // await sendToMonitoring(stats);
    } catch (error) {
        console.error("统计信息获取失败:", error);
    }
}

// 健康检查端点
addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);
    
    if (url.pathname === '/health') {
        event.respondWith(handleHealthCheck());
    }
});

// 处理健康检查
function handleHealthCheck() {
    const healthStatus = {
        status: wasmApp ? "healthy" : "initializing",
        timestamp: new Date().toISOString(),
        version: "1.0.0",
        features: {
            wasm: !!wasmApp,
            zk_proof: true,
            algorithms: true,
            edge_server: true,
        },
    };
    
    return new Response(JSON.stringify(healthStatus, null, 2), {
        status: 200,
        headers: {
            'Content-Type': 'application/json',
        },
    });
}

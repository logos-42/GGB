/**
 * Williw Workers 入口点（支持KV、DO和WASM）
 */

// WASM 模块导入
import { WorkersApp } from './williw_wasm.js';

// WASM 应用实例
let wasmApp = null;

// 初始化 WASM
async function initWasm() {
  if (wasmApp) return wasmApp;
  
  try {
    const config = {
      edge_server: {
        name: "williw-edge-server",
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
    
    wasmApp = new WorkersApp(config);
    console.log('WASM 模块初始化成功');
    return wasmApp;
  } catch (error) {
    console.error('WASM 初始化失败:', error);
    throw error;
  }
}

// Durable Object 类
export class TrainingSessionDO {
  constructor(state, env) {
    this.state = state;
    this.env = env;
    this.sessions = new Map();
  }

  async fetch(request) {
    const url = new URL(request.url);
    const path = url.pathname;

    if (path === '/session/create') {
      return this.createSession(request);
    } else if (path.startsWith('/session/')) {
      const sessionId = path.split('/')[2];
      return this.getSession(sessionId);
    }

    return new Response('Not found', { status: 404 });
  }

  async createSession(request) {
    const sessionId = crypto.randomUUID();
    const sessionData = {
      id: sessionId,
      createdAt: new Date().toISOString(),
      status: 'active',
      nodes: []
    };

    this.sessions.set(sessionId, sessionData);
    
    return new Response(JSON.stringify({
      success: true,
      sessionId: sessionId,
      data: sessionData
    }), {
      headers: { 'Content-Type': 'application/json' }
    });
  }

  async getSession(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) {
      return new Response(JSON.stringify({
        success: false,
        error: 'Session not found'
      }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    return new Response(JSON.stringify({
      success: true,
      data: session
    }), {
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const path = url.pathname;

    // KV 存储测试
    if (path === '/kv/test') {
      const key = 'test_key_' + Date.now();
      const value = { message: 'Hello from KV!', timestamp: new Date().toISOString() };
      
      await env.CACHE.put(key, JSON.stringify(value), {
        metadata: { contentType: 'application/json' },
        expirationTtl: 3600 // 1小时过期
      });

      const retrieved = await env.CACHE.get(key, { type: 'json' });
      
      return new Response(JSON.stringify({
        success: true,
        operation: 'kv_test',
        key: key,
        stored: value,
        retrieved: retrieved
      }), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // KV 读取测试
    if (path === '/kv/stats') {
      const keys = await env.CACHE.list();
      const stats = {
        totalKeys: keys.keys.length,
        keys: keys.keys.map(k => k.name)
      };
      
      return new Response(JSON.stringify(stats), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Durable Object 创建会话
    if (path === '/training/create') {
      const id = env.TRAINING_SESSION.idFromName('global-session');
      const stub = env.TRAINING_SESSION.get(id);
      return stub.fetch('https://dummy/session/create', request);
    }

    // Durable Object 获取会话
    if (path.startsWith('/training/session/')) {
      const sessionId = path.split('/')[3];
      const id = env.TRAINING_SESSION.idFromName('global-session');
      const stub = env.TRAINING_SESSION.get(id);
      return stub.fetch(`https://dummy/session/${sessionId}`, request);
    }

    // WASM 集成测试
    if (path === '/wasm/test') {
      try {
        const app = await initWasm();
        const stats = await app.get_stats();
        
        return new Response(JSON.stringify({
          success: true,
          wasm: {
            status: 'initialized',
            message: 'WASM WorkersApp is running',
            stats: stats
          }
        }), {
          headers: { 'Content-Type': 'application/json' }
        });
      } catch (error) {
        return new Response(JSON.stringify({
          success: false,
          error: error.message,
          stack: error.stack
        }), {
          status: 500,
          headers: { 'Content-Type': 'application/json' }
        });
      }
    }

    // 健康检查
    return new Response(JSON.stringify({
      status: "healthy",
      message: "Williw Worker is running with KV and DO support!",
      timestamp: new Date().toISOString(),
      version: "1.1.0",
      features: {
        kv: true,
        durableObjects: true,
        d1: true,
        wasm: true
      }
    }), {
      headers: { 'Content-Type': 'application/json' }
    });
  }
};

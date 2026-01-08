//! Iroh传输实现
//!
//! 基于iroh协议的传输层实现

use anyhow::Result;
use iroh::{Endpoint, NodeAddr, NodeId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{RouteInfo, TransportStats};

/// Iroh传输配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IrohConfig {
    /// 监听地址
    pub listen_addr: String,
    /// 最大连接数
    pub max_connections: usize,
    /// 是否启用 TLS
    pub enable_tls: bool,
    /// 是否启用压缩
    pub enable_compression: bool,
}

/// Iroh传输实现
pub struct IrohTransport {
    endpoint: Endpoint,
    config: IrohConfig,
    stats: Arc<RwLock<TransportStats>>,
    connections: Arc<Mutex<HashMap<NodeId, iroh::endpoint::Connection>>>,
}

impl IrohTransport {
    pub async fn new(config: IrohConfig) -> Result<Self> {
        let endpoint = Endpoint::builder()
            .bind_addr(config.listen_addr.parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()))
            .spawn()
            .map_err(|e| anyhow::anyhow!("创建 iroh endpoint 失败: {:?}", e))?;

        Ok(Self {
            endpoint,
            config,
            stats: Arc::new(RwLock::new(TransportStats::default())),
            connections: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 获取节点ID
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }

    /// 连接到远程节点
    pub async fn connect(&self, node_addr: NodeAddr) -> Result<iroh::endpoint::Connection> {
        let conn = self
            .endpoint
            .connect(node_addr, b"ggb-iroh")
            .await
            .map_err(|e| anyhow::anyhow!("连接失败: {:?}", e))?;

        // 存储连接
        {
            let mut connections = self.connections.lock().await;
            connections.insert(conn.remote_node_id().clone(), conn.clone());
        }

        Ok(conn)
    }
}

#[async_trait::async_trait]
impl super::Transport for IrohTransport {
    async fn send(&self, route: &RouteInfo, message: &[u8]) -> Result<()> {
        // 解析路由地址
        let node_addr: NodeAddr = route.address.parse()
            .map_err(|e| anyhow::anyhow!("无效的节点地址: {}: {}", route.address, e))?;

        // 连接到节点
        let conn = self.connect(node_addr).await?;

        // 发送数据
        let mut send = conn.open_uni().await
            .map_err(|e| anyhow::anyhow!("打开单向流失败: {:?}", e))?;

        send.write_all(message).await
            .map_err(|e| anyhow::anyhow!("写入数据失败: {:?}", e))?;

        send.finish().await
            .map_err(|e| anyhow::anyhow!("完成发送失败: {:?}", e))?;

        // 更新统计信息
        {
            let mut stats = self.stats.write();
            stats.total_sent_bytes += message.len() as u64;
        }

        Ok(())
    }

    async fn receive(&self) -> Result<(String, Vec<u8>)> {
        // 接受连接
        let connecting = self.endpoint.accept().await
            .ok_or_else(|| anyhow::anyhow!("接受连接失败"))?;

        let conn = connecting.await
            .map_err(|e| anyhow::anyhow!("连接建立失败: {:?}", e))?;

        // 存储连接
        {
            let mut connections = self.connections.lock().await;
            connections.insert(conn.remote_node_id().clone(), conn.clone());
        }

        // 接收数据
        let mut recv = conn.accept_uni().await
            .map_err(|e| anyhow::anyhow!("接受单向流失败: {:?}", e))?;

        let data = recv.read_to_end(1024 * 1024).await // 限制最大1MB
            .map_err(|e| anyhow::anyhow!("读取数据失败: {:?}", e))?;

        // 更新统计信息
        {
            let mut stats = self.stats.write();
            stats.total_received_bytes += data.len() as u64;
        }

        let node_id = conn.remote_node_id().to_string();
        Ok((node_id, data))
    }

    fn get_stats(&self) -> TransportStats {
        self.stats.read().clone()
    }
}

impl Default for TransportStats {
    fn default() -> Self {
        Self {
            total_sent_bytes: 0,
            total_received_bytes: 0,
            active_connections: 0,
            failed_sends: 0,
            average_latency_ms: 0.0,
        }
    }
}
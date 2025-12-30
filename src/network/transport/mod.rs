//! 传输层模块
//! 
//! 提供多种传输协议的统一接口。

mod quic;
mod libp2p;

// 重新导出公共接口
pub use quic::*;
pub use libp2p::*;

/// 传输协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransportType {
    /// QUIC 协议
    Quic,
    /// libp2p 协议
    Libp2p,
}

/// 传输配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransportConfig {
    /// 传输协议类型
    pub transport_type: TransportType,
    /// 监听地址
    pub listen_addr: String,
    /// 最大连接数
    pub max_connections: usize,
    /// 是否启用 TLS
    pub enable_tls: bool,
    /// 是否启用压缩
    pub enable_compression: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: TransportType::Quic,
            listen_addr: "0.0.0.0:0".to_string(),
            max_connections: 100,
            enable_tls: true,
            enable_compression: true,
        }
    }
}

/// 传输接口
pub trait Transport: Send + Sync {
    /// 发送消息
    async fn send(&self, route: &RouteInfo, message: &[u8]) -> anyhow::Result<()>;
    
    /// 接收消息
    async fn receive(&self) -> anyhow::Result<(String, Vec<u8>)>;
    
    /// 获取传输统计信息
    fn get_stats(&self) -> TransportStats;
}

/// 路由信息
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub destination: String,
    pub transport_type: TransportType,
    pub address: String,
    pub quality_score: f64,
}

/// 传输统计信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransportStats {
    pub total_sent_bytes: u64,
    pub total_received_bytes: u64,
    pub active_connections: usize,
    pub failed_sends: u64,
    pub average_latency_ms: f64,
}

/// 创建传输实例
pub async fn create_transport(config: &TransportConfig) -> anyhow::Result<Box<dyn Transport>> {
    match config.transport_type {
        TransportType::Quic => {
            let quic_config = QuicConfig {
                listen_addr: config.listen_addr.clone(),
                max_connections: config.max_connections,
                enable_tls: config.enable_tls,
                enable_compression: config.enable_compression,
            };
            Ok(Box::new(QuicTransport::new(quic_config).await?))
        }
        TransportType::Libp2p => {
            let libp2p_config = Libp2pConfig {
                listen_addr: config.listen_addr.clone(),
                max_connections: config.max_connections,
                enable_tls: config.enable_tls,
                enable_compression: config.enable_compression,
            };
            Ok(Box::new(Libp2pTransport::new(libp2p_config).await?))
        }
    }
}

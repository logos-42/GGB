//! 通讯模块
//!
//! 使用 iroh 作为底层通讯协议

pub mod config;
pub mod handle;
pub mod iroh;
pub mod routing;
pub mod p2p_distributor;
pub mod p2p_sender;
pub mod p2p_receiver;
pub mod transfer_protocol;
pub mod p2p_frontend_manager;
// pub mod p2p_web_integration; // 暂时禁用
pub mod p2p_frontend_starter;
pub mod p2p_app_integration;

// 重新导出常用类型
pub use config::{CommsConfig, BandwidthBudgetConfig};
pub use handle::{CommsHandle, IrohEvent, Topic};

use anyhow::Result;

/// 网络配置
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub addr: std::net::SocketAddr,
    pub enable_encryption: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:0".parse().unwrap(),
            enable_encryption: true,
        }
    }
}

/// 网络句柄
pub struct NetworkHandle {
    _placeholder: std::marker::PhantomData<()>,
}

impl NetworkHandle {
    /// 创建新的网络句柄
    pub async fn new(_config: NetworkConfig) -> Result<Self> {
        // 模拟 iroh 网络句柄的创建
        // 实际实现将取决于 iroh 的具体 API 版本
        Ok(Self { _placeholder: std::marker::PhantomData })
    }
    
    /// 获取节点ID
    pub fn node_id(&self) -> String {  // 使用 String 代替 iroh::net::NodeId
        // 返回模拟节点ID
        "SIMULATED_NODE_ID".to_string()
    }
    
    /// 关闭网络句柄
    pub async fn shutdown(self) -> Result<()> {
        // 模拟关闭操作
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_handle_creation() -> Result<()> {
        let config = NetworkConfig::default();
        let handle = NetworkHandle::new(config).await?;
        handle.shutdown().await?;
        Ok(())
    }
}

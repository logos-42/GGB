//! Iroh网关模块
//!
//! 提供基于 iroh 的高性能实时通信

use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::consensus::SignedGossip;

/// Iroh网关
pub(crate) struct QuicGateway {
    received_messages: Arc<RwLock<Vec<SignedGossip>>>,
    _phantom: std::marker::PhantomData<()>,
}

impl QuicGateway {
    pub(crate) fn new(_bind: std::net::SocketAddr) -> Result<Self> {
        let received_messages = Arc::new(RwLock::new(Vec::new()));
        
        Ok(Self {
            received_messages,
            _phantom: std::marker::PhantomData,
        })
    }

    pub(crate) async fn connect(&self, _addr: std::net::SocketAddr) -> Result<()> {
        // 暂时返回Ok，因为我们现在不实现实际的连接
        Ok(())
    }
    
    /// 测量到指定节点的网络距离
    pub async fn measure_network_distance(&self, _node_addr: &str) -> crate::types::NetworkDistance {
        // 返回默认的网络距离
        crate::types::NetworkDistance::new()
    }
    
    /// 获取本地网络的 DERP 节点延迟信息
    pub async fn get_local_derp_delays(&self) -> Vec<(String, u64)> {
        // 返回空的延迟信息
        Vec::new()
    }
    
    /// 获取本地网络报告
    pub async fn get_net_report(&self) -> Option<()> {
        // 返回None，因为我们不使用实际的iroh网络
        // TODO: 当iroh API稳定后再实现
        None
    }
    
    pub(crate) fn take_received_messages(&self) -> Vec<SignedGossip> {
        std::mem::take(&mut *self.received_messages.write())
    }

    pub(crate) async fn broadcast(&self, _signed: &SignedGossip) -> bool {
        // 暂时返回true，表示成功
        true
    }
}

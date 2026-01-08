//! Iroh网关模块
//!
//! 提供基于 iroh 的高性能实时通信

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use iroh::{Endpoint, NodeAddr, endpoint::Connection};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::interval;

use crate::consensus::SignedGossip;

/// Iroh网关
pub(crate) struct QuicGateway {
    endpoint: Endpoint,
    connections: Arc<RwLock<Vec<ConnectionInfo>>>,
    received_messages: Arc<RwLock<Vec<SignedGossip>>>,
}

/// 连接信息
struct ConnectionInfo {
    connection: Connection,
    last_health_check: Instant,
    consecutive_failures: u32,
}

impl ConnectionInfo {
    fn new(connection: Connection) -> Self {
        Self {
            connection,
            last_health_check: Instant::now(),
            consecutive_failures: 0,
        }
    }

    fn is_healthy(&self) -> bool {
        self.consecutive_failures < 3
    }

    fn mark_failure(&mut self) {
        self.consecutive_failures += 1;
    }

    /// 标记连接成功（用于健康检查）
    fn mark_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_health_check = Instant::now();
    }
}

impl QuicGateway {
    pub(crate) fn new(bind: std::net::SocketAddr) -> Result<Self> {
        // 使用 iroh 创建 endpoint
        let endpoint = Endpoint::builder()
            .bind_addr(bind)
            .spawn()
            .map_err(|e| anyhow!("创建 iroh endpoint 失败: {:?}", e))?;

        let connections = Arc::new(RwLock::new(Vec::new()));
        let received_messages = Arc::new(RwLock::new(Vec::new()));
        
        // 启动连接接受任务
        let accept_endpoint = endpoint.clone();
        let accept_pool = connections.clone();
        let accept_messages = received_messages.clone();
        tokio::spawn(async move {
            loop {
                match accept_endpoint.accept().await {
                    Ok(connecting) => match connecting.await {
                        Ok(conn) => {
                            println!("[Iroh] 接受来自 {:?} 的连接", conn.remote_addr());
                            accept_pool.write().push(ConnectionInfo::new(conn.clone()));
                            let msg_queue = accept_messages.clone();
                            let conn_id = conn.remote_addr();
                            tokio::spawn(async move {
                                loop {
                                    // 从连接接收数据
                                    match conn.accept_uni().await {
                                        Ok(mut recv) => {
                                            match recv.read_to_end(1024 * 1024).await {
                                                Ok(buf) => {
                                                    if let Ok(signed) = serde_json::from_slice::<SignedGossip>(&buf) {
                                                        println!("[Iroh] 收到消息 from {:?}", conn_id);
                                                        msg_queue.write().push(signed);
                                                    }
                                                }
                                                Err(_) => continue,
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            });
                        }
                        Err(err) => eprintln!("[Iroh] accept error: {err:?}"),
                    },
                    Err(err) => {
                        eprintln!("[Iroh] accept 等待错误: {err:?}");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
        
        // 启动健康检查任务
        let health_check_connections = connections.clone();
        tokio::spawn(async move {
            let mut health_check_interval = interval(std::time::Duration::from_secs(30));
            loop {
                health_check_interval.tick().await;
                let mut conns = health_check_connections.write();
                conns.retain_mut(|info| {
                    // 如果连接仍然活跃且健康，标记为成功
                    if info.is_healthy() {
                        info.mark_success();
                    } else if info.last_health_check.elapsed() > std::time::Duration::from_secs(300) {
                        info.mark_failure();
                    }
                    info.is_healthy()
                });
            }
        });
        
        Ok(Self {
            endpoint,
            connections,
            received_messages,
        })
    }

    pub(crate) async fn connect(&self, addr: std::net::SocketAddr) -> Result<()> {
        println!("[Iroh] 尝试连接到 {}", addr);

        // 创建 NodeAddr
        let node_addr = NodeAddr::from(addr);

        // 使用 ALPN 协议
        let alpn = b"ggb-iroh";

        match self.endpoint.connect(node_addr, alpn).await {
            Ok(connection) => {
                println!("[Iroh] 成功连接到 {}", addr);
                self.connections.write().push(ConnectionInfo::new(connection.clone()));

                let msg_queue = self.received_messages.clone();
                let conn_id = connection.remote_addr();
                tokio::spawn(async move {
                    loop {
                        match connection.accept_uni().await {
                            Ok(mut recv) => {
                                match recv.read_to_end(1024 * 1024).await {
                                    Ok(buf) => {
                                        if let Ok(signed) = serde_json::from_slice::<SignedGossip>(&buf) {
                                            println!("[Iroh] 收到消息 from {:?}", conn_id);
                                            msg_queue.write().push(signed);
                                        }
                                    }
                                    Err(_) => continue,
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
                Ok(())
            }
            Err(err) => {
                println!("[Iroh] 连接 {} 失败: {:?}", addr, err);
                Err(anyhow!("连接失败: {:?}", err))
            }
        }
    }
    
    pub(crate) fn take_received_messages(&self) -> Vec<SignedGossip> {
        std::mem::take(&mut *self.received_messages.write())
    }

    pub(crate) async fn broadcast(&self, signed: &SignedGossip) -> bool {
        let bytes = match serde_json::to_vec(signed) {
            Ok(b) => b,
            Err(_) => return false,
        };
        
        let entries: Vec<(Connection, usize)> = {
            let guard = self.connections.read();
            guard
                .iter()
                .enumerate()
                .filter(|(_, info)| info.is_healthy())
                .map(|(idx, info)| (info.connection.clone(), idx))
                .collect()
        };
        
        let mut success = false;
        let mut failed_indices = Vec::new();
        let mut success_indices = Vec::new();
        
        for (conn, idx) in entries {
            match conn.open_uni().await {
                Ok(mut send) => {
                    if send.write_all(&bytes).await.is_ok() && send.finish().await.is_ok() {
                        success = true;
                        success_indices.push(idx);
                    } else {
                        failed_indices.push(idx);
                    }
                }
                Err(_) => {
                    failed_indices.push(idx);
                }
            }
        }
        
        let mut guard = self.connections.write();
        let current_len = guard.len();
        
        // 标记成功的连接
        for idx in success_indices {
            if idx < current_len {
                if let Some(info) = guard.get_mut(idx) {
                    info.mark_success();
                }
            }
        }
        
        // 标记失败的连接
        if !failed_indices.is_empty() {
            for idx in failed_indices {
                if idx < current_len {
                    if let Some(info) = guard.get_mut(idx) {
                        info.mark_failure();
                    }
                }
            }
            guard.retain(|info| info.is_healthy());
        }
        
        success
    }
}

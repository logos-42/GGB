//! QUIC网关模块
//! 
//! 提供基于QUIC协议的高性能实时通信

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use quinn::{Endpoint, ServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;

use crate::consensus::SignedGossip;

/// QUIC网关
pub(crate) struct QuicGateway {
    endpoint: Endpoint,
    connections: Arc<RwLock<Vec<ConnectionInfo>>>,
    received_messages: Arc<RwLock<Vec<SignedGossip>>>,
}

/// 连接信息
struct ConnectionInfo {
    connection: quinn::Connection,
    last_health_check: Instant,
    consecutive_failures: u32,
}

impl ConnectionInfo {
    fn new(connection: quinn::Connection) -> Self {
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

/// 跳过服务器验证（用于自签名证书）
struct SkipServerVerification;

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> std::result::Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

impl QuicGateway {
    pub(crate) fn new(bind: SocketAddr) -> Result<Self> {
        let cert = generate_simple_self_signed(vec!["ggb-quic".into()])?;
        let cert_der = cert.serialize_der()?;
        let key_der = cert.serialize_private_key_der();
        
        let mut server_config = ServerConfig::with_single_cert(
            vec![Certificate(cert_der.clone())],
            PrivateKey(key_der.clone()),
        )?;
        server_config.transport = Arc::new(quinn::TransportConfig::default());
        
        let client_crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();
        let client_config = quinn::ClientConfig::new(Arc::new(client_crypto));
        
        let mut endpoint = Endpoint::server(server_config, bind)?;
        endpoint.set_default_client_config(client_config);
        let connections = Arc::new(RwLock::new(Vec::new()));
        let received_messages = Arc::new(RwLock::new(Vec::new()));
        
        // 启动连接接受任务
        let accept_endpoint = endpoint.clone();
        let accept_pool = connections.clone();
        let accept_messages = received_messages.clone();
        tokio::spawn(async move {
            loop {
                match accept_endpoint.accept().await {
                    Some(connecting) => match connecting.await {
                        Ok(conn) => {
                            println!("[QUIC] 接受来自 {} 的连接", conn.remote_address());
                            accept_pool.write().push(ConnectionInfo::new(conn.clone()));
                            let msg_queue = accept_messages.clone();
                            let remote = conn.remote_address();
                            tokio::spawn(async move {
                                loop {
                                    match conn.accept_uni().await {
                                        Ok(mut recv) => {
                                            match recv.read_to_end(1024 * 1024).await {
                                                Ok(buf) => {
                                                    if let Ok(signed) = serde_json::from_slice::<SignedGossip>(&buf) {
                                                        println!("[QUIC] 收到消息 from {}", remote);
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
                        Err(err) => eprintln!("[QUIC] accept error: {err:?}"),
                    },
                    None => tokio::time::sleep(Duration::from_secs(1)).await,
                }
            }
        });
        
        // 启动健康检查任务
        let health_check_connections = connections.clone();
        tokio::spawn(async move {
            let mut health_check_interval = interval(Duration::from_secs(30));
            loop {
                health_check_interval.tick().await;
                let mut conns = health_check_connections.write();
                conns.retain_mut(|info| {
                    if info.connection.close_reason().is_some() {
                        return false;
                    }
                    // 如果连接仍然活跃且健康，标记为成功
                    if info.connection.close_reason().is_none() && info.is_healthy() {
                        info.mark_success();
                    } else if info.last_health_check.elapsed() > Duration::from_secs(300) {
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

    pub(crate) async fn connect(&self, addr: SocketAddr) -> Result<()> {
        println!("[QUIC] 尝试连接到 {}", addr);
        match self.endpoint.connect(addr, "ggb-quic") {
            Ok(connecting) => match connecting.await {
                Ok(connection) => {
                    println!("[QUIC] 成功连接到 {}", addr);
                    self.connections.write().push(ConnectionInfo::new(connection.clone()));
                    
                    let msg_queue = self.received_messages.clone();
                    tokio::spawn(async move {
                        loop {
                            match connection.accept_uni().await {
                                Ok(mut recv) => {
                                    match recv.read_to_end(1024 * 1024).await {
                                        Ok(buf) => {
                                            if let Ok(signed) = serde_json::from_slice::<SignedGossip>(&buf) {
                                                println!("[QUIC] 收到消息 from {}", connection.remote_address());
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
                    println!("[QUIC] 连接 {} 失败: {:?}", addr, err);
                    Err(err.into())
                }
            },
            Err(err) => {
                println!("[QUIC] 无法启动连接到 {}: {:?}", addr, err);
                Err(err.into())
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
        
        let entries: Vec<(quinn::Connection, usize)> = {
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

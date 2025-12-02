use crate::consensus::SignedGossip;
use crate::device::NetworkType;
use anyhow::{anyhow, Result};
use libp2p::{
    gossipsub::{
        self, Behaviour as GossipsubBehaviour, Event as GossipsubEvent, IdentTopic as Topic,
        MessageAuthenticity, ValidationMode,
    },
    identity,
    mdns::{self, tokio::Behaviour as Mdns, Event as MdnsEvent},
    swarm::{NetworkBehaviour, SwarmBuilder},
    Multiaddr, PeerId, Swarm,
};
use parking_lot::RwLock;
use quinn::{Endpoint, ServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;

pub struct CommsConfig {
    pub topic: String,
    pub listen_addr: Option<Multiaddr>,
    pub quic_bind: Option<SocketAddr>,
    pub quic_bootstrap: Vec<SocketAddr>,
    pub bandwidth: BandwidthBudgetConfig,
}

impl Default for CommsConfig {
    fn default() -> Self {
        Self {
            topic: "ggs-training".into(),
            listen_addr: None,
            quic_bind: Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 9234)),
            quic_bootstrap: Vec::new(),
            bandwidth: BandwidthBudgetConfig::default(),
        }
    }
}

#[derive(Clone)]
pub struct BandwidthBudgetConfig {
    pub sparse_per_window: u32,
    pub dense_bytes_per_window: usize,
    pub window_secs: u64,
}

impl Default for BandwidthBudgetConfig {
    fn default() -> Self {
        Self {
            sparse_per_window: 12,
            dense_bytes_per_window: 256 * 1024,
            window_secs: 60,
        }
    }
}

struct BandwidthBudget {
    config: BandwidthBudgetConfig,
    window_start: Instant,
    sparse_sent: u32,
    dense_sent: usize,
}

impl BandwidthBudget {
    fn new(config: BandwidthBudgetConfig) -> Self {
        Self {
            config,
            window_start: Instant::now(),
            sparse_sent: 0,
            dense_sent: 0,
        }
    }

    fn rotate(&mut self) {
        if self.window_start.elapsed() >= Duration::from_secs(self.config.window_secs) {
            self.window_start = Instant::now();
            self.sparse_sent = 0;
            self.dense_sent = 0;
        }
    }

    fn allow_sparse(&mut self) -> bool {
        self.rotate();
        if self.sparse_sent < self.config.sparse_per_window {
            self.sparse_sent += 1;
            true
        } else {
            false
        }
    }

    fn allow_dense(&mut self, bytes: usize) -> bool {
        self.rotate();
        if self.dense_sent + bytes <= self.config.dense_bytes_per_window {
            self.dense_sent += bytes;
            true
        } else {
            false
        }
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
pub struct Behaviour {
    gossipsub: GossipsubBehaviour,
    mdns: Mdns,
}

#[derive(Debug)]
pub enum OutEvent {
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
}

impl From<GossipsubEvent> for OutEvent {
    fn from(v: GossipsubEvent) -> Self {
        OutEvent::Gossipsub(v)
    }
}

impl From<MdnsEvent> for OutEvent {
    fn from(v: MdnsEvent) -> Self {
        OutEvent::Mdns(v)
    }
}

pub struct CommsHandle {
    pub peer_id: PeerId,
    pub swarm: Swarm<Behaviour>,
    pub topic: Topic,
    quic: Option<Arc<QuicGateway>>,
    bandwidth: RwLock<BandwidthBudget>,
    network_type: parking_lot::RwLock<crate::device::NetworkType>,
}

impl CommsHandle {
    pub async fn new(config: CommsConfig) -> Result<Self> {
        let local_key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());

        let transport = libp2p::tokio_development_transport(local_key.clone())?;
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .validation_mode(ValidationMode::Permissive)
            .build()
            .expect("valid config");
        let mut gossipsub = GossipsubBehaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow!(e))?;
        let topic = Topic::new(config.topic.clone());
        gossipsub.subscribe(&topic)?;
        let mdns = Mdns::new(mdns::Config::default(), peer_id)?;
        let behaviour = Behaviour { gossipsub, mdns };
        let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();
        if let Some(addr) = config.listen_addr {
            swarm.listen_on(addr)?;
        }

        let quic = if let Some(bind) = config.quic_bind {
            let gateway = Arc::new(QuicGateway::new(bind)?);
            for addr in &config.quic_bootstrap {
                let _ = gateway.connect(*addr).await;
            }
            Some(gateway)
        } else {
            None
        };

        Ok(Self {
            peer_id: swarm.local_peer_id().clone(),
            swarm,
            topic,
            quic,
            bandwidth: RwLock::new(BandwidthBudget::new(config.bandwidth)),
            network_type: parking_lot::RwLock::new(NetworkType::Unknown),
        })
    }

    pub fn publish(&mut self, signed: &SignedGossip) -> Result<()> {
        let data = serde_json::to_vec(signed)?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), data)?;
        Ok(())
    }

    pub fn allow_sparse_update(&self) -> bool {
        self.bandwidth.write().allow_sparse()
    }

    pub fn allow_dense_snapshot(&self, bytes: usize) -> bool {
        // 检查网络类型是否允许密集快照
        let network_type = *self.network_type.read();
        if !network_type.allows_dense_snapshot() {
            return false;
        }
        self.bandwidth.write().allow_dense(bytes)
    }

    /// 更新网络类型
    pub fn update_network_type(&self, network_type: NetworkType) {
        *self.network_type.write() = network_type;
        println!("[网络] 网络类型更新: {:?}", network_type);
    }

    /// 获取当前网络类型
    pub fn network_type(&self) -> NetworkType {
        *self.network_type.read()
    }

    pub async fn broadcast_realtime(&self, signed: &SignedGossip) -> bool {
        if let Some(quic) = &self.quic {
            return quic.broadcast(signed).await;
        }
        false
    }
}

struct QuicGateway {
    endpoint: Endpoint,
    connections: Arc<RwLock<Vec<ConnectionInfo>>>,
}

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

    fn mark_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_health_check = Instant::now();
    }
}

impl QuicGateway {
    fn new(bind: SocketAddr) -> Result<Self> {
        let cert = generate_simple_self_signed(vec!["ggs-quic".into()])?;
        let cert_der = cert.serialize_der()?;
        let key_der = cert.serialize_private_key_der();
        let mut server_config = ServerConfig::with_single_cert(
            vec![Certificate(cert_der.clone())],
            PrivateKey(key_der.clone()),
        )?;
        server_config.transport = Arc::new(quinn::TransportConfig::default());
        let endpoint = Endpoint::server(server_config, bind)?;
        let connections = Arc::new(RwLock::new(Vec::new()));
        let accept_endpoint = endpoint.clone();
        let accept_pool = connections.clone();
        tokio::spawn(async move {
            loop {
                match accept_endpoint.accept().await {
                    Some(connecting) => match connecting.await {
                        Ok(conn) => {
                            accept_pool.write().push(ConnectionInfo::new(conn));
                        }
                        Err(err) => eprintln!("[QUIC] accept error: {err:?}"),
                    },
                    None => tokio::time::sleep(Duration::from_secs(1)).await,
                }
            }
        });
        
        // 启动连接健康检查任务
        let health_check_connections = connections.clone();
        tokio::spawn(async move {
            let mut health_check_interval = interval(Duration::from_secs(30));
            loop {
                health_check_interval.tick().await;
                let mut conns = health_check_connections.write();
                conns.retain_mut(|info| {
                    // 检查连接是否仍然有效
                    if info.connection.close_reason().is_some() {
                        return false;
                    }
                    // 如果超过 5 分钟没有成功通信，标记为失败
                    if info.last_health_check.elapsed() > Duration::from_secs(300) {
                        info.mark_failure();
                    }
                    info.is_healthy()
                });
            }
        });
        Ok(Self {
            endpoint,
            connections,
        })
    }

    async fn connect(&self, addr: SocketAddr) -> Result<()> {
        match self.endpoint.connect(addr, "ggs-quic") {
            Ok(connecting) => match connecting.await {
                Ok(connection) => {
                    self.connections
                        .write()
                        .push(ConnectionInfo::new(connection));
                    Ok(())
                }
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    /// 尝试重连所有失效的连接
    pub async fn reconnect_failed(&self, addrs: &[SocketAddr]) {
        let mut conns = self.connections.write();
        let failed_count = conns
            .iter()
            .filter(|info| !info.is_healthy())
            .count();
        
        if failed_count > 0 {
            println!("[QUIC] 检测到 {} 个失效连接，尝试重连", failed_count);
            // 移除失效连接
            conns.retain(|info| info.is_healthy());
            
            // 尝试重新连接
            for addr in addrs {
                if let Err(e) = self.connect(*addr).await {
                    eprintln!("[QUIC] 重连失败 {}: {:?}", addr, e);
                }
            }
        }
    }

    async fn broadcast(&self, signed: &SignedGossip) -> bool {
        let bytes = match serde_json::to_vec(signed) {
            Ok(b) => b,
            Err(_) => return false,
        };
        
        // 收集连接和对应的原始索引，使用连接对象本身而不是索引
        // 这样可以避免在向量修改后索引失效的问题
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
        let mut failed_original_indices = Vec::new();
        let mut success_original_indices = Vec::new();
        
        // 尝试发送到所有连接
        for (conn, original_idx) in entries {
            match conn.open_uni().await {
                Ok(mut send) => {
                    if send.write_all(&bytes).await.is_ok() && send.finish().await.is_ok() {
                        success = true;
                        success_original_indices.push(original_idx);
                    } else {
                        failed_original_indices.push(original_idx);
                    }
                }
                Err(_) => {
                    failed_original_indices.push(original_idx);
                }
            }
        }
        
        // 原子性地更新连接状态
        // 使用原始索引，但需要验证索引仍然有效（连接未被移除）
        if !success_original_indices.is_empty() || !failed_original_indices.is_empty() {
            let mut guard = self.connections.write();
            let current_len = guard.len();
            
            // 标记成功的连接
            for &idx in &success_original_indices {
                if idx < current_len {
                    if let Some(info) = guard.get_mut(idx) {
                        // 验证这确实是我们要标记的连接（通过检查连接是否仍然健康）
                        if info.is_healthy() {
                            info.mark_success();
                        }
                    }
                }
            }
            
            // 标记失败的连接
            for &idx in &failed_original_indices {
                if idx < current_len {
                    if let Some(info) = guard.get_mut(idx) {
                        info.mark_failure();
                    }
                }
            }
            
            // 移除不健康的连接（这可能会改变后续索引，但我们已经处理完了）
            guard.retain(|info| info.is_healthy());
        }
        
        success
    }
}

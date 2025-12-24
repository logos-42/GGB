use crate::consensus::SignedGossip;
use crate::device::NetworkType;
use anyhow::{anyhow, Result};
use libp2p::{
    gossipsub::{
        self, Behaviour as GossipsubBehaviour, Event as GossipsubEvent, IdentTopic as Topic,
        MessageAuthenticity, PublishError, ValidationMode,
    },
    identity,
    kad::{store::MemoryStore, Kademlia, KademliaEvent},
    mdns::{self, tokio::Behaviour as Mdns, Event as MdnsEvent},
    swarm::{NetworkBehaviour, SwarmBuilder},
    Multiaddr, PeerId, Swarm,
};
use libp2p::multiaddr::Protocol;
use std::path::PathBuf;
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
    pub enable_dht: bool,
    pub bootstrap_peers_file: Option<PathBuf>,
}

impl Default for CommsConfig {
    fn default() -> Self {
        Self {
            topic: "ggb-training".into(),
            listen_addr: None,
            quic_bind: Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 9234)),
            quic_bootstrap: Vec::new(),
            bandwidth: BandwidthBudgetConfig::default(),
            enable_dht: true,
            bootstrap_peers_file: None,
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
    kademlia: Kademlia<MemoryStore>,
}

#[derive(Debug)]
pub enum OutEvent {
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
    Kademlia(KademliaEvent),
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

impl From<KademliaEvent> for OutEvent {
    fn from(v: KademliaEvent) -> Self {
        OutEvent::Kademlia(v)
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
        
        // 初始化 Kademlia DHT
        let store = MemoryStore::new(peer_id);
        let kademlia = Kademlia::new(peer_id, store);
        
        // 从文件加载 bootstrap 节点（如果存在）
        let mut bootstrap_peers = Vec::new();
        if let Some(ref file_path) = config.bootstrap_peers_file {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                for line in content.lines() {
                    if let Ok(addr) = line.trim().parse::<Multiaddr>() {
                        bootstrap_peers.push(addr);
                    }
                }
            }
        }
        
        // 添加 bootstrap 节点到 Kademlia
        let mut kademlia = kademlia;
        for addr in &bootstrap_peers {
            // 从 Multiaddr 中提取 PeerId
            // Multiaddr 格式通常是: /ip4/127.0.0.1/tcp/8080/p2p/<peer_id>
            let mut addr_clone = addr.clone();
            if let Some(protocol) = addr_clone.pop() {
                if let Protocol::P2p(multihash) = protocol {
                    if let Ok(peer_id) = PeerId::from_multihash(multihash) {
                        kademlia.add_address(&peer_id, addr.clone());
                    }
                }
            }
        }
        
        // 如果启用 DHT，开始 bootstrap
        if config.enable_dht && !bootstrap_peers.is_empty() {
            kademlia.bootstrap()?;
        }
        
        let behaviour = Behaviour {
            gossipsub,
            mdns,
            kademlia,
        };
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
        match self.swarm.behaviour_mut().gossipsub.publish(self.topic.clone(), data) {
            Ok(_) => Ok(()),
            Err(PublishError::InsufficientPeers) => {
                // 没有足够的 peer 是正常情况（节点刚启动时），静默忽略
                Ok(())
            }
            Err(e) => Err(anyhow!("Gossipsub 发布失败: {:?}", e)),
        }
    }

    pub fn allow_sparse_update(&self) -> bool {
        self.bandwidth.write().allow_sparse()
    }

    pub fn allow_dense_snapshot(&self, bytes: usize) -> bool {
        let network_type = *self.network_type.read();
        if !network_type.allows_dense_snapshot() {
            return false;
        }
        self.bandwidth.write().allow_dense(bytes)
    }

    pub fn update_network_type(&self, network_type: NetworkType) {
        *self.network_type.write() = network_type;
        println!("[网络] 网络类型更新: {:?}", network_type);
    }

    pub fn network_type(&self) -> NetworkType {
        *self.network_type.read()
    }

    pub fn add_peer(&mut self, peer: &PeerId) {
        self.swarm.behaviour_mut().gossipsub.add_explicit_peer(peer);
    }

    pub fn remove_peer(&mut self, peer: &PeerId) {
        self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(peer);
    }

    pub async fn broadcast_realtime(&self, signed: &SignedGossip) -> bool {
        if let Some(quic) = &self.quic {
            return quic.broadcast(signed).await;
        }
        false
    }
    
    pub fn take_quic_messages(&self) -> Vec<SignedGossip> {
        if let Some(quic) = &self.quic {
            return quic.take_received_messages();
        }
        Vec::new()
    }
}

struct QuicGateway {
    endpoint: Endpoint,
    connections: Arc<RwLock<Vec<ConnectionInfo>>>,
    received_messages: Arc<RwLock<Vec<SignedGossip>>>,
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

    /// 标记连接成功（用于健康检查）
    fn mark_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_health_check = Instant::now();
    }
}

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
    fn new(bind: SocketAddr) -> Result<Self> {
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

    async fn connect(&self, addr: SocketAddr) -> Result<()> {
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
    
    fn take_received_messages(&self) -> Vec<SignedGossip> {
        std::mem::take(&mut *self.received_messages.write())
    }

    async fn broadcast(&self, signed: &SignedGossip) -> bool {
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


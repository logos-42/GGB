//! 通信句柄模块
//! 
//! 提供通信的主要接口和功能

use anyhow::{anyhow, Result};
use libp2p::{
    gossipsub::{MessageAuthenticity, PublishError, ValidationMode},
    identity,
    multiaddr::Protocol,
    swarm::{Swarm, SwarmBuilder},
    Multiaddr, PeerId,
};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

use crate::consensus::SignedGossip;
use crate::device::NetworkType;

use super::config::{CommsConfig, BandwidthBudget};
use super::libp2p::{Behaviour, Topic};
use super::quic::QuicGateway;

/// 通信句柄
pub struct CommsHandle {
    pub peer_id: PeerId,
    pub swarm: Swarm<Behaviour>,
    pub topic: Topic,
    quic: Option<Arc<QuicGateway>>,
    bandwidth: RwLock<BandwidthBudget>,
    network_type: parking_lot::RwLock<NetworkType>,
}

impl CommsHandle {
    pub async fn new(config: CommsConfig) -> Result<Self> {
        let local_key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());

        let transport = libp2p::tokio_development_transport(local_key.clone())?;
        let gossipsub_config = libp2p::gossipsub::ConfigBuilder::default()
            .validation_mode(ValidationMode::Permissive)
            .build()
            .expect("valid config");
        let mut gossipsub = libp2p::gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow!(e))?;
        let topic = Topic::new(config.topic.clone());
        gossipsub.subscribe(&topic)?;
        
        let mdns = libp2p::mdns::tokio::Behaviour::new(
            libp2p::mdns::Config::default(), 
            peer_id
        )?;
        
        // 初始化中继客户端（根据配置决定是否有效使用）
        let relay = libp2p::relay::Behaviour::new(peer_id, libp2p::relay::Config::default());
        
        // 初始化自动NAT
        let autonat = libp2p::autonat::Behaviour::new(peer_id, libp2p::autonat::Config::default());
        
        // 初始化DCUtR（直接连接升级）
        let dcutr = libp2p::dcutr::Behaviour::new(peer_id);
        
        // 初始化 Kademlia DHT
        let store = libp2p::kad::store::MemoryStore::new(peer_id);
        let kademlia = libp2p::kad::Kademlia::new(peer_id, store);
        
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
        
        // 根据安全配置决定是否启用DHT
        let enable_dht = config.enable_dht && !config.security.hide_ip;
        if enable_dht && !bootstrap_peers.is_empty() {
            println!("[安全] 启用公共DHT（IP可能暴露）");
            kademlia.bootstrap()?;
        } else if config.security.hide_ip {
            println!("[安全] 禁用公共DHT保护IP隐私");
        }
        
        let behaviour = Behaviour {
            gossipsub,
            relay,
            autonat,
            dcutr,
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

    /// 连接到中继节点
    pub async fn connect_to_relay(&mut self, relay_addr: Multiaddr) -> Result<()> {
        println!("[中继] 尝试连接到中继节点: {}", relay_addr);
        
        // 从地址中提取PeerId
        let mut addr_clone = relay_addr.clone();
        if let Some(protocol) = addr_clone.pop() {
            if let Protocol::P2p(multihash) = protocol {
                if let Ok(peer_id) = PeerId::from_multihash(multihash) {
                    // 添加到swarm
                    self.swarm
                        .behaviour_mut()
                        .relay
                        .add_relay(&peer_id, relay_addr.clone())?;
                    
                    println!("[中继] 已添加中继节点: {}", peer_id);
                    return Ok(());
                }
            }
        }
        
        Err(anyhow!("无法从地址中提取有效的PeerId: {}", relay_addr))
    }

    /// 获取中继地址（用于其他节点连接）
    pub fn get_relay_address(&self, target_peer_id: &PeerId) -> Option<Multiaddr> {
        // 构建中继地址格式: /ip4/中继IP/tcp/端口/p2p/中继PeerId/p2p-circuit/p2p/目标PeerId
        // 注意：这里需要实际的中继节点地址，目前返回None
        // 在实际应用中，需要维护中继节点列表并选择合适的地址
        None
    }
}

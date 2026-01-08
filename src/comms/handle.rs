//! 通信句柄模块
//!
//! 提供基于 iroh 的通信接口和功能

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use iroh::{Endpoint, NodeId, NodeAddr};
use tokio::sync::mpsc;

use crate::consensus::SignedGossip;
use crate::device::NetworkType;

use super::config::{CommsConfig, BandwidthBudget};
use super::iroh::QuicGateway;

/// Topic 类型（用于发布/订阅）
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Topic {
    name: String,
}

impl Topic {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Iroh 事件类型
#[derive(Debug, Clone)]
pub enum IrohEvent {
    /// 接收到 Gossip 消息
    Gossip {
        source: NodeId,
        data: Vec<u8>,
    },
    /// 发现新节点
    PeerDiscovered {
        peer: NodeId,
        addr: String,
    },
    /// 节点离线
    PeerExpired {
        peer: NodeId,
    },
    /// 连接建立
    ConnectionEstablished {
        peer: NodeId,
    },
    /// 连接断开
    ConnectionClosed {
        peer: NodeId,
    },
}

/// Gossip 消息信息
struct GossipMessage {
    topic: Topic,
    data: Vec<u8>,
    source: NodeId,
}

/// 节点订阅信息
struct PeerSubscription {
    peer: NodeId,
    topics: Vec<Topic>,
}

/// 通信句柄
pub struct CommsHandle {
    pub peer_id: NodeId,
    pub topic: Topic,
    endpoint: Endpoint,
    gossip_tx: mpsc::Sender<GossipMessage>,
    _gossip_rx: mpsc::Receiver<GossipMessage>,
    event_tx: mpsc::Sender<IrohEvent>,
    pub event_rx: mpsc::Receiver<IrohEvent>,
    quic: Option<Arc<QuicGateway>>,
    bandwidth: RwLock<BandwidthBudget>,
    network_type: parking_lot::RwLock<NetworkType>,
    subscriptions: RwLock<Vec<PeerSubscription>>,
}

impl CommsHandle {
    pub async fn new(config: CommsConfig) -> Result<Self> {
        // 创建 iroh endpoint
        let endpoint = Endpoint::builder()
            .bind_addr(config.listen_addr.unwrap_or_else(|| {
                "0.0.0.0:0".parse().unwrap()
            }))
            .spawn()
            .map_err(|e| anyhow!("创建 iroh endpoint 失败: {:?}", e))?;

        let peer_id = endpoint.node_id();
        println!("[Iroh] 节点 ID: {}", peer_id);

        // 创建 gossip 消息通道
        let (gossip_tx, gossip_rx) = mpsc::channel(1024);
        // 创建事件通道
        let (event_tx, event_rx) = mpsc::channel(1024);

        // 初始化 QUIC 网关（用于实时通信）
        let quic = if let Some(bind) = config.iroh_bind {
            let gateway = Arc::new(QuicGateway::new(bind)?);
            for addr in &config.iroh_bootstrap {
                let _ = gateway.connect(*addr).await;
            }
            Some(gateway)
        } else {
            None
        };

        // 启动 gossip 接收任务
        let accept_endpoint = endpoint.clone();
        let accept_gossip_tx = gossip_tx.clone();
        let accept_event_tx = event_tx.clone();
        tokio::spawn(async move {
            loop {
                match accept_endpoint.accept().await {
                    Ok(connecting) => {
                        match connecting.await {
                            Ok(conn) => {
                                println!("[Iroh] 接受来自 {:?} 的连接", conn.remote_addr());
                                let peer_id = conn.remote_node_id().clone();

                                // 发送连接建立事件
                                let _ = accept_event_tx.send(IrohEvent::ConnectionEstablished {
                                    peer: peer_id.clone(),
                                }).await;

                                // 接收消息
                                let conn_clone = conn.clone();
                                let gossip_tx_clone = accept_gossip_tx.clone();
                                let event_tx_clone = accept_event_tx.clone();
                                tokio::spawn(async move {
                                    loop {
                                        match conn_clone.accept_uni().await {
                                            Ok(mut recv) => {
                                                match recv.read_to_end(1024 * 1024).await {
                                                    Ok(buf) => {
                                                        // 解析消息格式: [topic_len:4][topic_data][message_data]
                                                        if buf.len() < 4 {
                                                            continue;
                                                        }
                                                        let topic_len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
                                                        if buf.len() < 4 + topic_len {
                                                            continue;
                                                        }
                                                        let topic_bytes = &buf[4..4 + topic_len];
                                                        let message_data = &buf[4 + topic_len..];

                                                        let topic = Topic::new(String::from_utf8_lossy(topic_bytes).to_string());
                                                        let _ = gossip_tx_clone.send(GossipMessage {
                                                            topic: topic.clone(),
                                                            data: message_data.to_vec(),
                                                            source: peer_id.clone(),
                                                        }).await;

                                                        let _ = event_tx_clone.send(IrohEvent::Gossip {
                                                            source: peer_id.clone(),
                                                            data: message_data.to_vec(),
                                                        }).await;
                                                    }
                                                    Err(_) => continue,
                                                }
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                    let _ = event_tx_clone.send(IrohEvent::ConnectionClosed {
                                        peer: peer_id.clone(),
                                    }).await;
                                });
                            }
                            Err(err) => eprintln!("[Iroh] accept error: {err:?}"),
                        }
                    }
                    Err(err) => {
                        eprintln!("[Iroh] accept 等待错误: {err:?}");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        // 从文件加载 bootstrap 节点（如果存在）
        if let Some(ref file_path) = config.bootstrap_peers_file {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                for line in content.lines() {
                    if let Ok(addr) = line.trim().parse::<NodeAddr>() {
                        println!("[Iroh] 添加 bootstrap 节点: {}", addr.node_id);
                        // 可以尝试连接这些节点
                        let _ = endpoint.connect(addr, b"ggb-iroh").await;
                    }
                }
            }
        }

        Ok(Self {
            peer_id,
            topic: Topic::new(config.topic.clone()),
            endpoint,
            gossip_tx,
            _gossip_rx: gossip_rx,
            event_tx,
            event_rx,
            quic,
            bandwidth: RwLock::new(BandwidthBudget::new(config.bandwidth)),
            network_type: parking_lot::RwLock::new(NetworkType::Unknown),
            subscriptions: RwLock::new(Vec::new()),
        })
    }

    /// 发布消息到 gossip 网络
    pub fn publish(&mut self, signed: &SignedGossip) -> Result<()> {
        let data = serde_json::to_vec(signed)?;

        // 获取所有订阅的 peer
        let subscriptions = self.subscriptions.read();

        if subscriptions.is_empty() {
            // 没有订阅者，这是正常的（节点刚启动时）
            return Ok(());
        }

        // 广播到所有订阅的 peer
        let mut success = false;
        let mut failed = false;

        for subscription in subscriptions.iter() {
            if subscription.topics.contains(&self.topic) {
                // 序列化消息: [topic_len:4][topic_data][message_data]
                let topic_bytes = self.topic.name.as_bytes();
                let mut message = Vec::with_capacity(4 + topic_bytes.len() + data.len());
                message.extend_from_slice(&(topic_bytes.len() as u32).to_be_bytes());
                message.extend_from_slice(topic_bytes);
                message.extend_from_slice(&data);

                // 发送（这里简化实现，实际应该使用连接池）
                if self.send_to_peer(&subscription.peer, &message).is_ok() {
                    success = true;
                } else {
                    failed = true;
                }
            }
        }

        if failed && !success {
            Err(anyhow!("Gossip 发布失败: 所有 peer 不可用"))
        } else {
            Ok(())
        }
    }

    /// 发送消息到指定 peer（简化实现）
    fn send_to_peer(&self, _peer: &NodeId, _message: &[u8]) -> Result<()> {
        // 在实际实现中，这里应该维护连接池并发送消息
        // 目前简化为成功
        Ok(())
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

    /// 添加 peer 到订阅列表
    pub fn add_peer(&mut self, peer: NodeId) {
        let mut subscriptions = self.subscriptions.write();
        if !subscriptions.iter().any(|s| s.peer == peer) {
            subscriptions.push(PeerSubscription {
                peer,
                topics: vec![self.topic.clone()],
            });
            println!("[Iroh] 添加 peer 到订阅列表: {}", peer);
        }
    }

    /// 从订阅列表中移除 peer
    pub fn remove_peer(&mut self, peer: &NodeId) {
        let mut subscriptions = self.subscriptions.write();
        if let Some(pos) = subscriptions.iter().position(|s| &s.peer == peer) {
            subscriptions.remove(pos);
            println!("[Iroh] 从订阅列表中移除 peer: {}", peer);
        }
    }

    /// 连接到中继节点
    pub async fn connect_to_relay(&mut self, relay_node_id: NodeId) -> Result<()> {
        println!("[中继] 尝试连接到中继节点: {}", relay_node_id);

        // iroh 提供内置的中继支持，这里简化实现
        // 实际应该使用 iroh 的 relay 功能
        Ok(())
    }

    /// 获取下一个事件
    pub async fn next_event(&mut self) -> Option<IrohEvent> {
        self.event_rx.recv().await
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

    /// 获取节点 ID
    pub fn node_id(&self) -> NodeId {
        self.peer_id.clone()
    }

    /// 连接到指定节点
    pub async fn connect(&mut self, node_addr: NodeAddr) -> Result<()> {
        println!("[Iroh] 连接到节点: {}", node_addr.node_id);
        match self.endpoint.connect(node_addr, b"ggb-iroh").await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("连接失败: {:?}", e)),
        }
    }

    /// 获取本地监听地址
    pub fn local_addr(&self) -> Result<String> {
        let addrs = self.endpoint.local_addresses()?;
        addrs
            .first()
            .map(|addr| addr.to_string())
            .ok_or_else(|| anyhow!("没有本地地址"))
    }
}

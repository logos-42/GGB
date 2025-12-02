mod comms;
mod consensus;
mod crypto;
mod device;
#[cfg(feature = "ffi")]
mod ffi;
mod inference;
mod stats;
mod topology;
mod types;

use crate::comms::{CommsConfig, CommsHandle, OutEvent};
use crate::consensus::{ConsensusConfig, ConsensusEngine, SignedGossip};
use crate::crypto::{CryptoConfig, CryptoSuite};
use crate::device::{DeviceCapabilities, DeviceManager};
use crate::inference::{InferenceConfig, InferenceEngine};
use crate::stats::TrainingStatsManager;
use crate::topology::{TopologyConfig, TopologySelector};
use crate::types::{GeoPoint, GgsMessage};
use anyhow::Result;
use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use std::sync::Arc;
use tokio::time::{interval, Duration};

struct AppConfig {
    inference: InferenceConfig,
    comms: CommsConfig,
    topology: TopologyConfig,
    crypto: CryptoConfig,
    consensus: ConsensusConfig,
    device_manager: DeviceManager,
}

impl AppConfig {
    /// 根据设备能力自动调整配置
    pub fn from_device_capabilities(capabilities: DeviceCapabilities) -> Self {
        let model_dim = capabilities.recommended_model_dim();
        let network_type = capabilities.network_type;

        // 根据设备能力调整推理配置
        let inference = InferenceConfig {
            model_dim,
            model_path: None,
        };

        // 根据网络类型调整带宽预算
        let bandwidth_factor = network_type.bandwidth_factor();
        let comms = CommsConfig {
            topic: "ggs-training".into(),
            listen_addr: None,
            quic_bind: Some(std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                9234,
            )),
            quic_bootstrap: Vec::new(),
            bandwidth: crate::comms::BandwidthBudgetConfig {
                sparse_per_window: (12.0 * bandwidth_factor) as u32,
                dense_bytes_per_window: ((256 * 1024) as f32 * bandwidth_factor) as usize,
                window_secs: 60,
            },
        };

        // 根据设备能力调整拓扑配置
        let topology = TopologyConfig {
            max_neighbors: capabilities.recommended_max_neighbors(),
            failover_pool: capabilities.recommended_failover_pool(),
            min_score: 0.15,
            geo_scale_km: 500.0,
            peer_stale_secs: 120,
        };

        Self {
            inference,
            comms,
            topology,
            crypto: CryptoConfig::default(),
            consensus: ConsensusConfig::default(),
            device_manager: DeviceManager::with_capabilities(capabilities),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let device_manager = DeviceManager::new();
        let capabilities = device_manager.get();
        Self::from_device_capabilities(capabilities)
    }
}

struct Node {
    comms: CommsHandle,
    inference: InferenceEngine,
    topology: TopologySelector,
    consensus: ConsensusEngine,
    device_manager: DeviceManager,
    stats: Arc<TrainingStatsManager>,
    tick_counter: u64,
}

impl Node {
    async fn new(config: AppConfig) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let geo = GeoPoint::random(&mut rng);
        let capabilities = config.device_manager.get();
        
        let inference = InferenceEngine::new(config.inference)?;
        let comms = CommsHandle::new(config.comms).await?;
        
        // 设置初始网络类型
        comms.update_network_type(capabilities.network_type);
        
        let topology = TopologySelector::new(geo.clone(), config.topology);
        let crypto_suite = Arc::new(CryptoSuite::new(config.crypto)?);
        let consensus = ConsensusEngine::new(crypto_suite.clone(), config.consensus);
        
        // 初始化统计管理器
        let model_hash = inference.tensor_hash();
        let model_version = 1;
        let stats = Arc::new(TrainingStatsManager::new(model_hash.clone(), model_version));
        
        println!(
            "启动 GGS 节点 => peer: {}, eth {}, sol {} @ ({:.2},{:.2})",
            comms.peer_id,
            crypto_suite.eth_address(),
            crypto_suite.sol_address(),
            geo.lat,
            geo.lon
        );
        println!("模型维度: {}", inference.model_dim());
        println!(
            "设备能力: {}MB 内存, {} 核心, 网络: {:?}, 电池: {:?}",
            capabilities.max_memory_mb,
            capabilities.cpu_cores,
            capabilities.network_type,
            capabilities
                .battery_level
                .map(|l| format!("{:.0}%", l * 100.0))
                .unwrap_or_else(|| "N/A".to_string())
        );
        
        Ok(Self {
            comms,
            inference,
            topology,
            consensus,
            device_manager: config.device_manager,
            stats,
            tick_counter: 0,
        })
    }

    async fn run(mut self) -> Result<()> {
        let capabilities = self.device_manager.get();
        let tick_interval = capabilities.recommended_tick_interval();
        let mut ticker = interval(tick_interval);
        let mut device_refresh = interval(Duration::from_secs(60)); // 每分钟刷新设备状态
        
        println!("训练频率: {:?}", tick_interval);
        
        loop {
            // 检查是否应该暂停训练（低电量）
            let should_pause = {
                let caps = self.device_manager.get();
                caps.should_pause_training()
            };
            
            if should_pause {
                println!("[电池保护] 电量过低，暂停训练");
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
            
            tokio::select! {
                event = self.comms.swarm.select_next_some() => {
                    if let SwarmEvent::Behaviour(out) = event {
                        self.handle_network_event(out).await?;
                    }
                }
                _ = ticker.tick() => {
                    // 动态调整 tick 间隔（如果电池状态变化）
                    let caps = self.device_manager.get();
                    let new_interval = caps.recommended_tick_interval();
                    if new_interval != tick_interval {
                        ticker = interval(new_interval);
                        println!("[自适应] 调整训练频率: {:?}", new_interval);
                    }
                    self.on_tick().await?;
                }
                _ = device_refresh.tick() => {
                    // 定期刷新设备状态（网络类型、电池等）
                    self.device_manager.refresh();
                    let caps = self.device_manager.get();
                    
                    // 更新网络类型
                    let old_network = self.comms.network_type();
                    if caps.network_type != old_network {
                        self.comms.update_network_type(caps.network_type);
                        println!(
                            "[网络切换] {:?} -> {:?}",
                            old_network,
                            caps.network_type
                        );
                    }
                    
                    // 更新电池状态
                    if let Some(level) = caps.battery_level {
                        println!(
                            "[电池状态] 电量: {:.0}%, 充电: {}",
                            level * 100.0,
                            caps.is_charging
                        );
                    }
                }
            }
        }
    }

    async fn on_tick(&mut self) -> Result<()> {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        self.stats.increment_tick();
        
        let hash = self.inference.tensor_hash();
        let version = self.inference.tensor_snapshot().version;
        self.stats.update_model(hash.clone(), version);
        
        let heartbeat = GgsMessage::Heartbeat {
            peer: self.comms.peer_id.to_string(),
            model_hash: hash,
        };
        self.publish_signed(heartbeat).await?;
        self.stats.record_heartbeat_sent();

        let embedding = self.inference.embedding();
        let probe = GgsMessage::SimilarityProbe {
            embedding,
            position: self.topology.position(),
            sender: self.comms.peer_id.to_string(),
        };
        self.publish_signed(probe).await?;
        self.stats.record_probe_sent();

        self.inference.local_train_step();
        self.consensus.prune_stale();
        if self.tick_counter % 12 == 0 {
            self.maybe_broadcast_dense().await?;
        }
        
        // 更新连接的节点数量
        let (primary, _backups) = self.topology.neighbor_sets();
        self.stats.update_connected_peers(primary.len());
        
        // 每 10 个 tick 输出统计摘要
        if self.tick_counter % 10 == 0 {
            let summary = self.stats.get_summary();
            let convergence = self.inference.convergence_score();
            println!("{}", summary.format());
            println!(
                "  收敛度: {:.3} | 参数变化: {:.6} | 标准差: {:.6}",
                convergence,
                self.inference.parameter_change_magnitude(),
                self.inference.parameter_std_dev()
            );
        }
        
        self.check_topology_health();
        Ok(())
    }

    async fn handle_network_event(&mut self, event: OutEvent) -> Result<()> {
        match event {
            OutEvent::Gossipsub(g) => {
                if let libp2p::gossipsub::Event::Message {
                    propagation_source,
                    message,
                    ..
                } = g
                {
                    if let Ok(signed) = serde_json::from_slice::<SignedGossip>(&message.data) {
                        if self.consensus.verify(&signed) {
                            self.handle_signed_message(signed, propagation_source.to_string())
                                .await?;
                        } else {
                            eprintln!("签名验证失败，来自 {:?}", propagation_source);
                        }
                    }
                }
            }
            OutEvent::Mdns(event) => {
                if let libp2p::mdns::Event::Discovered(peers) = event {
                    for (peer, _addr) in peers {
                        println!("通过 mDNS 发现节点 {peer}");
                    }
                }
            }
        }
        Ok(())
    }

    async fn publish_signed(&mut self, payload: GgsMessage) -> Result<()> {
        let signed = self.consensus.sign(payload)?;
        self.comms.publish(&signed)?;
        if !self.comms.broadcast_realtime(&signed).await {
            println!("[FAILOVER] QUIC 广播失败，已回落到纯 Gossip");
        }
        Ok(())
    }

    async fn handle_signed_message(&mut self, signed: SignedGossip, source: String) -> Result<()> {
        match &signed.payload {
            GgsMessage::Heartbeat { peer, .. } => {
                self.consensus.update_stake(peer, 0.0, 0.0, 0.05);
                self.stats.record_heartbeat_received(peer);
                println!("收到 {} 的心跳 (via {source})", peer);
            }
            GgsMessage::SimilarityProbe {
                embedding,
                position,
                sender,
            } => {
                self.stats.record_probe_received(sender);
                let self_embedding = self.inference.embedding();
                self.topology.update_peer(
                    sender,
                    embedding.clone(),
                    position.clone(),
                    &self_embedding,
                );
                if let Some(snapshot) = self.topology.peer_snapshot(sender) {
                    let stake = self.consensus.stake_weight(sender);
                    println!(
                        "拓扑更新：{} => sim {:.3}, geo {:.3}, stake {:.3}, dim {}, pos ({:.1},{:.1})",
                        sender,
                        snapshot.similarity,
                        snapshot.geo_affinity,
                        stake,
                        snapshot.embedding_dim,
                        snapshot.position.lat,
                        snapshot.position.lon
                    );
                }
                if self.should_send_sparse_update(sender) {
                    if self.comms.allow_sparse_update() {
                        let update = self.inference.make_sparse_update(16);
                        let msg = GgsMessage::SparseUpdate {
                            update,
                            sender: self.comms.peer_id.to_string(),
                        };
                        self.publish_signed(msg).await?;
                        self.stats.record_sparse_update_sent(sender);
                    } else {
                        println!("[带宽限制] 本轮跳过稀疏更新");
                    }
                }
            }
            GgsMessage::SparseUpdate { sender, update } => {
                self.inference.apply_sparse_update(update);
                self.consensus.update_stake(sender, 0.1, 0.0, 0.1);
                self.stats.record_sparse_update_received(sender);
                println!("应用来自 {} 的稀疏更新", sender);
            }
            GgsMessage::DenseSnapshot { snapshot, sender } => {
                self.inference.apply_dense_snapshot(snapshot);
                self.consensus.update_stake(sender, 0.0, 0.2, 0.05);
                self.stats.record_dense_snapshot_received(sender);
                println!("融合 {} 的模型快照", sender);
            }
        }
        Ok(())
    }

    fn should_send_sparse_update(&self, target: &str) -> bool {
        let primary = self.topology.select_neighbors();
        if primary.iter().any(|peer| peer == target) {
            return true;
        }
        self.topology.mark_unreachable(target);
        false
    }

    fn check_topology_health(&self) {
        let (primary, backups) = self.topology.neighbor_sets();
        if primary.len() < self.topology.max_neighbors() && !backups.is_empty() {
            println!(
                "[拓扑 Failover] 主邻居 {}/{}，启用备份 {:?}",
                primary.len(),
                self.topology.max_neighbors(),
                backups
            );
        } else if backups.len() < self.topology.failover_pool() {
            println!(
                "[拓扑提示] 备份邻居不足 {}/{}",
                backups.len(),
                self.topology.failover_pool()
            );
        }
    }

    async fn maybe_broadcast_dense(&mut self) -> Result<()> {
        let network_type = self.comms.network_type();
        if !network_type.allows_dense_snapshot() {
            // 移动网络下跳过密集快照
            return Ok(());
        }
        
        let snapshot = self.inference.tensor_snapshot();
        let bytes = snapshot.values.len() * std::mem::size_of::<f32>();
        if self.comms.allow_dense_snapshot(bytes) {
            let msg = GgsMessage::DenseSnapshot {
                snapshot,
                sender: self.comms.peer_id.to_string(),
            };
            self.publish_signed(msg).await?;
            self.stats.record_dense_snapshot_sent();
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let mut stats_output: Option<String> = None;
    let mut node_id: Option<usize> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--stats-output" => {
                if i + 1 < args.len() {
                    stats_output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--node-id" => {
                if i + 1 < args.len() {
                    node_id = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    
    if let Some(id) = node_id {
        println!("节点 ID: {}", id);
    }
    
    let config = AppConfig::default();
    let node = Node::new(config).await?;
    
    // 如果指定了统计输出文件，设置定期导出
    if let Some(output_path) = stats_output {
        let stats_path = std::path::PathBuf::from(&output_path);
        let stats_manager = Arc::clone(&node.stats);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = stats_manager.export_json_to_file(&stats_path) {
                    eprintln!("导出统计数据失败: {:?}", e);
                }
            }
        });
    }
    
    node.run().await
}

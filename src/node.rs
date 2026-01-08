use crate::comms::{CommsHandle, IrohEvent};
use crate::config::AppConfig;
use crate::consensus::{ConsensusEngine, SignedGossip};
use crate::crypto::CryptoSuite;
use crate::device::DeviceManager;
use crate::inference::InferenceEngine;
use crate::stats::TrainingStatsManager;
use crate::topology::TopologySelector;
use crate::types::{GeoPoint, GgbMessage};
use anyhow::Result;
use futures::StreamExt;
use iroh::NodeId;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct Node {
    pub comms: CommsHandle,
    pub inference: InferenceEngine,
    pub topology: TopologySelector,
    pub consensus: ConsensusEngine,
    pub device_manager: DeviceManager,
    pub stats: Arc<TrainingStatsManager>,
    pub tick_counter: u64,
    pub checkpoint_dir: Option<PathBuf>,
    pub checkpoint_interval: u64, // 每 N 个 tick 保存一次 checkpoint
}

impl Node {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let geo = GeoPoint::random(&mut rng);
        let capabilities = config.device_manager.get();

        let inference = InferenceEngine::new(config.inference.clone())?;

        // 尝试加载最新的 checkpoint
        if let Some(ref checkpoint_dir) = config.inference.checkpoint_dir {
            if let Ok(Some(latest_checkpoint)) = InferenceEngine::find_latest_checkpoint(checkpoint_dir) {
                match inference.load_checkpoint(&latest_checkpoint) {
                    Ok(_) => {
                        println!("[Checkpoint] 已加载最新 checkpoint: {:?}", latest_checkpoint);
                    }
                    Err(e) => {
                        eprintln!("[Checkpoint] 加载失败: {:?}", e);
                    }
                }
            }
        }

        let comms = CommsHandle::new(config.comms).await?;

        // 设置初始网络类型
        comms.update_network_type(capabilities.network_type);

        let topology = TopologySelector::new(geo.clone(), config.topology);
        let crypto_suite = Arc::new(CryptoSuite::new(config.crypto)?);
        let consensus = ConsensusEngine::new(crypto_suite.clone(), config.consensus);

        // 初始化统计管理器
        let model_hash = inference.tensor_hash();
        let model_version = inference.tensor_snapshot().version;
        let stats = Arc::new(TrainingStatsManager::new(model_hash.clone(), model_version));

        println!(
            "启动 GGB 节点 => node: {}, eth {}, sol {} @ ({:.2},{:.2})",
            comms.node_id(),
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
            checkpoint_dir: config.inference.checkpoint_dir.clone(),
            checkpoint_interval: 100, // 默认每 100 个 tick 保存一次
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let capabilities = self.device_manager.get();
        let mut tick_interval = capabilities.recommended_tick_interval();
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
                event = self.comms.next_event() => {
                    if let Some(event) = event {
                        self.handle_network_event(event).await?;
                    }
                }
                _ = ticker.tick() => {
                    // 动态调整 tick 间隔（如果电池状态变化）
                    let caps = self.device_manager.get();
                    let new_interval = caps.recommended_tick_interval();
                    if new_interval != tick_interval {
                        tick_interval = new_interval;
                        ticker = interval(tick_interval);
                        println!("[自适应] 调整训练频率: {:?}", tick_interval);
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
                        // 同步更新设备管理器中的网络类型
                        self.device_manager.update_network_type(caps.network_type);
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
                        // 更新设备管理器中的电池状态
                        self.device_manager.update_battery(caps.battery_level, caps.is_charging);
                    }

                    // 更新硬件信息（内存和CPU）
                    self.device_manager.update_hardware(caps.max_memory_mb, caps.cpu_cores);
                }
            }
        }
    }

    async fn on_tick(&mut self) -> Result<()> {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        self.stats.increment_tick();

        // 处理通过 QUIC 接收到的消息
        let quic_messages = self.comms.take_quic_messages();
        for signed in quic_messages {
            if self.consensus.verify(&signed) {
                self.handle_signed_message(signed, "QUIC".to_string()).await?;
            }
        }

        let hash = self.inference.tensor_hash();
        let version = self.inference.tensor_snapshot().version;
        self.stats.update_model(hash.clone(), version);

        let heartbeat = GgbMessage::Heartbeat {
            peer: self.comms.node_id().to_string(),
            model_hash: hash,
        };
        self.publish_signed(heartbeat).await?;
        self.stats.record_heartbeat_sent();

        let embedding = self.inference.embedding();
        let probe = GgbMessage::SimilarityProbe {
            embedding,
            position: self.topology.position(),
            sender: self.comms.node_id().to_string(),
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

        // 定期保存 checkpoint
        if let Some(ref checkpoint_dir) = self.checkpoint_dir {
            if self.tick_counter % self.checkpoint_interval == 0 && self.tick_counter > 0 {
                let checkpoint_path = checkpoint_dir.join(format!("checkpoint_{}", self.tick_counter));
                match self.inference.save_checkpoint_structured(&checkpoint_path) {
                    Ok(_) => {
                        println!("[Checkpoint] 已保存: {:?}", checkpoint_path);
                    }
                    Err(e) => {
                        eprintln!("[Checkpoint] 保存失败: {:?}", e);
                    }
                }
            }
        }

        self.check_topology_health();
        Ok(())
    }

    async fn handle_network_event(&mut self, event: IrohEvent) -> Result<()> {
        match event {
            IrohEvent::Gossip { source, data } => {
                if let Ok(signed) = serde_json::from_slice::<SignedGossip>(&data) {
                    if self.consensus.verify(&signed) {
                        self.handle_signed_message(signed, source.to_string())
                            .await?;
                    } else {
                        eprintln!("签名验证失败，来自 {:?}", source);
                    }
                }
            }
            IrohEvent::PeerDiscovered { peer, addr } => {
                println!("[Iroh] 发现节点 {} @ {}", peer, addr);
                // 将发现的节点添加到订阅列表
                self.comms.add_peer(peer);
            }
            IrohEvent::PeerExpired { peer } => {
                println!("[Iroh] 节点离线 {}", peer);
                self.comms.remove_peer(&peer);
            }
            IrohEvent::ConnectionEstablished { peer } => {
                println!("[Iroh] 连接建立: {}", peer);
                // 连接建立后，可以添加到订阅列表
                self.comms.add_peer(peer);
            }
            IrohEvent::ConnectionClosed { peer } => {
                println!("[Iroh] 连接断开: {}", peer);
                self.comms.remove_peer(&peer);
            }
        }
        Ok(())
    }

    async fn publish_signed(&mut self, payload: GgbMessage) -> Result<()> {
        let signed = self.consensus.sign(payload)?;
        self.comms.publish(&signed)?;
        if !self.comms.broadcast_realtime(&signed).await {
            println!("[FAILOVER] QUIC 广播失败，已回落到纯 Gossip");
        }
        Ok(())
    }

    async fn handle_signed_message(&mut self, signed: SignedGossip, source: String) -> Result<()> {
        match &signed.payload {
            GgbMessage::Heartbeat { peer, .. } => {
                self.consensus.update_stake(peer, 0.0, 0.0, 0.05);
                self.stats.record_heartbeat_received(peer);
                println!("收到 {} 的心跳 (via {source})", peer);
            }
            GgbMessage::SimilarityProbe {
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
                        let msg = GgbMessage::SparseUpdate {
                            update,
                            sender: self.comms.node_id().to_string(),
                        };
                        self.publish_signed(msg).await?;
                        self.stats.record_sparse_update_sent(sender);
                    } else {
                        println!("[带宽限制] 本轮跳过稀疏更新");
                    }
                }
            }
            GgbMessage::SparseUpdate { sender, update } => {
                self.inference.apply_sparse_update(update);
                self.consensus.update_stake(sender, 0.1, 0.0, 0.1);
                self.stats.record_sparse_update_received(sender);
                println!("应用来自 {} 的稀疏更新", sender);
            }
            GgbMessage::DenseSnapshot { snapshot, sender } => {
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
            let msg = GgbMessage::DenseSnapshot {
                snapshot,
                sender: self.comms.node_id().to_string(),
            };
            self.publish_signed(msg).await?;
            self.stats.record_dense_snapshot_sent();
        }
        Ok(())
    }
}
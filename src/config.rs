use crate::comms::{CommsConfig, BandwidthBudgetConfig};
use crate::consensus::ConsensusConfig;
use crate::crypto::CryptoConfig;
use crate::device::{DeviceCapabilities, DeviceManager};
use crate::inference::InferenceConfig;
use crate::topology::TopologyConfig;
use crate::inference::LossType;

#[derive(Clone)]
pub struct AppConfig {
    pub inference: InferenceConfig,
    pub comms: CommsConfig,
    pub topology: TopologyConfig,
    pub crypto: CryptoConfig,
    pub consensus: ConsensusConfig,
    pub device_manager: DeviceManager,
}

impl AppConfig {
    /// 根据设备能力自动调整配置
    pub fn from_device_capabilities(capabilities: DeviceCapabilities) -> Self {
        // 如果检测失败，使用默认桌面配置作为回退
        let capabilities = if capabilities.max_memory_mb == 0 || capabilities.cpu_cores == 0 {
            println!("[警告] 设备检测失败，使用默认桌面配置");
            DeviceCapabilities::default_desktop()
        } else {
            capabilities
        };
        let model_dim = capabilities.recommended_model_dim();
        let network_type = capabilities.network_type;

        // 根据设备能力调整推理配置
        // 支持通过环境变量选择损失函数类型
        let loss_type = if let Ok(loss_env) = std::env::var("GGB_LOSS_TYPE") {
            match loss_env.to_uppercase().as_str() {
                "CROSSENTROPY" | "CE" => {
                    println!("[配置] 使用交叉熵损失函数");
                    LossType::CrossEntropy
                }
                "MAE" => {
                    println!("[配置] 使用平均绝对误差损失函数");
                    LossType::MAE
                }
                "MSE" | _ => {
                    println!("[配置] 使用均方误差损失函数（默认）");
                    LossType::MSE
                }
            }
        } else {
            LossType::MSE
        };

        let inference = InferenceConfig {
            model_dim,
            model_path: None,
            checkpoint_dir: None,
            learning_rate: 0.001,
            use_training: false,
            loss_type,
        };

        // 根据网络类型调整带宽预算
        let bandwidth_factor = network_type.bandwidth_factor();
        let comms = CommsConfig {
            topic: "ggb-training".into(),
            // 使用随机可用端口监听，启用 mDNS 节点发现
            listen_addr: Some("/ip4/0.0.0.0/tcp/0".parse().unwrap()),
            quic_bind: Some(std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                9234,
            )),
            quic_bootstrap: Vec::new(),
            bandwidth: BandwidthBudgetConfig {
                sparse_per_window: (12.0 * bandwidth_factor) as u32,
                dense_bytes_per_window: ((256 * 1024) as f32 * bandwidth_factor) as usize,
                window_secs: 60,
            },
            enable_dht: true,
            bootstrap_peers_file: Some(std::path::PathBuf::from("bootstrap_peers.txt")),
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
        // 检查环境变量，支持使用预设的设备配置
        let capabilities = if let Ok(device_type) = std::env::var("GGB_DEVICE_TYPE") {
            match device_type.as_str() {
                "desktop" | "default" => {
                    println!("[配置] 使用默认桌面设备配置");
                    DeviceCapabilities::default_desktop()
                }
                "low" | "low_end" => {
                    println!("[配置] 使用低端移动设备配置");
                    DeviceCapabilities::low_end_mobile()
                }
                "mid" | "mid_range" => {
                    println!("[配置] 使用中端移动设备配置");
                    DeviceCapabilities::mid_range_mobile()
                }
                "high" | "high_end" => {
                    println!("[配置] 使用高端移动设备配置");
                    DeviceCapabilities::high_end_mobile()
                }
                _ => {
                    println!("[配置] 未知设备类型 '{}'，使用自动检测", device_type);
                    let device_manager = DeviceManager::new();
                    device_manager.get()
                }
            }
        } else {
            let device_manager = DeviceManager::new();
            device_manager.get()
        };

        Self::from_device_capabilities(capabilities)
    }
}
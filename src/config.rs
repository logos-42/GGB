use crate::comms::{CommsConfig, BandwidthBudgetConfig};
use crate::consensus::ConsensusConfig;
use crate::crypto::CryptoConfig;
use crate::device::{DeviceCapabilities, DeviceManager};
use crate::inference::InferenceConfig;
use crate::topology::TopologyConfig;
use crate::inference::LossType;
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub hide_ip: bool,
    pub use_relay: bool,
    pub relay_nodes: Vec<Multiaddr>,
    pub private_network_key: Option<String>,
    pub max_hops: u8,
    pub enable_autonat: bool,
    pub enable_dcutr: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            hide_ip: false,  // 默认不隐藏IP，保持向后兼容
            use_relay: false,
            relay_nodes: Vec::new(),
            private_network_key: None,
            max_hops: 3,
            enable_autonat: true,
            enable_dcutr: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub inference: InferenceConfig,
    pub comms: CommsConfig,
    pub topology: TopologyConfig,
    pub crypto: CryptoConfig,
    pub consensus: ConsensusConfig,
    pub device_manager: DeviceManager,
    pub security: SecurityConfig,
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
            security: SecurityConfig::default(),
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
            security: SecurityConfig::default(),
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

impl SecurityConfig {
    /// 验证安全配置的合理性
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // 检查IP隐藏配置
        if self.hide_ip {
            if self.use_relay && self.relay_nodes.is_empty() {
                errors.push("启用IP隐藏和中继时，必须提供至少一个中继节点".to_string());
            }
            
            if !self.use_relay {
                errors.push("警告：启用IP隐藏但未使用中继，隐私保护可能不完整".to_string());
            }
            
            if self.enable_dcutr {
                errors.push("警告：启用IP隐藏时使用DCUtR可能暴露IP".to_string());
            }
        }
        
        // 检查中继跳数
        if self.max_hops == 0 || self.max_hops > 5 {
            errors.push("中继跳数必须在1-5之间".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// 获取隐私建议
    pub fn get_privacy_advice(&self) -> Vec<String> {
        let mut advice = Vec::new();
        
        if self.hide_ip {
            advice.push("✓ IP隐藏已启用".to_string());
            
            if self.use_relay && !self.relay_nodes.is_empty() {
                advice.push(format!("✓ 使用 {} 个中继节点", self.relay_nodes.len()));
            } else if self.use_relay {
                advice.push("⚠ 启用中继但未配置中继节点".to_string());
            }
            
            if self.enable_dcutr {
                advice.push("⚠ DCUtR已启用，可能尝试建立直接连接".to_string());
            }
        } else {
            advice.push("⚠ IP隐藏未启用，节点IP可能暴露".to_string());
            advice.push("建议：设置 hide_ip = true 并配置中继节点".to_string());
        }
        
        advice
    }
}

impl AppConfig {
    /// 从TOML文件加载配置
    pub fn from_toml_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        
        // 验证配置
        if let Err(errors) = config.security.validate() {
            println!("[配置验证] 发现配置问题：");
            for error in &errors {
                println!("  - {}", error);
            }
        }
        
        // 显示隐私建议
        let advice = config.security.get_privacy_advice();
        if !advice.is_empty() {
            println!("[隐私配置] 当前设置：");
            for item in advice {
                println!("  {}", item);
            }
        }
        
        Ok(config)
    }
    
    /// 验证整个应用配置
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // 验证安全配置
        if let Err(security_errors) = self.security.validate() {
            errors.extend(security_errors);
        }
        
        // 验证通信配置一致性
        if self.security.hide_ip && self.comms.enable_dht {
            errors.push("启用IP隐藏时应禁用公共DHT (enable_dht = false)".to_string());
        }
        
        if self.security.use_relay && self.comms.security.relay_nodes.is_empty() {
            errors.push("启用中继但未配置中继节点".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
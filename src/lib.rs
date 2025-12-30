//! GGB 库模块
//! 
//! 将核心功能导出为库，以便测试和集成使用

#![allow(non_snake_case)] // 允许使用 GGB 作为 crate 名称

// 核心模块
pub mod core;
pub mod privacy;
pub mod network;
pub mod device;
pub mod consensus;
pub mod training;
pub mod inference;

// 可选模块
#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(feature = "blockchain")]
pub mod blockchain;

// 重新导出常用类型
pub use core::config::{AppConfig, ConfigManager, ConfigBuilder};
pub use privacy::{PrivacyConfig, PrivacyLevel, CryptoEngine, CryptoKey};
pub use network::{NetworkConfig, NetworkHandle, BandwidthBudgetConfig};
pub use device::{DeviceConfig, DeviceCapabilities, DeviceManager};
pub use consensus::{ConsensusConfig, ConsensusEngine};
pub use inference::{InferenceConfig, InferenceEngine};
pub use training::{TrainingConfig, TrainingEngine};

// 类型别名
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// GGB 应用
pub struct GgbApp {
    config: AppConfig,
    config_manager: ConfigManager,
    network: Option<NetworkHandle>,
    device_manager: DeviceManager,
}

impl GgbApp {
    /// 创建新的 GGB 应用
    pub fn new(config: AppConfig) -> Result<Self> {
        let config_manager = ConfigManager::new();
        let device_manager = DeviceManager::new();
        
        Ok(Self {
            config,
            config_manager,
            network: None,
            device_manager,
        })
    }
    
    /// 启动应用
    pub async fn start(&mut self) -> Result<()> {
        // 初始化网络
        let network = NetworkHandle::new(self.config.network.clone()).await?;
        self.network = Some(network);
        
        // 启动设备管理器
        self.device_manager.start()?;
        
        Ok(())
    }
    
    /// 停止应用
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(network) = &self.network {
            // 关闭网络连接
        }
        
        self.device_manager.stop()?;
        Ok(())
    }
    
    /// 获取应用状态
    pub fn get_status(&self) -> AppStatus {
        AppStatus {
            config: self.config.clone(),
            device_capabilities: self.device_manager.get_capabilities(),
            network_connected: self.network.is_some(),
        }
    }
}

/// 应用状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppStatus {
    pub config: AppConfig,
    pub device_capabilities: DeviceCapabilities,
    pub network_connected: bool,
}

/// 创建默认配置
pub fn create_default_config() -> AppConfig {
    ConfigBuilder::new().build()
}

/// 从文件加载配置
pub fn load_config_from_file(path: &str) -> Result<AppConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&content)?;
    Ok(config)
}

/// 保存配置到文件
pub fn save_config_to_file(config: &AppConfig, path: &str) -> Result<()> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
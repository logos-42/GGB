//! 隐私保护模块
//! 
//! 提供统一的隐私保护功能，包括加密、流量混淆、元数据保护等。
//! 采用模块化设计，支持灵活的隐私级别配置。

pub mod crypto;
// pub mod overlay; // 移除不必要的模块
// pub mod engine;  // 移除不必要的模块

#[cfg(feature = "zk_proof")]
pub mod zk;

// 重新导出公共接口
pub use crypto::*;

#[cfg(feature = "zk_proof")]
pub use zk::*;

/// 隐私级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PrivacyLevel {
    /// 性能优先：最小化加密
    Performance,
    /// 平衡模式：选择性加密
    Balanced,
    /// 最大隐私：完整加密+混淆
    Maximum,
}

/// 隐私配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrivacyConfig {
    /// 隐私级别
    pub level: PrivacyLevel,
    /// 是否启用流量混淆
    pub enable_traffic_obfuscation: bool,
    /// 是否启用元数据保护
    pub enable_metadata_protection: bool,
    /// 是否启用选择性加密
    pub enable_selective_encryption: bool,
    /// 加密算法配置
    pub crypto_config: crypto::CryptoConfig,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            level: PrivacyLevel::Balanced,
            enable_traffic_obfuscation: true,
            enable_metadata_protection: true,
            enable_selective_encryption: true,
            crypto_config: crypto::CryptoConfig::default(),
        }
    }
}

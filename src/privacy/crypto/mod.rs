//! 加密子系统
//! 
//! 提供高性能、可配置的加密功能，支持多种算法和硬件加速。

mod algorithms;
// mod engine;  // 移除不必要的模块
// mod hardware;  // 移除不必要的模块

// 重新导出公共接口
pub use algorithms::*;

#[cfg(feature = "zk_proof")]
pub use crate::zk::*;

/// 加密算法枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EncryptionAlgorithm {
    /// ChaCha20-Poly1305 (移动设备友好)
    ChaCha20Poly1305,
    /// AES-256-CBC (硬件加速)
    Aes256Cbc,
    /// Blake3 哈希加密
    Blake3,
}

/// 加密配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CryptoConfig {
    /// 默认加密算法
    pub default_algorithm: EncryptionAlgorithm,
    /// 是否启用硬件加速
    pub enable_hardware_acceleration: bool,
    /// 是否启用批量处理
    pub enable_batch_processing: bool,
    /// 是否启用零拷贝
    pub enable_zero_copy: bool,
    /// 批量处理大小（字节）
    pub batch_size_bytes: usize,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            default_algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
            enable_hardware_acceleration: true,
            enable_batch_processing: true,
            enable_zero_copy: true,
            batch_size_bytes: 1024 * 1024, // 1MB
        }
    }
}

/// 性能指标
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CryptoMetrics {
    /// 加密延迟（毫秒）
    pub encryption_latency_ms: f64,
    /// 解密延迟（毫秒）
    pub decryption_latency_ms: f64,
    /// 吞吐量（MB/s）
    pub throughput_mbps: f64,
    /// CPU 使用率（%）
    pub cpu_usage_percent: f64,
    /// 内存使用（MB）
    pub memory_usage_mb: f64,
}

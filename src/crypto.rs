//! 加密模块 - 已重构为子模块
//! 
//! 此文件已重构，请使用 `crypto::` 命名空间下的模块：
//! 
//! - `crypto::base` - 基础加密功能
//! - `crypto::high_performance` - 高性能加密引擎
//! - `crypto::batch` - 批量处理功能
//! - `crypto::hardware` - 硬件加速支持
//! - `crypto::selective` - 选择性加密
//! - `crypto::zero_copy` - 零拷贝加密
//! 
//! 常用类型已重新导出到 `crypto` 命名空间。

// 重新导出所有功能
pub use crate::crypto::*;

// 为了向后兼容，也导出常用类型
pub use crate::crypto::base::{
    CryptoConfig, CryptoSuite, EthSignature, SolSignature, SignatureBundle,
};

pub use crate::crypto::high_performance::{
    HighPerformanceCrypto, HighPerformanceCryptoConfig,
};

pub use crate::crypto::{
    EncryptionAlgorithm, PrivacyLevel, PerformanceMetrics, 
    PrivacyPerformanceMetrics, PerformanceComparison, BalanceMode,
};
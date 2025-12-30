//! 统一加密引擎
//! 
//! 提供高性能、可配置的加密引擎，支持批量处理、并行计算和性能监控。

use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use rayon::prelude::*;

use super::{CryptoConfig, CryptoMetrics, EncryptionAlgorithm, EncryptorFactory, CryptoKey};
use super::algorithms::{Encryptor, EncryptedData};

/// 统一加密引擎
pub struct CryptoEngine {
    config: CryptoConfig,
    encryptor: Box<dyn Encryptor>,
    metrics: Arc<RwLock<CryptoMetrics>>,
}

impl CryptoEngine {
    /// 创建新的加密引擎
    pub fn new(config: CryptoConfig) -> Result<Self> {
        let encryptor = EncryptorFactory::create(config.default_algorithm);
        
        Ok(Self {
            config,
            encryptor,
            metrics: Arc::new(RwLock::new(CryptoMetrics {
                encryption_latency_ms: 0.0,
                decryption_latency_ms: 0.0,
                throughput_mbps: 0.0,
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
            })),
        })
    }
    
    /// 加密数据
    pub fn encrypt(&self, plaintext: &[u8], key: &CryptoKey) -> Result<EncryptedData> {
        let start = Instant::now();
        
        let result = self.encryptor.encrypt(plaintext, key);
        
        let duration = start.elapsed();
        self.update_encryption_metrics(duration, plaintext.len());
        
        result
    }
    
    /// 解密数据
    pub fn decrypt(&self, encrypted: &EncryptedData, key: &CryptoKey) -> Result<Vec<u8>> {
        let start = Instant::now();
        
        let result = self.encryptor.decrypt(encrypted, key);
        
        let duration = start.elapsed();
        self.update_decryption_metrics(duration, encrypted.ciphertext.len());
        
        result
    }
    
    /// 批量加密
    pub fn encrypt_batch(&self, plaintexts: &[&[u8]], key: &CryptoKey) -> Result<Vec<EncryptedData>> {
        if !self.config.enable_batch_processing {
            return plaintexts.iter()
                .map(|&pt| self.encrypt(pt, key))
                .collect();
        }
        
        let start = Instant::now();
        let total_size: usize = plaintexts.iter().map(|pt| pt.len()).sum();
        
        let results: Vec<Result<EncryptedData>> = if self.config.enable_zero_copy {
            // 使用并行处理
            plaintexts.par_iter()
                .map(|&pt| self.encryptor.encrypt(pt, key))
                .collect()
        } else {
            // 顺序处理
            plaintexts.iter()
                .map(|&pt| self.encryptor.encrypt(pt, key))
                .collect()
        };
        
        let duration = start.elapsed();
        self.update_encryption_metrics(duration, total_size);
        
        results.into_iter().collect()
    }
    
    /// 批量解密
    pub fn decrypt_batch(&self, encrypted_list: &[&EncryptedData], key: &CryptoKey) -> Result<Vec<Vec<u8>>> {
        if !self.config.enable_batch_processing {
            return encrypted_list.iter()
                .map(|&enc| self.decrypt(enc, key))
                .collect();
        }
        
        let start = Instant::now();
        let total_size: usize = encrypted_list.iter().map(|enc| enc.ciphertext.len()).sum();
        
        let results: Vec<Result<Vec<u8>>> = if self.config.enable_zero_copy {
            // 使用并行处理
            encrypted_list.par_iter()
                .map(|&enc| self.encryptor.decrypt(enc, key))
                .collect()
        } else {
            // 顺序处理
            encrypted_list.iter()
                .map(|&enc| self.encryptor.decrypt(enc, key))
                .collect()
        };
        
        let duration = start.elapsed();
        self.update_decryption_metrics(duration, total_size);
        
        results.into_iter().collect()
    }
    
    /// 获取性能指标
    pub fn get_metrics(&self) -> CryptoMetrics {
        self.metrics.read().clone()
    }
    
    /// 更新加密指标
    fn update_encryption_metrics(&self, duration: Duration, data_size: usize) {
        let mut metrics = self.metrics.write();
        
        // 更新延迟（指数移动平均）
        let latency_ms = duration.as_secs_f64() * 1000.0;
        metrics.encryption_latency_ms = 0.9 * metrics.encryption_latency_ms + 0.1 * latency_ms;
        
        // 更新吞吐量
        let throughput_mbps = (data_size as f64 / duration.as_secs_f64()) / (1024.0 * 1024.0);
        metrics.throughput_mbps = 0.9 * metrics.throughput_mbps + 0.1 * throughput_mbps;
    }
    
    /// 更新解密指标
    fn update_decryption_metrics(&self, duration: Duration, data_size: usize) {
        let mut metrics = self.metrics.write();
        
        // 更新延迟（指数移动平均）
        let latency_ms = duration.as_secs_f64() * 1000.0;
        metrics.decryption_latency_ms = 0.9 * metrics.decryption_latency_ms + 0.1 * latency_ms;
        
        // 更新吞吐量
        let throughput_mbps = (data_size as f64 / duration.as_secs_f64()) / (1024.0 * 1024.0);
        metrics.throughput_mbps = 0.9 * metrics.throughput_mbps + 0.1 * throughput_mbps;
    }
}

/// 加密引擎构建器
pub struct CryptoEngineBuilder {
    config: CryptoConfig,
}

impl CryptoEngineBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: CryptoConfig::default(),
        }
    }
    
    /// 设置加密算法
    pub fn algorithm(mut self, algorithm: EncryptionAlgorithm) -> Self {
        self.config.default_algorithm = algorithm;
        self
    }
    
    /// 启用/禁用硬件加速
    pub fn hardware_acceleration(mut self, enable: bool) -> Self {
        self.config.enable_hardware_acceleration = enable;
        self
    }
    
    /// 启用/禁用批量处理
    pub fn batch_processing(mut self, enable: bool) -> Self {
        self.config.enable_batch_processing = enable;
        self
    }
    
    /// 启用/禁用零拷贝
    pub fn zero_copy(mut self, enable: bool) -> Self {
        self.config.enable_zero_copy = enable;
        self
    }
    
    /// 设置批量大小
    pub fn batch_size(mut self, size_bytes: usize) -> Self {
        self.config.batch_size_bytes = size_bytes;
        self
    }
    
    /// 构建加密引擎
    pub fn build(self) -> Result<CryptoEngine> {
        CryptoEngine::new(self.config)
    }
}

impl Default for CryptoEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

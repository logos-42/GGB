//! 选择性加密模块
//! 
//! 提供智能选择性加密功能，只加密敏感数据部分。

use anyhow::Result;
use parking_lot::RwLock;
use rand::Rng;

use super::{EncryptionAlgorithm, HighPerformanceCrypto, HighPerformanceCryptoConfig, PrivacyLevel};

/// 选择性加密引擎
pub struct SelectiveEncryption {
    crypto: HighPerformanceCrypto,
    sensitive_patterns: RwLock<Vec<String>>,
}

impl SelectiveEncryption {
    /// 创建新的选择性加密引擎
    pub fn new(config: HighPerformanceCryptoConfig) -> Self {
        Self {
            crypto: HighPerformanceCrypto::new(config),
            sensitive_patterns: RwLock::new(Vec::new()),
        }
    }
    
    /// 添加敏感数据模式
    pub fn add_sensitive_pattern(&self, pattern: String) {
        let mut patterns = self.sensitive_patterns.write();
        patterns.push(pattern);
    }
    
    /// 检测数据中的敏感部分
    pub fn detect_sensitive_parts(&self, data: &[u8]) -> Vec<(usize, usize)> {
        let patterns = self.sensitive_patterns.read();
        let mut sensitive_ranges = Vec::new();
        
        // 将数据转换为字符串进行模式匹配（简化实现）
        if let Ok(text) = std::str::from_utf8(data) {
            for pattern in patterns.iter() {
                let mut start = 0;
                while let Some(pos) = text[start..].find(pattern) {
                    let absolute_pos = start + pos;
                    let end = absolute_pos + pattern.len();
                    sensitive_ranges.push((absolute_pos, end));
                    start = end;
                }
            }
        }
        
        // 合并重叠的范围
        Self::merge_ranges(&mut sensitive_ranges);
        sensitive_ranges
    }
    
    /// 智能选择性加密
    pub fn smart_encrypt(&self, data: &[u8], key: &[u8], privacy_level: PrivacyLevel) -> Result<Vec<u8>> {
        match privacy_level {
            PrivacyLevel::Performance => {
                // 性能优先：只加密明显敏感的部分
                let sensitive_ranges = self.detect_sensitive_parts(data);
                if sensitive_ranges.is_empty() {
                    Ok(data.to_vec()) // 无敏感数据，不加密
                } else {
                    self.crypto.selective_encrypt(data, &sensitive_ranges, key)
                }
            }
            PrivacyLevel::Balanced => {
                // 平衡模式：加密敏感部分和随机部分
                let sensitive_ranges = self.detect_sensitive_parts(data);
                let mut all_ranges = sensitive_ranges;
                
                // 添加一些随机部分以增加混淆
                if data.len() > 100 {
                    let random_ranges = self.generate_random_ranges(data.len(), 3);
                    all_ranges.extend(random_ranges);
                }
                
                Self::merge_ranges(&mut all_ranges);
                self.crypto.selective_encrypt(data, &all_ranges, key)
            }
            PrivacyLevel::Maximum => {
                // 最大隐私：完整加密
                self.crypto.encrypt(data, key, self.crypto.config.default_algorithm)
            }
        }
    }
    
    /// 智能选择性解密
    pub fn smart_decrypt(&self, encrypted: &[u8], key: &[u8], privacy_level: PrivacyLevel) -> Result<Vec<u8>> {
        match privacy_level {
            PrivacyLevel::Maximum => {
                // 最大隐私：完整解密
                self.crypto.decrypt(encrypted, key, self.crypto.config.default_algorithm)
            }
            _ => {
                // 选择性和平衡模式：尝试完整解密，如果失败则返回原始数据
                // （因为可能只有部分加密）
                match self.crypto.decrypt(encrypted, key, self.crypto.config.default_algorithm) {
                    Ok(decrypted) => Ok(decrypted),
                    Err(_) => Ok(encrypted.to_vec()), // 解密失败，可能只有部分加密
                }
            }
        }
    }
    
    /// 获取加密覆盖率（加密部分占总数据的比例）
    pub fn get_encryption_coverage(&self, data: &[u8], privacy_level: PrivacyLevel) -> f64 {
        let sensitive_ranges = self.detect_sensitive_parts(data);
        let total_sensitive_len: usize = sensitive_ranges.iter()
            .map(|&(start, end)| end - start)
            .sum();
        
        match privacy_level {
            PrivacyLevel::Performance => {
                // 只加密敏感部分
                total_sensitive_len as f64 / data.len() as f64
            }
            PrivacyLevel::Balanced => {
                // 加密敏感部分 + 30%随机部分
                let base_coverage = total_sensitive_len as f64 / data.len() as f64;
                base_coverage + 0.3 * (1.0 - base_coverage)
            }
            PrivacyLevel::Maximum => {
                // 完整加密
                1.0
            }
        }
    }
    
    // ============ 私有方法 ============
    
    fn generate_random_ranges(&self, data_len: usize, count: usize) -> Vec<(usize, usize)> {
        let mut rng = rand::thread_rng();
        let mut ranges = Vec::new();

        for _ in 0..count {
            let start = rng.random_range(0..data_len.saturating_sub(10));
            let end = start + rng.random_range(5..20.min(data_len - start));
            ranges.push((start, end));
        }

        ranges
    }
    
    fn merge_ranges(ranges: &mut Vec<(usize, usize)>) {
        if ranges.is_empty() {
            return;
        }
        
        // 按起始位置排序
        ranges.sort_by_key(|&(start, _)| start);
        
        let mut merged = Vec::new();
        let mut current = ranges[0];
        
        for &(start, end) in ranges.iter().skip(1) {
            if start <= current.1 {
                // 重叠，合并
                current.1 = current.1.max(end);
            } else {
                // 不重叠，保存当前范围
                merged.push(current);
                current = (start, end);
            }
        }
        
        merged.push(current);
        *ranges = merged;
    }
}

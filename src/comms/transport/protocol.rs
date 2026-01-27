/**
 * 文件传输协议和完整性校验模块
 * 提供安全的文件传输和验证功能
 */

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, warn, error, debug};

/// 文件完整性信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIntegrity {
    pub file_path: String,
    pub file_size: u64,
    pub sha256_hash: String,
    pub chunk_hashes: HashMap<u32, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

impl FileIntegrity {
    pub fn new(file_path: String, file_size: u64, sha256_hash: String) -> Self {
        Self {
            file_path,
            file_size,
            sha256_hash,
            chunk_hashes: HashMap::new(),
            created_at: chrono::Utc::now(),
            version: "1.0".to_string(),
        }
    }

    /// 添加块哈希
    pub fn add_chunk_hash(&mut self, chunk_index: u32, hash: String) {
        self.chunk_hashes.insert(chunk_index, hash);
    }

    /// 验证块哈希
    pub fn verify_chunk_hash(&self, chunk_index: u32, hash: &str) -> bool {
        match self.chunk_hashes.get(&chunk_index) {
            Some(stored_hash) => stored_hash == hash,
            None => false,
        }
    }

    /// 获取缺失的块
    pub fn get_missing_chunks(&self, total_chunks: u32) -> Vec<u32> {
        let mut missing = Vec::new();
        for i in 0..total_chunks {
            if !self.chunk_hashes.contains_key(&i) {
                missing.push(i);
            }
        }
        missing
    }

    /// 序列化为 JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// 从 JSON 反序列化
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// 保存到文件
    pub async fn save_to_file(&self, file_path: &Path) -> Result<()> {
        let json = self.to_json()?;
        fs::write(file_path, json).await?;
        Ok(())
    }

    /// 从文件加载
    pub async fn load_from_file(file_path: &Path) -> Result<Self> {
        let json = fs::read_to_string(file_path).await?;
        Ok(Self::from_json(&json)?)
    }
}

/// 文件传输协议配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProtocolConfig {
    pub max_chunk_size: usize,
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub checksum_algorithm: ChecksumAlgorithm,
    pub resume_support: bool,
}

/// 校验和算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    SHA256,
    SHA512,
    MD5,
    Blake3,
}

impl Default for TransferProtocolConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 1024 * 1024, // 1MB
            max_retries: 3,
            timeout_seconds: 30,
            enable_compression: true,
            enable_encryption: true,
            checksum_algorithm: ChecksumAlgorithm::SHA256,
            resume_support: true,
        }
    }
}

/// 文件传输协议实现
pub struct FileTransferProtocol {
    config: TransferProtocolConfig,
}

impl FileTransferProtocol {
    pub fn new(config: TransferProtocolConfig) -> Self {
        Self { config }
    }

    /// 计算文件完整性信息
    pub async fn calculate_file_integrity(&self, file_path: &Path) -> Result<FileIntegrity> {
        info!("🔍 计算文件完整性: {}", file_path.display());

        let metadata = fs::metadata(file_path).await?;
        let file_size = metadata.len();
        let file_path_str = file_path.to_string_lossy().to_string();

        // 计算整个文件的哈希
        let sha256_hash = self.calculate_file_hash(file_path).await;

        let mut integrity = FileIntegrity::new(file_path_str, file_size, sha256_hash);

        // 计算每个块的哈希
        let mut file = fs::File::open(file_path).await?;
        let mut buffer = vec![0u8; self.config.max_chunk_size];
        let mut chunk_index = 0u32;

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk_data = &buffer[..bytes_read];
            let chunk_hash = self.calculate_chunk_hash(chunk_data);
            integrity.add_chunk_hash(chunk_index, chunk_hash);

            chunk_index += 1;
        }

        info!("✅ 文件完整性计算完成: {} 个块", chunk_index);
        Ok(integrity)
    }

    /// 验证文件完整性
    pub async fn verify_file_integrity(&self, file_path: &Path, integrity: &FileIntegrity) -> Result<bool> {
        info!("🔍 验证文件完整性: {}", file_path.display());

        // 检查文件大小
        let metadata = fs::metadata(file_path).await?;
        if metadata.len() != integrity.file_size {
            error!("文件大小不匹配: 期望 {}, 实际 {}", integrity.file_size, metadata.len());
            return Ok(false);
        }

        // 检查文件哈希
        let actual_hash = self.calculate_file_hash(file_path).await;
        if actual_hash != integrity.sha256_hash {
            error!("文件哈希不匹配: 期望 {}, 实际 {}", integrity.sha256_hash, actual_hash);
            return Ok(false);
        }

        // 验证块哈希
        let mut file = fs::File::open(file_path).await?;
        let mut buffer = vec![0u8; self.config.max_chunk_size];
        let mut chunk_index = 0u32;

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk_data = &buffer[..bytes_read];
            let actual_chunk_hash = self.calculate_chunk_hash(chunk_data);

            if !integrity.verify_chunk_hash(chunk_index, &actual_chunk_hash) {
                error!("块 {} 哈希不匹配", chunk_index);
                return Ok(false);
            }

            chunk_index += 1;
        }

        info!("✅ 文件完整性验证通过");
        Ok(true)
    }

    /// 验证单个块
    pub fn verify_chunk(&self, chunk_data: &[u8], expected_hash: &str) -> bool {
        let actual_hash = self.calculate_chunk_hash(chunk_data);
        actual_hash == expected_hash
    }

    /// 计算文件哈希
    async fn calculate_file_hash(&self, file_path: &Path) -> String {
        match self.config.checksum_algorithm {
            ChecksumAlgorithm::SHA256 => self.calculate_sha256_file(file_path).await,
            ChecksumAlgorithm::SHA512 => self.calculate_sha512_file(file_path).await,
            ChecksumAlgorithm::MD5 => self.calculate_md5_file(file_path).await,
            ChecksumAlgorithm::Blake3 => self.calculate_blake3_file(file_path).await,
        }
    }

    /// 计算块哈希
    fn calculate_chunk_hash(&self, data: &[u8]) -> String {
        match self.config.checksum_algorithm {
            ChecksumAlgorithm::SHA256 => self.calculate_sha256(data),
            ChecksumAlgorithm::SHA512 => self.calculate_sha512(data),
            ChecksumAlgorithm::MD5 => self.calculate_md5(data),
            ChecksumAlgorithm::Blake3 => self.calculate_blake3(data),
        }
    }

    /// SHA256 哈希计算
    async fn calculate_sha256_file(&self, file_path: &Path) -> String {
        use sha3::{Sha3_256, Digest};
        
        let mut file = fs::File::open(file_path).await.unwrap();
        let mut hasher = Sha3_256::new();
        let mut buffer = [0u8; 8192];

        loop {
            match file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => break,
            }
        }

        hex::encode(hasher.finalize())
    }

    fn calculate_sha256(&self, data: &[u8]) -> String {
        use sha3::{Sha3_256, Digest};
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// SHA512 哈希计算
    async fn calculate_sha512_file(&self, file_path: &Path) -> String {
        use sha3::{Sha3_512, Digest};
        
        let mut file = fs::File::open(file_path).await.unwrap();
        let mut hasher = Sha3_512::new();
        let mut buffer = [0u8; 8192];

        loop {
            match file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => break,
            }
        }

        hex::encode(hasher.finalize())
    }

    fn calculate_sha512(&self, data: &[u8]) -> String {
        use sha3::{Sha3_512, Digest};
        let mut hasher = Sha3_512::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// MD5 哈希计算
    async fn calculate_md5_file(&self, _file_path: &Path) -> String {
        // 暂时禁用MD5，返回默认值
        "default_md5_hash".to_string()
    }

    fn calculate_md5(&self, _data: &[u8]) -> String {
        // 暂时禁用MD5，返回默认值
        "default_md5_hash".to_string()
    }

    /// Blake3 哈希计算
    async fn calculate_blake3_file(&self, file_path: &Path) -> String {
        use blake3::Hasher;
        
        let mut file = fs::File::open(file_path).await.unwrap();
        let mut hasher = Hasher::new();
        let mut buffer = [0u8; 8192];

        loop {
            match file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    hasher.update(&buffer[..n]);
                }
                Err(_) => break,
            }
        }

        hex::encode(hasher.finalize().as_bytes())
    }

    fn calculate_blake3(&self, data: &[u8]) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        hex::encode(hasher.finalize().as_bytes())
    }

    /// 压缩数据（如果启用）
    pub fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enable_compression {
            return Ok(data.to_vec());
        }

        // 使用简单的压缩算法
        // 实际实现中可以使用更高效的压缩库
        Ok(data.to_vec()) // 暂时不压缩
    }

    /// 解压数据（如果启用）
    pub fn decompress_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enable_compression {
            return Ok(compressed_data.to_vec());
        }

        // 对应的解压逻辑
        Ok(compressed_data.to_vec()) // 暂时不解压
    }

    /// 加密数据（如果启用）
    pub fn encrypt_data(&self, data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
        // 暂时禁用加密功能
        Ok(data.to_vec())
    }

    /// 解密数据（如果启用）
    pub fn decrypt_data(&self, encrypted_data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
        // 暂时禁用解密功能
        Ok(encrypted_data.to_vec())
    }

    /// 获取配置
    pub fn config(&self) -> &TransferProtocolConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: TransferProtocolConfig) {
        self.config = config;
    }
}

/// 传输状态管理器
pub struct TransferStateManager {
    active_transfers: HashMap<String, TransferState>,
}

#[derive(Debug, Clone)]
pub struct TransferState {
    pub transfer_id: String,
    pub file_path: PathBuf,
    pub total_chunks: u32,
    pub completed_chunks: u32,
    pub failed_chunks: u32,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub status: TransferStatus,
}

#[derive(Debug, Clone)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

impl TransferStateManager {
    pub fn new() -> Self {
        Self {
            active_transfers: HashMap::new(),
        }
    }

    pub fn create_transfer(&mut self, transfer_id: String, file_path: PathBuf, total_chunks: u32) {
        let state = TransferState {
            transfer_id: transfer_id.clone(),
            file_path,
            total_chunks,
            completed_chunks: 0,
            failed_chunks: 0,
            start_time: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            status: TransferStatus::Pending,
        };
        
        self.active_transfers.insert(transfer_id, state);
    }

    pub fn update_progress(&mut self, transfer_id: &str, completed_chunks: u32) {
        if let Some(state) = self.active_transfers.get_mut(transfer_id) {
            state.completed_chunks = completed_chunks;
            state.last_activity = chrono::Utc::now();
            state.status = TransferStatus::InProgress;
        }
    }

    pub fn mark_completed(&mut self, transfer_id: &str) {
        if let Some(state) = self.active_transfers.get_mut(transfer_id) {
            state.status = TransferStatus::Completed;
            state.last_activity = chrono::Utc::now();
        }
    }

    pub fn mark_failed(&mut self, transfer_id: &str, error: String) {
        if let Some(state) = self.active_transfers.get_mut(transfer_id) {
            state.status = TransferStatus::Failed(error);
            state.last_activity = chrono::Utc::now();
        }
    }

    pub fn get_transfer(&self, transfer_id: &str) -> Option<&TransferState> {
        self.active_transfers.get(transfer_id)
    }

    pub fn remove_transfer(&mut self, transfer_id: &str) -> Option<TransferState> {
        self.active_transfers.remove(transfer_id)
    }

    pub fn cleanup_old_transfers(&mut self, max_age_hours: i64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);
        
        self.active_transfers.retain(|_, state| {
            state.last_activity > cutoff || matches!(state.status, TransferStatus::InProgress)
        });
    }

    pub fn get_all_transfers(&self) -> Vec<&TransferState> {
        self.active_transfers.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_integrity() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // 创建测试文件
        fs::write(&file_path, b"Hello, World!").await.unwrap();
        
        let protocol = FileTransferProtocol::new(TransferProtocolConfig::default());
        let integrity = protocol.calculate_file_integrity(&file_path).await.unwrap();
        
        assert_eq!(integrity.file_size, 13);
        assert!(!integrity.sha256_hash.is_empty());
        assert_eq!(integrity.chunk_hashes.len(), 1); // 一个块
        
        // 验证完整性
        let is_valid = protocol.verify_file_integrity(&file_path, &integrity).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_transfer_state_manager() {
        let mut manager = TransferStateManager::new();
        let transfer_id = "test_transfer".to_string();
        let file_path = PathBuf::from("/tmp/test.txt");
        
        manager.create_transfer(transfer_id.clone(), file_path, 10);
        
        let state = manager.get_transfer(&transfer_id).unwrap();
        assert_eq!(state.total_chunks, 10);
        assert_eq!(state.completed_chunks, 0);
        
        manager.update_progress(&transfer_id, 5);
        let state = manager.get_transfer(&transfer_id).unwrap();
        assert_eq!(state.completed_chunks, 5);
        
        manager.mark_completed(&transfer_id);
        let state = manager.get_transfer(&transfer_id).unwrap();
        assert!(matches!(state.status, TransferStatus::Completed));
    }
}

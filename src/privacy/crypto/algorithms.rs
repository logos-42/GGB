//! 加密算法实现
//! 
//! 提供多种加密算法的统一实现。

use anyhow::{anyhow, Result};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, KeyInit};
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use blake3::Hasher;
use rand::RngCore;
use zeroize::Zeroize;

use super::EncryptionAlgorithm;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

/// 加密密钥
#[derive(Clone)]
pub struct CryptoKey {
    key: Vec<u8>,
    algorithm: EncryptionAlgorithm,
}

impl CryptoKey {
    /// 生成新的加密密钥
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        let key_size = match algorithm {
            EncryptionAlgorithm::ChaCha20Poly1305 => 32,
            EncryptionAlgorithm::Aes256Cbc => 32,
            EncryptionAlgorithm::Blake3 => 32,
        };
        
        let mut key = vec![0u8; key_size];
        rand::thread_rng().fill_bytes(&mut key);
        
        Self { key, algorithm }
    }
    
    /// 从字节创建密钥
    pub fn from_bytes(bytes: &[u8], algorithm: EncryptionAlgorithm) -> Result<Self> {
        let key_size = match algorithm {
            EncryptionAlgorithm::ChaCha20Poly1305 => 32,
            EncryptionAlgorithm::Aes256Cbc => 32,
            EncryptionAlgorithm::Blake3 => 32,
        };
        
        if bytes.len() != key_size {
            return Err(anyhow!("Invalid key size: expected {}, got {}", key_size, bytes.len()));
        }
        
        Ok(Self {
            key: bytes.to_vec(),
            algorithm,
        })
    }
    
    /// 获取密钥字节
    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }
    
    /// 获取算法类型
    pub fn algorithm(&self) -> EncryptionAlgorithm {
        self.algorithm
    }
}

impl Drop for CryptoKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

/// 加密数据
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Option<Vec<u8>>,
    pub algorithm: EncryptionAlgorithm,
}

/// 加密器接口
pub trait Encryptor {
    /// 加密数据
    fn encrypt(&self, plaintext: &[u8], key: &CryptoKey) -> Result<EncryptedData>;
    
    /// 解密数据
    fn decrypt(&self, encrypted: &EncryptedData, key: &CryptoKey) -> Result<Vec<u8>>;
}

/// ChaCha20-Poly1305 加密器
pub struct ChaCha20Poly1305Encryptor;

impl Encryptor for ChaCha20Poly1305Encryptor {
    fn encrypt(&self, plaintext: &[u8], key: &CryptoKey) -> Result<EncryptedData> {
        if key.algorithm() != EncryptionAlgorithm::ChaCha20Poly1305 {
            return Err(anyhow!("Invalid key algorithm for ChaCha20Poly1305"));
        }
        
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        Ok(EncryptedData {
            ciphertext,
            nonce: Some(nonce.to_vec()),
            algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
        })
    }
    
    fn decrypt(&self, encrypted: &EncryptedData, key: &CryptoKey) -> Result<Vec<u8>> {
        if key.algorithm() != EncryptionAlgorithm::ChaCha20Poly1305 {
            return Err(anyhow!("Invalid key algorithm for ChaCha20Poly1305"));
        }
        
        let nonce = encrypted.nonce.as_ref()
            .ok_or_else(|| anyhow!("Missing nonce for ChaCha20Poly1305"))?;
        
        if nonce.len() != 12 {
            return Err(anyhow!("Invalid nonce size: expected 12, got {}", nonce.len()));
        }
        
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
}

/// AES-256-CBC 加密器
pub struct Aes256CbcEncryptor;

impl Encryptor for Aes256CbcEncryptor {
    fn encrypt(&self, plaintext: &[u8], key: &CryptoKey) -> Result<EncryptedData> {
        if key.algorithm() != EncryptionAlgorithm::Aes256Cbc {
            return Err(anyhow!("Invalid key algorithm for AES-256-CBC"));
        }
        
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);
        
        let cipher = Aes256Cbc::new_from_slices(key.as_bytes(), &iv)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        
        let ciphertext = cipher.encrypt_vec(plaintext);
        
        Ok(EncryptedData {
            ciphertext,
            nonce: Some(iv.to_vec()),
            algorithm: EncryptionAlgorithm::Aes256Cbc,
        })
    }
    
    fn decrypt(&self, encrypted: &EncryptedData, key: &CryptoKey) -> Result<Vec<u8>> {
        if key.algorithm() != EncryptionAlgorithm::Aes256Cbc {
            return Err(anyhow!("Invalid key algorithm for AES-256-CBC"));
        }
        
        let iv = encrypted.nonce.as_ref()
            .ok_or_else(|| anyhow!("Missing IV for AES-256-CBC"))?;
        
        if iv.len() != 16 {
            return Err(anyhow!("Invalid IV size: expected 16, got {}", iv.len()));
        }
        
        let cipher = Aes256Cbc::new_from_slices(key.as_bytes(), iv)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        
        let plaintext = cipher.decrypt_vec(&encrypted.ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
}

/// Blake3 哈希加密器
pub struct Blake3Encryptor;

impl Encryptor for Blake3Encryptor {
    fn encrypt(&self, plaintext: &[u8], key: &CryptoKey) -> Result<EncryptedData> {
        if key.algorithm() != EncryptionAlgorithm::Blake3 {
            return Err(anyhow!("Invalid key algorithm for Blake3"));
        }
        
        let mut hasher = Hasher::new_keyed(key.as_bytes());
        hasher.update(plaintext);
        let hash = hasher.finalize();
        
        // 将哈希与明文进行XOR操作
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        let hash_bytes = hash.as_bytes();
        
        for (i, &byte) in plaintext.iter().enumerate() {
            let hash_byte = hash_bytes[i % hash_bytes.len()];
            ciphertext.push(byte ^ hash_byte);
        }
        
        Ok(EncryptedData {
            ciphertext,
            nonce: None,
            algorithm: EncryptionAlgorithm::Blake3,
        })
    }
    
    fn decrypt(&self, encrypted: &EncryptedData, key: &CryptoKey) -> Result<Vec<u8>> {
        // Blake3 是对称的，解密与加密相同
        self.encrypt(&encrypted.ciphertext, key)
    }
}

/// 加密器工厂
pub struct EncryptorFactory;

impl EncryptorFactory {
    /// 创建指定算法的加密器
    pub fn create(algorithm: EncryptionAlgorithm) -> Box<dyn Encryptor> {
        match algorithm {
            EncryptionAlgorithm::ChaCha20Poly1305 => Box::new(ChaCha20Poly1305Encryptor),
            EncryptionAlgorithm::Aes256Cbc => Box::new(Aes256CbcEncryptor),
            EncryptionAlgorithm::Blake3 => Box::new(Blake3Encryptor),
        }
    }
}

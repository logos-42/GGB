//! 网络安全和隐私保护模块
//! 
//! 提供IP隐藏、流量混淆、身份保护等功能

use crate::config::SecurityConfig;
use libp2p::PeerId;
use parking_lot::RwLock;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 流量混淆器
pub struct TrafficObfuscator {
    config: SecurityConfig,
    last_rotation: Instant,
    rotation_interval: Duration,
    padding_sizes: Vec<usize>,
}

impl TrafficObfuscator {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            last_rotation: Instant::now(),
            rotation_interval: Duration::from_secs(300), // 每5分钟更换混淆模式
            padding_sizes: vec![64, 128, 256, 512, 1024],
        }
    }

    /// 添加随机填充到数据
    pub fn obfuscate_data(&self, data: &[u8]) -> Vec<u8> {
        if !self.config.hide_ip {
            return data.to_vec();
        }

        let mut rng = rand::thread_rng();
        let padding_size = *self.padding_sizes.choose(&mut rng).unwrap_or(&128);
        
        let mut result = Vec::with_capacity(data.len() + padding_size);
        result.extend_from_slice(data);
        
        // 添加随机填充
        for _ in 0..padding_size {
            result.push(rng.gen());
        }
        
        result
    }

    /// 移除填充数据
    pub fn deobfuscate_data(&self, data: &[u8], original_len: usize) -> Vec<u8> {
        if data.len() < original_len {
            return data.to_vec();
        }
        data[..original_len].to_vec()
    }

    /// 检查是否需要更换混淆模式
    pub fn should_rotate(&self) -> bool {
        self.last_rotation.elapsed() > self.rotation_interval
    }

    /// 更换混淆模式
    pub fn rotate(&mut self) {
        self.last_rotation = Instant::now();
        // 随机打乱填充大小
        let mut rng = rand::thread_rng();
        for i in 0..self.padding_sizes.len() {
            let j = rng.gen_range(i..self.padding_sizes.len());
            self.padding_sizes.swap(i, j);
        }
    }
}

/// 身份保护管理器
pub struct IdentityProtector {
    config: SecurityConfig,
    current_peer_id: RwLock<Option<PeerId>>,
    peer_id_history: RwLock<HashMap<PeerId, Instant>>,
    rotation_interval: Duration,
}

impl IdentityProtector {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            current_peer_id: RwLock::new(None),
            peer_id_history: RwLock::new(HashMap::new()),
            rotation_interval: Duration::from_secs(3600), // 每小时更换一次身份
        }
    }

    /// 生成新的临时PeerId
    pub fn generate_temporary_peer_id(&self) -> PeerId {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        PeerId::from(local_key.public())
    }

    /// 获取当前PeerId，如果需要则生成新的
    pub fn get_current_peer_id(&self) -> PeerId {
        let mut current = self.current_peer_id.write();
        
        if current.is_none() || self.should_rotate_identity() {
            let new_peer_id = self.generate_temporary_peer_id();
            *current = Some(new_peer_id.clone());
            
            // 记录历史
            let mut history = self.peer_id_history.write();
            history.insert(new_peer_id.clone(), Instant::now());
            
            println!("[身份保护] 生成新的临时PeerId: {}", new_peer_id);
            new_peer_id
        } else {
            current.as_ref().unwrap().clone()
        }
    }

    /// 检查是否需要更换身份
    fn should_rotate_identity(&self) -> bool {
        if !self.config.hide_ip {
            return false;
        }

        let current = self.current_peer_id.read();
        if let Some(peer_id) = current.as_ref() {
            let history = self.peer_id_history.read();
            if let Some(created_at) = history.get(peer_id) {
                return created_at.elapsed() > self.rotation_interval;
            }
        }
        true
    }

    /// 清理过期的历史身份
    pub fn cleanup_old_identities(&self) {
        let cutoff = Instant::now() - Duration::from_secs(86400); // 24小时前
        let mut history = self.peer_id_history.write();
        history.retain(|_, &mut timestamp| timestamp > cutoff);
    }

    /// 获取身份历史（用于调试）
    pub fn get_identity_history(&self) -> Vec<(PeerId, Instant)> {
        let history = self.peer_id_history.read();
        history.iter().map(|(k, v)| (k.clone(), *v)).collect()
    }
}

/// 网络隐私检查器
pub struct PrivacyChecker {
    config: SecurityConfig,
}

impl PrivacyChecker {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// 检查地址是否暴露IP
    pub fn is_address_exposing_ip(&self, addr: &libp2p::Multiaddr) -> bool {
        if !self.config.hide_ip {
            return false;
        }

        let addr_str = addr.to_string();
        
        // 检查是否包含IP地址
        if addr_str.contains("/ip4/") || addr_str.contains("/ip6/") {
            // 检查是否为中继地址
            if addr_str.contains("/p2p-circuit/") {
                return false; // 中继地址不暴露真实IP
            }
            return true; // 直接IP地址暴露
        }
        
        false
    }

    /// 验证地址隐私性
    pub fn validate_address_privacy(&self, addr: &libp2p::Multiaddr) -> Result<(), String> {
        if self.config.hide_ip && self.is_address_exposing_ip(addr) {
            return Err(format!("地址 {} 暴露了IP地址，违反隐私设置", addr));
        }
        Ok(())
    }

    /// 获取隐私建议
    pub fn get_privacy_advice(&self) -> Vec<String> {
        let mut advice = Vec::new();
        
        if self.config.hide_ip {
            advice.push("IP隐藏已启用，真实IP不会暴露给公共网络".to_string());
            
            if !self.config.use_relay {
                advice.push("警告：IP隐藏已启用但未使用中继，建议启用中继以获得更好的隐私保护".to_string());
            }
            
            if self.config.enable_dcutr {
                advice.push("注意：DCUtR可能尝试建立直接连接，这可能暴露IP".to_string());
            }
        } else {
            advice.push("IP隐藏未启用，节点IP可能暴露给公共网络".to_string());
            advice.push("建议启用hide_ip和use_relay以保护隐私".to_string());
        }
        
        advice
    }
}

/// 安全工具函数
pub mod utils {
    use super::*;
    
    /// 创建安全的Multiaddr（使用中继）
    pub fn create_secure_address(
        relay_addr: &libp2p::Multiaddr,
        target_peer_id: &PeerId,
    ) -> libp2p::Multiaddr {
        let mut addr = relay_addr.clone();
        addr.push(libp2p::multiaddr::Protocol::P2pCircuit);
        addr.push(libp2p::multiaddr::Protocol::P2p(target_peer_id.clone()));
        addr
    }
    
    /// 检查是否为中继地址
    pub fn is_relay_address(addr: &libp2p::Multiaddr) -> bool {
        addr.to_string().contains("/p2p-circuit/")
    }
    
    /// 从地址中提取目标PeerId（如果是中继地址）
    pub fn extract_target_peer_id(addr: &libp2p::Multiaddr) -> Option<PeerId> {
        let mut protocols = addr.iter();
        let mut found_circuit = false;
        
        while let Some(protocol) = protocols.next() {
            if let libp2p::multiaddr::Protocol::P2pCircuit = protocol {
                found_circuit = true;
            } else if found_circuit {
                if let libp2p::multiaddr::Protocol::P2p(peer_id) = protocol {
                    return Some(peer_id);
                }
            }
        }
        
        None
    }
}

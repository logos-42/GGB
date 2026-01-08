//! 网络安全和隐私保护模块
//!
//! 提供IP隐藏、流量混淆、身份保护等功能

use crate::config::SecurityConfig;
use iroh::NodeId;
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
    current_peer_id: RwLock<Option<NodeId>>,
    peer_id_history: RwLock<HashMap<NodeId, Instant>>,
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

    /// 生成新的临时NodeId
    pub fn generate_temporary_node_id(&self) -> NodeId {
        NodeId::from_bytes(rand::thread_rng().gen::<[u8; 32]>())
    }

    /// 获取当前NodeId，如果需要则生成新的
    pub fn get_current_node_id(&self) -> NodeId {
        let mut current = self.current_peer_id.write();

        if current.is_none() || self.should_rotate_identity() {
            let new_node_id = self.generate_temporary_node_id();
            *current = Some(new_node_id.clone());

            // 记录历史
            let mut history = self.peer_id_history.write();
            history.insert(new_node_id.clone(), Instant::now());

            println!("[身份保护] 生成新的临时NodeId: {}", new_node_id);
            new_node_id
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
        if let Some(node_id) = current.as_ref() {
            let history = self.peer_id_history.read();
            if let Some(created_at) = history.get(node_id) {
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
    pub fn get_identity_history(&self) -> Vec<(NodeId, Instant)> {
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
    pub fn is_address_exposing_ip(&self, addr: &String) -> bool {
        if !self.config.hide_ip {
            return false;
        }

        let addr_str = addr;

        // 检查是否包含IP地址
        if addr_str.contains("/ip4/") || addr_str.contains("/ip6/") {
            return true; // 直接IP地址暴露
        }

        false
    }

    /// 验证地址隐私性
    pub fn validate_address_privacy(&self, addr: &String) -> Result<(), String> {
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

    /// 创建安全的 NodeAddr
    pub fn create_secure_address(
        relay_node_addr: &iroh::NodeAddr,
        target_node_id: &NodeId,
    ) -> iroh::NodeAddr {
        // iroh 使用 relay 节点的方式不同，这里简化实现
        relay_node_addr.clone()
    }

    /// 检查是否为中继地址
    pub fn is_relay_address(addr: &String) -> bool {
        addr.contains("relay") || addr.contains("circuit")
    }

    /// 从地址中提取目标NodeId（如果是中继地址）
    pub fn extract_target_node_id(addr: &String) -> Option<NodeId> {
        // 简化实现，实际需要解析 iroh 的地址格式
        None
    }
}

// ==================== 隐私-性能平衡引擎 ====================

/// 隐私级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyLevel {
    Minimum,    // 最小隐私，最大性能
    Balanced,   // 平衡模式
    Maximum,    // 最大隐私
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub latency_ms: f32,
    pub throughput_mbps: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
}

/// 隐私指标
#[derive(Debug, Clone)]
pub struct PrivacyMetrics {
    pub ip_hidden: bool,
    pub traffic_obfuscated: bool,
    pub metadata_protected: bool,
    pub connection_anonymous: bool,
    pub overall_score: f32,  // 0.0-1.0
}

/// 平衡决策
#[derive(Debug, Clone)]
pub struct BalanceDecision {
    pub privacy_level: PrivacyLevel,
    pub encryption_strength: f32,  // 0.0-1.0
    pub obfuscation_enabled: bool,
    pub routing_strategy: RoutingStrategy,
    pub connection_pool_size: usize,
}

/// 平衡隐私引擎
pub struct BalancedPrivacyEngine {
    config: crate::config::PrivacyPerformanceConfig,
    performance_history: Vec<PerformanceMetrics>,
    privacy_history: Vec<PrivacyMetrics>,
    last_adjustment: Instant,
    adjustment_interval: Duration,
}

impl BalancedPrivacyEngine {
    /// 创建新的平衡隐私引擎
    pub fn new(config: crate::config::PrivacyPerformanceConfig) -> Self {
        Self {
            config,
            performance_history: Vec::new(),
            privacy_history: Vec::new(),
            last_adjustment: Instant::now(),
            adjustment_interval: Duration::from_secs(30),
        }
    }
    
    /// 根据当前条件做出平衡决策
    pub fn make_decision(&mut self, current_performance: &PerformanceMetrics, current_privacy: &PrivacyMetrics) -> BalanceDecision {
        // 记录历史数据
        self.performance_history.push(current_performance.clone());
        self.privacy_history.push(current_privacy.clone());
        
        // 限制历史数据大小
        if self.performance_history.len() > 100 {
            self.performance_history.remove(0);
        }
        if self.privacy_history.len() > 100 {
            self.privacy_history.remove(0);
        }
        
        // 根据配置模式做出决策
        match self.config.mode {
            crate::config::BalanceMode::Performance => self.performance_first_decision(current_performance, current_privacy),
            crate::config::BalanceMode::Privacy => self.privacy_first_decision(current_performance, current_privacy),
            crate::config::BalanceMode::Balanced => self.balanced_decision(current_performance, current_privacy),
            crate::config::BalanceMode::Adaptive => self.adaptive_decision(current_performance, current_privacy),
        }
    }
    
    /// 性能优先决策
    fn performance_first_decision(&self, performance: &PerformanceMetrics, privacy: &PrivacyMetrics) -> BalanceDecision {
        BalanceDecision {
            privacy_level: PrivacyLevel::Minimum,
            encryption_strength: 0.3,  // 较低加密强度
            obfuscation_enabled: false, // 禁用混淆
            routing_strategy: RoutingStrategy::PerformanceFirst,
            connection_pool_size: self.config.connection_pool_size,
        }
    }
    
    /// 隐私优先决策
    fn privacy_first_decision(&self, performance: &PerformanceMetrics, privacy: &PrivacyMetrics) -> BalanceDecision {
        BalanceDecision {
            privacy_level: PrivacyLevel::Maximum,
            encryption_strength: 1.0,  // 最高加密强度
            obfuscation_enabled: true,  // 启用混淆
            routing_strategy: RoutingStrategy::PrivacyFirst,
            connection_pool_size: self.config.connection_pool_size.max(5), // 至少5个连接
        }
    }
    
    /// 平衡决策
    fn balanced_decision(&self, performance: &PerformanceMetrics, privacy: &PrivacyMetrics) -> BalanceDecision {
        let performance_score = self.calculate_performance_score(performance);
        let privacy_score = privacy.overall_score;
        
        // 加权计算平衡点
        let balance_point = performance_score * self.config.performance_weight 
            + privacy_score * (1.0 - self.config.performance_weight);
        
        BalanceDecision {
            privacy_level: if balance_point > 0.7 {
                PrivacyLevel::Balanced
            } else if balance_point > 0.4 {
                PrivacyLevel::Minimum
            } else {
                PrivacyLevel::Maximum
            },
            encryption_strength: balance_point.clamp(0.3, 0.8),
            obfuscation_enabled: balance_point < 0.6,
            routing_strategy: RoutingStrategy::SmartBalance,
            connection_pool_size: self.config.connection_pool_size,
        }
    }
    
    /// 自适应决策
    fn adaptive_decision(&mut self, performance: &PerformanceMetrics, privacy: &PrivacyMetrics) -> BalanceDecision {
        // 检查是否需要调整
        if self.last_adjustment.elapsed() < self.adjustment_interval {
            // 使用上次决策
            return self.balanced_decision(performance, privacy);
        }
        
        self.last_adjustment = Instant::now();
        
        // 分析历史趋势
        let performance_trend = self.analyze_performance_trend();
        let privacy_trend = self.analyze_privacy_trend();
        
        // 根据趋势调整
        let mut decision = self.balanced_decision(performance, privacy);
        
        if performance_trend < -0.1 {
            // 性能下降，降低隐私保护
            decision.encryption_strength = (decision.encryption_strength * 0.8).max(0.3);
            decision.obfuscation_enabled = false;
        } else if privacy_trend < -0.1 {
            // 隐私下降，增强保护
            decision.encryption_strength = (decision.encryption_strength * 1.2).min(1.0);
            decision.obfuscation_enabled = true;
        }
        
        decision
    }
    
    /// 计算性能评分
    fn calculate_performance_score(&self, metrics: &PerformanceMetrics) -> f32 {
        // 简单评分算法，实际应用中需要更复杂的计算
        let latency_score = (100.0 / metrics.latency_ms.max(1.0)).min(1.0);
        let throughput_score = (metrics.throughput_mbps / 100.0).min(1.0);
        let cpu_score = 1.0 - (metrics.cpu_usage_percent / 100.0).min(1.0);
        
        (latency_score * 0.4 + throughput_score * 0.4 + cpu_score * 0.2).clamp(0.0, 1.0)
    }
    
    /// 分析性能趋势
    fn analyze_performance_trend(&self) -> f32 {
        if self.performance_history.len() < 2 {
            return 0.0;
        }
        
        let recent: Vec<_> = self.performance_history.iter().rev().take(5).collect();
        let older: Vec<_> = self.performance_history.iter().rev().skip(5).take(5).collect();
        
        if older.is_empty() {
            return 0.0;
        }
        
        let recent_avg: f32 = recent.iter().map(|m| self.calculate_performance_score(m)).sum::<f32>() / recent.len() as f32;
        let older_avg: f32 = older.iter().map(|m| self.calculate_performance_score(m)).sum::<f32>() / older.len() as f32;
        
        (recent_avg - older_avg) / older_avg.max(0.1)
    }
    
    /// 分析隐私趋势
    fn analyze_privacy_trend(&self) -> f32 {
        if self.privacy_history.len() < 2 {
            return 0.0;
        }
        
        let recent: Vec<_> = self.privacy_history.iter().rev().take(5).collect();
        let older: Vec<_> = self.privacy_history.iter().rev().skip(5).take(5).collect();
        
        if older.is_empty() {
            return 0.0;
        }
        
        let recent_avg: f32 = recent.iter().map(|m| m.overall_score).sum::<f32>() / recent.len() as f32;
        let older_avg: f32 = older.iter().map(|m| m.overall_score).sum::<f32>() / older.len() as f32;
        
        (recent_avg - older_avg) / older_avg.max(0.1)
    }
    
    /// 获取性能历史
    pub fn get_performance_history(&self) -> &[PerformanceMetrics] {
        &self.performance_history
    }
    
    /// 获取隐私历史
    pub fn get_privacy_history(&self) -> &[PrivacyMetrics] {
        &self.privacy_history
    }
    
    /// 重置历史数据
    pub fn reset_history(&mut self) {
        self.performance_history.clear();
        self.privacy_history.clear();
    }
}

/// 隐私-性能监控器
pub struct PrivacyPerformanceMonitor {
    engine: BalancedPrivacyEngine,
    last_metrics: Option<(PerformanceMetrics, PrivacyMetrics)>,
    decision_history: Vec<BalanceDecision>,
}

impl PrivacyPerformanceMonitor {
    /// 创建新的监控器
    pub fn new(config: crate::config::PrivacyPerformanceConfig) -> Self {
        Self {
            engine: BalancedPrivacyEngine::new(config),
            last_metrics: None,
            decision_history: Vec::new(),
        }
    }
    
    /// 更新指标并获取决策
    pub fn update_and_decide(&mut self, performance: PerformanceMetrics, privacy: PrivacyMetrics) -> BalanceDecision {
        self.last_metrics = Some((performance.clone(), privacy.clone()));
        let decision = self.engine.make_decision(&performance, &privacy);
        self.decision_history.push(decision.clone());
        
        // 限制历史大小
        if self.decision_history.len() > 50 {
            self.decision_history.remove(0);
        }
        
        decision
    }
    
    /// 获取当前决策
    pub fn get_current_decision(&self) -> Option<&BalanceDecision> {
        self.decision_history.last()
    }
    
    /// 获取决策历史
    pub fn get_decision_history(&self) -> &[BalanceDecision] {
        &self.decision_history
    }
    
    /// 获取性能趋势
    pub fn get_performance_trend(&self) -> f32 {
        self.engine.analyze_performance_trend()
    }
    
    /// 获取隐私趋势
    pub fn get_privacy_trend(&self) -> f32 {
        self.engine.analyze_privacy_trend()
    }
    
    /// 生成监控报告
    pub fn generate_report(&self) -> MonitorReport {
        let performance_history = self.engine.get_performance_history();
        let privacy_history = self.engine.get_privacy_history();
        
        MonitorReport {
            total_decisions: self.decision_history.len(),
            recent_performance: performance_history.last().cloned(),
            recent_privacy: privacy_history.last().cloned(),
            performance_trend: self.get_performance_trend(),
            privacy_trend: self.get_privacy_trend(),
            current_decision: self.get_current_decision().cloned(),
        }
    }
}

/// 监控报告
#[derive(Debug, Clone)]
pub struct MonitorReport {
    pub total_decisions: usize,
    pub recent_performance: Option<PerformanceMetrics>,
    pub recent_privacy: Option<PrivacyMetrics>,
    pub performance_trend: f32,
    pub privacy_trend: f32,
    pub current_decision: Option<BalanceDecision>,
}

/// 自适应保护选择器
pub struct AdaptiveProtectionSelector {
    monitor: PrivacyPerformanceMonitor,
    protection_levels: HashMap<PrivacyLevel, ProtectionConfig>,
}

impl AdaptiveProtectionSelector {
    /// 创建新的选择器
    pub fn new(config: crate::config::PrivacyPerformanceConfig) -> Self {
        let mut protection_levels = HashMap::new();
        
        // 定义不同隐私级别的保护配置
        protection_levels.insert(PrivacyLevel::Minimum, ProtectionConfig {
            encryption_algorithm: "chacha20".to_string(),
            key_size: 128,
            enable_obfuscation: false,
            padding_size: 0,
            connection_rotation_interval: Duration::from_secs(3600),
        });
        
        protection_levels.insert(PrivacyLevel::Balanced, ProtectionConfig {
            encryption_algorithm: "aes-256-gcm".to_string(),
            key_size: 256,
            enable_obfuscation: true,
            padding_size: 64,
            connection_rotation_interval: Duration::from_secs(1800),
        });
        
        protection_levels.insert(PrivacyLevel::Maximum, ProtectionConfig {
            encryption_algorithm: "aes-256-gcm".to_string(),
            key_size: 256,
            enable_obfuscation: true,
            padding_size: 128,
            connection_rotation_interval: Duration::from_secs(900),
        });
        
        Self {
            monitor: PrivacyPerformanceMonitor::new(config),
            protection_levels,
        }
    }
    
    /// 根据当前条件选择保护配置
    pub fn select_protection(&mut self, performance: PerformanceMetrics, privacy: PrivacyMetrics) -> ProtectionConfig {
        let decision = self.monitor.update_and_decide(performance, privacy);
        
        // 获取对应隐私级别的保护配置
        self.protection_levels.get(&decision.privacy_level)
            .cloned()
            .unwrap_or_else(|| self.protection_levels[&PrivacyLevel::Balanced].clone())
    }
    
    /// 获取监控报告
    pub fn get_report(&self) -> MonitorReport {
        self.monitor.generate_report()
    }
}

/// 保护配置
#[derive(Debug, Clone)]
pub struct ProtectionConfig {
    pub encryption_algorithm: String,
    pub key_size: u32,
    pub enable_obfuscation: bool,
    pub padding_size: usize,
    pub connection_rotation_interval: Duration,
}
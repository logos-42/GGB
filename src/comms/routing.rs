//! 智能路由系统模块
//! 
//! 提供智能路由选择、性能监控和隐私分析功能

use std::collections::VecDeque;
use std::time::Instant;

/// 路由类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteType {
    Direct,      // 直接连接
    Relay,       // 中继连接
    Proxy,       // 代理连接
    Tor,         // Tor网络
    Mixed,       // 混合路由
}

/// 路由质量指标
#[derive(Debug, Clone)]
pub struct RouteQuality {
    pub latency_ms: f32,
    pub bandwidth_mbps: f32,
    pub packet_loss_percent: f32,
    pub jitter_ms: f32,
    pub reliability: f32,  // 0.0-1.0
    pub last_updated: Instant,
}

/// 路由评分
#[derive(Debug, Clone)]
pub struct RouteScore {
    pub performance_score: f32,  // 0.0-1.0
    pub privacy_score: f32,      // 0.0-1.0
    pub cost_score: f32,         // 0.0-1.0（成本/资源消耗）
    pub overall_score: f32,      // 加权总分
}

/// 路由信息
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub route_type: RouteType,
    pub target: String,
    pub next_hop: Option<String>,
    pub quality: RouteQuality,
    pub score: RouteScore,
    pub established_at: Instant,
    pub usage_count: u64,
}

/// 路由选择记录
#[derive(Debug, Clone)]
pub struct RouteSelection {
    pub timestamp: Instant,
    pub target: String,
    pub selected_route: RouteInfo,
    pub alternative_routes: Vec<RouteInfo>,
    pub selection_reason: String,
}

/// 路由错误类型
#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("无可用路由")]
    NoRoutesAvailable,
    
    #[error("路由发现失败: {0}")]
    DiscoveryFailed(String),
    
    #[error("路由质量过低")]
    PoorRouteQuality,
    
    #[error("隐私要求不满足")]
    PrivacyRequirementNotMet,
    
    #[error("性能要求不满足")]
    PerformanceRequirementNotMet,
    
    #[error("路由建立失败: {0}")]
    RouteEstablishmentFailed(String),
}

/// 性能监控器
pub(crate) struct PerformanceMonitor {
    latency_history: VecDeque<f32>,
    bandwidth_history: VecDeque<f32>,
    packet_loss_history: VecDeque<f32>,
}

impl PerformanceMonitor {
    pub(crate) fn new() -> Self {
        Self {
            latency_history: VecDeque::with_capacity(100),
            bandwidth_history: VecDeque::with_capacity(100),
            packet_loss_history: VecDeque::with_capacity(100),
        }
    }
    
    pub(crate) fn update(&mut self, latency: f32, bandwidth: f32, packet_loss: f32) {
        self.latency_history.push_back(latency);
        self.bandwidth_history.push_back(bandwidth);
        self.packet_loss_history.push_back(packet_loss);
        
        // 保持历史数据大小
        if self.latency_history.len() > 100 {
            self.latency_history.pop_front();
        }
        if self.bandwidth_history.len() > 100 {
            self.bandwidth_history.pop_front();
        }
        if self.packet_loss_history.len() > 100 {
            self.packet_loss_history.pop_front();
        }
    }
    
    pub(crate) fn get_average_latency(&self) -> f32 {
        if self.latency_history.is_empty() {
            return 50.0; // 默认值
        }
        self.latency_history.iter().sum::<f32>() / self.latency_history.len() as f32
    }
    
    pub(crate) fn get_average_bandwidth(&self) -> f32 {
        if self.bandwidth_history.is_empty() {
            return 10.0; // 默认10Mbps
        }
        self.bandwidth_history.iter().sum::<f32>() / self.bandwidth_history.len() as f32
    }
}

/// 隐私分析器
pub(crate) struct PrivacyAnalyzer {
    ip_exposure_history: VecDeque<bool>,
    traffic_analysis_resistance: VecDeque<f32>,
}

impl PrivacyAnalyzer {
    pub(crate) fn new() -> Self {
        Self {
            ip_exposure_history: VecDeque::with_capacity(100),
            traffic_analysis_resistance: VecDeque::with_capacity(100),
        }
    }
    
    pub(crate) fn analyze_route_privacy(&self, route_type: RouteType) -> f32 {
        match route_type {
            RouteType::Direct => 0.3,    // 直接连接隐私性低
            RouteType::Relay => 0.7,     // 中继连接中等隐私
            RouteType::Proxy => 0.8,     // 代理连接高隐私
            RouteType::Tor => 0.95,      // Tor网络极高隐私
            RouteType::Mixed => 0.6,     // 混合路由中等隐私
        }
    }
    
    pub(crate) fn update_analysis(&mut self, ip_exposed: bool, resistance: f32) {
        self.ip_exposure_history.push_back(ip_exposed);
        self.traffic_analysis_resistance.push_back(resistance);
        
        if self.ip_exposure_history.len() > 100 {
            self.ip_exposure_history.pop_front();
        }
        if self.traffic_analysis_resistance.len() > 100 {
            self.traffic_analysis_resistance.pop_front();
        }
    }
}

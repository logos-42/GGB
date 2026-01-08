//! 网络延迟测量模块
//!
//! 提供基于Iroh的网络延迟测量功能，用于估算节点间的地理距离

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

use iroh::{Endpoint, net::{NodeAddr, NodeId}, endpoint::Connection, net_report::Report};

/// 延迟测量结果
#[derive(Debug, Clone)]
pub struct LatencyMeasurement {
    /// 往返时间（毫秒）
    pub rtt_ms: f64,
    /// 连接状态
    pub connection_status: ConnectionStatus,
    /// 测量时间戳
    pub timestamp: std::time::Instant,
}

/// 连接状态
#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    /// 直接连接
    Direct,
    /// 通过中继连接
    Relay(String), // Relay URL
    /// 未知状态
    Unknown,
}

/// 网络延迟探测器
pub struct NetworkLatencyDetector {
    /// Iroh端点
    endpoint: Endpoint,
    /// 延迟缓存
    latency_cache: std::sync::Arc<parking_lot::RwLock<HashMap<String, LatencyMeasurement>>>,
    /// 默认超时时间
    timeout_duration: Duration,
}

impl NetworkLatencyDetector {
    /// 创建新的网络延迟探测器
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            latency_cache: std::sync::Arc::new(parking_lot::RwLock::new(HashMap::new())),
            timeout_duration: Duration::from_secs(5),
        }
    }

    /// 设置超时时间
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// 测量到指定节点的延迟
    pub async fn measure_latency(&self, node_addr: &NodeAddr) -> Option<LatencyMeasurement> {
        let start = std::time::Instant::now();

        // 连接到节点
        let connect_result = timeout(self.timeout_duration, 
            self.endpoint.connect(node_addr.clone(), b"ggb-latency-probe")
        ).await;

        match connect_result {
            Ok(conn_result) => {
                match conn_result {
                    Ok(connection) => {
                        // 发送一个小数据包并测量响应时间
                        let ping_result = self.ping_connection(&connection).await;
                        
                        let rtt = start.elapsed().as_millis() as f64;
                        
                        // 确定连接状态
                        let connection_status = self.determine_connection_status(&connection).await;
                        
                        let measurement = LatencyMeasurement {
                            rtt_ms: rtt,
                            connection_status,
                            timestamp: std::time::Instant::now(),
                        };

                        // 缓存测量结果
                        {
                            let mut cache = self.latency_cache.write();
                            cache.insert(node_addr.node_id.to_string(), measurement.clone());
                        }

                        Some(measurement)
                    }
                    Err(e) => {
                        eprintln!("连接失败: {:?}", e);
                        None
                    }
                }
            Err(_) => {
                eprintln!("连接超时");
                None
            }
        }
    }

    /// 对连接进行ping测试
    async fn ping_connection(&self, connection: &Connection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 创建一个单向流并发送ping数据
        let mut send_stream = connection.open_uni().await?;
        
        // 发送ping消息
        let ping_msg = b"PING";
        send_stream.write_all(ping_msg).await?;
        send_stream.finish().await?;
        
        Ok(())
    }

    /// 确定连接状态（直接连接还是通过中继）
    async fn determine_connection_status(&self, connection: &Connection) -> ConnectionStatus {
        // 尝试获取连接的详细信息
        // 注意：这依赖于Iroh的具体API，可能需要根据实际版本调整
        match connection.remote_addr() {
            iroh::endpoint::RemoteAddr::Relay(url) => ConnectionStatus::Relay(url.to_string()),
            iroh::endpoint::RemoteAddr::Direct(_) => ConnectionStatus::Direct,
        }
    }

    /// 获取缓存的延迟测量结果
    pub fn get_cached_latency(&self, node_id: &str) -> Option<LatencyMeasurement> {
        let cache = self.latency_cache.read();
        cache.get(node_id).cloned()
    }

    /// 清除过期的缓存条目
    pub fn cleanup_expired_cache(&self, max_age: Duration) {
        let mut cache = self.latency_cache.write();
        let now = std::time::Instant::now();
        cache.retain(|_, measurement| {
            now.duration_since(measurement.timestamp) <= max_age
        });
    }

    /// 获取当前网络报告（包含DERP节点延迟信息）
    pub async fn get_net_report(&self) -> Option<Report> {
        self.endpoint.net_report().initialized().await
    }

    /// 根据RTT估算物理距离
    /// 
    /// 基于光在光纤中的传播速度（约200,000 km/s）进行估算
    pub fn estimate_distance_km(&self, rtt_ms: f64) -> f64 {
        // 光在光纤中的速度约为真空中光速的2/3，即约200,000 km/s
        // RTT是往返时间，所以我们需要除以2得到单程距离
        let speed_km_per_ms = 200_000.0 / 1000.0; // 200 km/ms
        let one_way_time_ms = rtt_ms / 2.0;
        
        // 计算距离（考虑网络开销，实际距离会比理论值小）
        let estimated_distance = one_way_time_ms * speed_km_per_ms;
        
        // 应用一些修正因子，因为实际网络中还有路由器处理时间等
        estimated_distance * 0.7 // 网络开销修正因子
    }

    /// 根据RTT返回距离级别（模糊距离）
    pub fn distance_level_from_rtt(&self, rtt_ms: f64) -> crate::types::DistanceLevel {
        // 基于RTT的模糊距离分类
        if rtt_ms <= 20.0 {
            crate::types::DistanceLevel::VeryClose // 非常近（<20ms）- 可能在同一城市
        } else if rtt_ms <= 100.0 {
            crate::types::DistanceLevel::Close      // 近（21-100ms）- 同一国家
        } else if rtt_ms <= 300.0 {
            crate::types::DistanceLevel::Medium     // 中等（101-300ms）- 跨洲
        } else {
            crate::types::DistanceLevel::Far        // 远（>300ms）- 全球范围
        }
    }
}

/// 基于延迟的距离计算器
pub struct DistanceCalculator {
    /// 本地延迟探测器
    detector: NetworkLatencyDetector,
}

impl DistanceCalculator {
    /// 创建新的距离计算器
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            detector: NetworkLatencyDetector::new(endpoint),
        }
    }

    /// 获取到目标节点的网络距离信息
    pub async fn get_network_distance(&self, node_addr: &NodeAddr) -> crate::types::NetworkDistance {
        match self.detector.measure_latency(node_addr).await {
            Some(measurement) => {
                // 创建网络距离对象
                let mut relay_delays = Vec::new();
                
                // 如果是通过中继连接，添加中继延迟
                if let ConnectionStatus::Relay(relay_url) = measurement.connection_status {
                    relay_delays.push((relay_url, measurement.rtt_ms as u64));
                }
                
                crate::types::NetworkDistance {
                    relay_delays,
                    end_to_end_delay: Some(measurement.rtt_ms as u64),
                }
            }
            None => {
                // 如果无法测量，返回空的网络距离对象
                crate::types::NetworkDistance::new()
            }
        }
    }

    /// 估算两个节点之间的相对距离
    pub fn estimate_relative_distance(
        &self, 
        local_latencies: &crate::types::NetworkDistance,
        remote_latencies: &crate::types::NetworkDistance
    ) -> f32 {
        // 基于共同中继节点的延迟相似性计算相对距离
        local_latencies.similarity_to(remote_latencies)
    }
    
    /// 获取本地网络的DERP节点延迟信息
    pub async fn get_local_derp_delays(&self) -> Vec<(String, u64)> {
        let mut delays = Vec::new();
        
        // 获取网络报告
        if let Some(report) = self.detector.get_net_report().await {
            // 遍历所有DERP节点及其延迟
            for (relay_url, latency) in report.relay_latency.iter() {
                // 将Duration转换为毫秒
                let latency_ms = latency.as_millis() as u64;
                delays.push((relay_url.to_string(), latency_ms));
            }
        }
        
        delays
    }
}

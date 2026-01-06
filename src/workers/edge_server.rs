//! 边缘服务器Worker实现
//! 
//! 在Cloudflare Workers上运行的边缘服务器功能

use crate::workers::{
    NodeInfo, Heartbeat, TaskRequest, MatchRequest, NodeStatus, MatchingStrategy,
};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 边缘服务器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EdgeServerConfig {
    /// 服务器名称
    pub name: String,
    /// 最大节点数
    pub max_nodes: usize,
    /// 心跳超时时间（秒）
    pub heartbeat_timeout_secs: u64,
    /// 节点匹配策略
    pub default_matching_strategy: MatchingStrategy,
    /// 是否启用地理匹配
    pub enable_geo_matching: bool,
    /// 是否启用性能匹配
    pub enable_performance_matching: bool,
}

/// 节点注册信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeRegistration {
    pub node_id: String,
    pub registration_time: i64,
    pub assigned_tasks: usize,
    pub completed_tasks: usize,
    pub success_rate: f64,
}

/// 任务响应
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub status: TaskStatus,
    pub assigned_nodes: Vec<String>,
    pub estimated_completion_time: i64,
    pub proof_required: bool,
}

/// 匹配的节点
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchedNode {
    pub node_id: String,
    pub score: f64,
    pub capabilities: serde_json::Value,
    pub estimated_completion_time_ms: u64,
    pub network_latency_ms: u64,
}

/// 边缘服务器Worker
pub struct EdgeServerWorker {
    config: EdgeServerConfig,
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    registrations: Arc<RwLock<HashMap<String, NodeRegistration>>>,
    heartbeats: Arc<RwLock<HashMap<String, i64>>>,
    tasks: Arc<RwLock<HashMap<String, TaskRequest>>>,
}

impl EdgeServerWorker {
    /// 创建新的边缘服务器Worker
    pub fn new(config: EdgeServerConfig) -> Result<Self> {
        Ok(Self {
            config,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            registrations: Arc::new(RwLock::new(HashMap::new())),
            heartbeats: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 注册节点
    pub async fn register_node(&self, node_info: NodeInfo) -> Result<NodeRegistration> {
        let node_id = node_info.node_id.clone();
        let now = chrono::Utc::now().timestamp();
        
        let registration = NodeRegistration {
            node_id: node_id.clone(),
            registration_time: now,
            assigned_tasks: 0,
            completed_tasks: 0,
            success_rate: 1.0,
        };
        
        // 存储节点信息
        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(node_id.clone(), node_info);
        }
        
        // 存储注册信息
        {
            let mut registrations = self.registrations.write().await;
            registrations.insert(node_id.clone(), registration.clone());
        }
        
        // 更新心跳
        {
            let mut heartbeats = self.heartbeats.write().await;
            heartbeats.insert(node_id, now);
        }
        
        Ok(registration)
    }
    
    /// 更新心跳
    pub async fn update_heartbeat(&self, heartbeat: Heartbeat) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        
        let mut heartbeats = self.heartbeats.write().await;
        heartbeats.insert(heartbeat.node_id, now);
        
        Ok(())
    }
    
    /// 提交任务
    pub async fn submit_task(&self, task_request: TaskRequest) -> Result<TaskResponse> {
        let task_id = task_request.task_id.clone();
        let now = chrono::Utc::now().timestamp();
        
        // 存储任务
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task_request.clone());
        }
        
        // 创建任务响应
        let response = TaskResponse {
            task_id: task_id.clone(),
            status: TaskStatus::Pending,
            assigned_nodes: Vec::new(),
            estimated_completion_time: now + 300, // 默认5分钟后
            proof_required: true,
        };
        
        Ok(response)
    }
    
    /// 匹配节点
    pub async fn match_nodes(&self, match_request: MatchRequest) -> Result<Vec<MatchedNode>> {
        let nodes = self.nodes.read().await;
        let registrations = self.registrations.read().await;
        let heartbeats = self.heartbeats.read().await;
        
        let now = chrono::Utc::now().timestamp();
        let timeout = self.config.heartbeat_timeout_secs as i64;
        
        let mut available_nodes = Vec::new();
        
        // 筛选可用节点
        for (node_id, node_info) in nodes.iter() {
            // 检查节点是否可用
            if !node_info.available {
                continue;
            }
            
            // 检查心跳是否超时
            if let Some(last_heartbeat) = heartbeats.get(node_id) {
                if now - *last_heartbeat > timeout {
                    continue;
                }
            } else {
                continue;
            }
            
            // 获取注册信息
            let registration = registrations.get(node_id);
            
            // 计算节点评分
            let score = self.calculate_node_score(
                node_info,
                registration,
                &match_request,
            );
            
            if score > 0.0 {
                available_nodes.push(MatchedNode {
                    node_id: node_id.clone(),
                    score,
                    capabilities: serde_json::Value::Null, // 简化处理
                    estimated_completion_time_ms: 1000, // 默认1秒
                    network_latency_ms: 50, // 默认50ms
                });
            }
        }
        
        // 根据评分排序
        available_nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // 限制返回数量
        let max_nodes = 10.min(available_nodes.len());
        Ok(available_nodes[..max_nodes].to_vec())
    }
    
    /// 计算节点评分
    fn calculate_node_score(
        &self,
        node_info: &NodeInfo,
        registration: Option<&NodeRegistration>,
        match_request: &MatchRequest,
    ) -> f64 {
        let mut score = 0.0;
        
        // 基础可用性分数
        if node_info.available {
            score += 0.3;
        }
        
        // 注册信息分数
        if let Some(reg) = registration {
            // 成功率分数
            score += reg.success_rate * 0.3;
            
            // 任务完成数分数（对数缩放，防止过大）
            let task_score = (reg.completed_tasks as f64 + 1.0).ln() / 10.0;
            score += task_score.min(0.2);
        }
        
        // 根据匹配策略调整分数
        match match_request.strategy {
            MatchingStrategy::Performance => {
                // 性能匹配：考虑计算能力
                // 这里简化处理，实际需要从capabilities中提取
                score += 0.2;
            }
            MatchingStrategy::Geography => {
                // 地理匹配：考虑地理位置
                if node_info.location.is_some() {
                    score += 0.2;
                }
            }
            MatchingStrategy::LoadBalance => {
                // 负载均衡：考虑当前负载
                if let Some(reg) = registration {
                    let load_score = 1.0 - (reg.assigned_tasks as f64 / 100.0).min(1.0);
                    score += load_score * 0.2;
                }
            }
            MatchingStrategy::Hybrid => {
                // 混合策略：综合考量
                score += 0.1;
                if node_info.location.is_some() {
                    score += 0.05;
                }
                if let Some(reg) = registration {
                    let load_score = 1.0 - (reg.assigned_tasks as f64 / 50.0).min(1.0);
                    score += load_score * 0.05;
                }
            }
        }
        
        score.min(1.0)
    }
    
    /// 清理过期节点
    pub async fn cleanup_expired_nodes(&self) -> Result<usize> {
        let now = chrono::Utc::now().timestamp();
        let timeout = self.config.heartbeat_timeout_secs as i64;
        
        let heartbeats = self.heartbeats.read().await;
        let mut expired_nodes = Vec::new();
        
        // 找出过期节点
        for (node_id, last_heartbeat) in heartbeats.iter() {
            if now - *last_heartbeat > timeout {
                expired_nodes.push(node_id.clone());
            }
        }
        
        // 清理过期节点
        if !expired_nodes.is_empty() {
            let mut nodes = self.nodes.write().await;
            let mut registrations = self.registrations.write().await;
            let mut heartbeats = self.heartbeats.write().await;
            
            for node_id in &expired_nodes {
                nodes.remove(node_id);
                registrations.remove(node_id);
                heartbeats.remove(node_id);
            }
        }
        
        Ok(expired_nodes.len())
    }
    
    /// 获取节点统计信息
    pub async fn get_node_stats(&self) -> NodeStats {
        let nodes = self.nodes.read().await;
        let registrations = self.registrations.read().await;
        let heartbeats = self.heartbeats.read().await;
        
        let now = chrono::Utc::now().timestamp();
        let timeout = self.config.heartbeat_timeout_secs as i64;
        
        let mut total_nodes = 0;
        let mut active_nodes = 0;
        let mut total_tasks = 0;
        let mut avg_success_rate = 0.0;
        
        for (node_id, _) in nodes.iter() {
            total_nodes += 1;
            
            // 检查是否活跃
            if let Some(last_heartbeat) = heartbeats.get(node_id) {
                if now - *last_heartbeat <= timeout {
                    active_nodes += 1;
                }
            }
            
            // 统计任务信息
            if let Some(registration) = registrations.get(node_id) {
                total_tasks += registration.completed_tasks;
                avg_success_rate += registration.success_rate;
            }
        }
        
        if total_nodes > 0 {
            avg_success_rate /= total_nodes as f64;
        }
        
        NodeStats {
            total_nodes,
            active_nodes,
            total_tasks,
            avg_success_rate,
            timestamp: now,
        }
    }
}

/// 任务状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 节点统计信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeStats {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_tasks: usize,
    pub avg_success_rate: f64,
    pub timestamp: i64,
}

/// WASM兼容的接口
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    
    #[wasm_bindgen]
    pub struct WasmEdgeServer {
        server: EdgeServerWorker,
    }
    
    #[wasm_bindgen]
    impl WasmEdgeServer {
        #[wasm_bindgen(constructor)]
        pub fn new(config_js: JsValue) -> Result<WasmEdgeServer, JsValue> {
            let config: EdgeServerConfig = serde_wasm_bindgen::from_value(config_js)
                .map_err(|e| JsValue::from_str(&format!("配置解析失败: {}", e)))?;
            
            let server = EdgeServerWorker::new(config)
                .map_err(|e| JsValue::from_str(&format!("服务器创建失败: {}", e)))?;
            
            Ok(WasmEdgeServer { server })
        }
        
        #[wasm_bindgen]
        pub async fn register_node(&self, node_info_js: JsValue) -> Result<JsValue, JsValue> {
            let node_info: NodeInfo = serde_wasm_bindgen::from_value(node_info_js)
                .map_err(|e| JsValue::from_str(&format!("节点信息解析失败: {}", e)))?;
            
            let registration = self.server.register_node(node_info).await
                .map_err(|e| JsValue::from_str(&format!("节点注册失败: {}", e)))?;
            
            serde_wasm_bindgen::to_value(&registration)
                .map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
        
        #[wasm_bindgen]
        pub async fn match_nodes(&self, match_request_js: JsValue) -> Result<JsValue, JsValue> {
            let match_request: MatchRequest = serde_wasm_bindgen::from_value(match_request_js)
                .map_err(|e| JsValue::from_str(&format!("匹配请求解析失败: {}", e)))?;
            
            let matched_nodes = self.server.match_nodes(match_request).await
                .map_err(|e| JsValue::from_str(&format!("节点匹配失败: {}", e)))?;
            
            serde_wasm_bindgen::to_value(&matched_nodes)
                .map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
        
        #[wasm_bindgen]
        pub async fn get_stats(&self) -> Result<JsValue, JsValue> {
            let stats = self.server.get_node_stats().await;
            
            serde_wasm_bindgen::to_value(&stats)
                .map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
    }
}

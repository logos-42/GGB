//! Workers算法管理器
//! 
//! 在Cloudflare Workers上运行的智能算法调度

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 算法请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmRequest {
    /// 任务ID
    pub task_id: String,
    /// 任务类型
    pub task_type: AlgorithmTaskType,
    /// 可用节点列表
    pub available_nodes: Vec<NodeInfo>,
    /// 任务需求
    pub requirements: TaskRequirements,
    /// 算法类型
    pub algorithm_type: AlgorithmType,
    /// 算法参数
    pub parameters: serde_json::Value,
}

/// 算法响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmResponse {
    /// 任务ID
    pub task_id: String,
    /// 分配结果
    pub allocation: TaskAllocation,
    /// 算法执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 算法评分
    pub score: f64,
}

/// 任务分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAllocation {
    /// 分配的节点
    pub assigned_nodes: Vec<AssignedNode>,
    /// 子任务分配
    pub subtasks: Vec<Subtask>,
    /// 预计完成时间（毫秒）
    pub estimated_completion_time_ms: u64,
    /// 总成本评分
    pub total_cost: f64,
}

/// Worker算法管理器
pub struct WorkerAlgorithmManager {
    config: WorkerAlgorithmManagerConfig,
}

impl WorkerAlgorithmManager {
    /// 创建新的算法管理器
    pub fn new(config: WorkerAlgorithmManagerConfig) -> Self {
        Self { config }
    }
    
    /// 分配任务
    pub async fn allocate(&self, request: AlgorithmRequest) -> Result<AlgorithmResponse> {
        let start_time = std::time::Instant::now();
        
        // 根据算法类型选择分配策略
        let allocation = match request.algorithm_type {
            AlgorithmType::ParticleSwarm => {
                self.allocate_with_pso(&request).await?
            }
            AlgorithmType::GeneticAlgorithm => {
                self.allocate_with_ga(&request).await?
            }
            AlgorithmType::AntColony => {
                self.allocate_with_aco(&request).await?
            }
            AlgorithmType::Hybrid => {
                self.allocate_with_hybrid(&request).await?
            }
            AlgorithmType::Random => {
                self.allocate_random(&request).await?
            }
        };
        
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(AlgorithmResponse {
            task_id: request.task_id,
            allocation,
            execution_time_ms,
            score: 0.8, // 简化评分
        })
    }
    
    /// 使用粒子群算法分配
    async fn allocate_with_pso(&self, request: &AlgorithmRequest) -> Result<TaskAllocation> {
        // 简化实现：随机选择节点
        let mut assigned_nodes = Vec::new();
        let mut subtasks = Vec::new();
        
        let node_count = request.available_nodes.len().min(3);
        for i in 0..node_count {
            if let Some(node) = request.available_nodes.get(i) {
                assigned_nodes.push(AssignedNode {
                    node_id: node.node_id.clone(),
                    capability_score: 0.7,
                    network_score: 0.8,
                    cost: 1.0,
                });
                
                subtasks.push(Subtask {
                    subtask_id: format!("{}_{}", request.task_id, i),
                    node_id: node.node_id.clone(),
                    data_size: 1024,
                    estimated_time_ms: 1000,
                });
            }
        }
        
        Ok(TaskAllocation {
            assigned_nodes,
            subtasks,
            estimated_completion_time_ms: 5000,
            total_cost: node_count as f64,
        })
    }
    
    /// 使用遗传算法分配
    async fn allocate_with_ga(&self, request: &AlgorithmRequest) -> Result<TaskAllocation> {
        // 简化实现：选择前几个节点
        let mut assigned_nodes = Vec::new();
        let mut subtasks = Vec::new();
        
        let node_count = request.available_nodes.len().min(2);
        for i in 0..node_count {
            if let Some(node) = request.available_nodes.get(i) {
                assigned_nodes.push(AssignedNode {
                    node_id: node.node_id.clone(),
                    capability_score: 0.8,
                    network_score: 0.7,
                    cost: 0.9,
                });
                
                subtasks.push(Subtask {
                    subtask_id: format!("{}_{}", request.task_id, i),
                    node_id: node.node_id.clone(),
                    data_size: 2048,
                    estimated_time_ms: 2000,
                });
            }
        }
        
        Ok(TaskAllocation {
            assigned_nodes,
            subtasks,
            estimated_completion_time_ms: 4000,
            total_cost: node_count as f64 * 0.9,
        })
    }
    
    /// 使用蚁群算法分配
    async fn allocate_with_aco(&self, request: &AlgorithmRequest) -> Result<TaskAllocation> {
        // 简化实现：考虑地理位置的分配
        let mut assigned_nodes = Vec::new();
        let mut subtasks = Vec::new();
        
        // 假设有地理位置信息
        let has_geo = request.available_nodes.iter()
            .any(|n| n.location.is_some());
        
        let node_count = if has_geo { 1 } else { 2 };
        
        for i in 0..node_count.min(request.available_nodes.len()) {
            if let Some(node) = request.available_nodes.get(i) {
                let geo_score = if node.location.is_some() { 0.9 } else { 0.5 };
                
                assigned_nodes.push(AssignedNode {
                    node_id: node.node_id.clone(),
                    capability_score: 0.6,
                    network_score: geo_score,
                    cost: 1.1,
                });
                
                subtasks.push(Subtask {
                    subtask_id: format!("{}_{}", request.task_id, i),
                    node_id: node.node_id.clone(),
                    data_size: 512,
                    estimated_time_ms: 1500,
                });
            }
        }
        
        Ok(TaskAllocation {
            assigned_nodes,
            subtasks,
            estimated_completion_time_ms: 3000,
            total_cost: node_count as f64 * 1.1,
        })
    }
    
    /// 使用混合算法分配
    async fn allocate_with_hybrid(&self, request: &AlgorithmRequest) -> Result<TaskAllocation> {
        // 简化实现：结合多种策略
        let pso_result = self.allocate_with_pso(request).await?;
        let ga_result = self.allocate_with_ga(request).await?;
        
        // 选择成本较低的结果
        let allocation = if pso_result.total_cost <= ga_result.total_cost {
            pso_result
        } else {
            ga_result
        };
        
        Ok(allocation)
    }
    
    /// 随机分配
    async fn allocate_random(&self, request: &AlgorithmRequest) -> Result<TaskAllocation> {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let node_count = rng.random_range(1..=request.available_nodes.len().min(3));

        let mut assigned_nodes = Vec::new();
        let mut subtasks = Vec::new();

        for i in 0..node_count {
            if let Some(node) = request.available_nodes.get(i) {
                let capability_score = rng.random_range(0.5..0.9);
                let network_score = rng.random_range(0.5..0.9);
                let cost = rng.random_range(0.8..1.2);

                assigned_nodes.push(AssignedNode {
                    node_id: node.node_id.clone(),
                    capability_score,
                    network_score,
                    cost,
                });

                subtasks.push(Subtask {
                    subtask_id: format!("{}_{}", request.task_id, i),
                    node_id: node.node_id.clone(),
                    data_size: rng.random_range(512..4096),
                    estimated_time_ms: rng.random_range(500..3000),
                });
            }
        }

        Ok(TaskAllocation {
            assigned_nodes,
            subtasks,
            estimated_completion_time_ms: rng.random_range(2000..8000),
            total_cost: node_count as f64 * rng.random_range(0.8..1.2),
        })
    }
}

// 类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub capabilities: serde_json::Value,
    pub location: Option<serde_json::Value>,
    pub network_info: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignedNode {
    pub node_id: String,
    pub capability_score: f64,
    pub network_score: f64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub subtask_id: String,
    pub node_id: String,
    pub data_size: u64,
    pub estimated_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmTaskType {
    Training,
    Inference,
    Validation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmType {
    ParticleSwarm,
    GeneticAlgorithm,
    AntColony,
    Hybrid,
    Random,
}

pub type TaskRequirements = serde_json::Value;
pub type WorkerAlgorithmManagerConfig = serde_json::Value;

/// WASM兼容的接口
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    
    #[wasm_bindgen]
    pub struct WasmAlgorithmManager {
        manager: WorkerAlgorithmManager,
    }
    
    #[wasm_bindgen]
    impl WasmAlgorithmManager {
        #[wasm_bindgen(constructor)]
        pub fn new(config_js: JsValue) -> Result<WasmAlgorithmManager, JsValue> {
            let config: WorkerAlgorithmManagerConfig = serde_wasm_bindgen::from_value(config_js)
                .map_err(|e| JsValue::from_str(&format!("配置解析失败: {}", e)))?;
            
            let manager = WorkerAlgorithmManager::new(config);
            Ok(WasmAlgorithmManager { manager })
        }
        
        #[wasm_bindgen]
        pub async fn allocate(&self, request_js: JsValue) -> Result<JsValue, JsValue> {
            let request: AlgorithmRequest = serde_wasm_bindgen::from_value(request_js)
                .map_err(|e| JsValue::from_str(&format!("请求解析失败: {}", e)))?;
            
            let response = self.manager.allocate(request).await
                .map_err(|e| JsValue::from_str(&format!("算法分配失败: {}", e)))?;
            
            serde_wasm_bindgen::to_value(&response)
                .map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
    }
}

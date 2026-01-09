//! 智能算法模块
//! 
//! 提供粒子群算法、遗传算法、蚁群算法等智能调度算法

pub mod pso;
pub mod ga;
pub mod aco;
pub mod hybrid;

// 重新导出常用类型
pub use pso::{ParticleSwarmOptimizer, PSOConfig, Particle, Swarm};
pub use ga::{GeneticAlgorithm, GAConfig, Chromosome, Population};
pub use aco::{AntColonyOptimizer, ACOConfig, Ant, Pheromone};
pub use hybrid::{HybridOptimizer, HybridConfig};

/// 算法配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlgorithmConfig {
    /// 粒子群算法配置
    pub pso: PSOConfig,
    /// 遗传算法配置
    pub ga: GAConfig,
    /// 蚁群算法配置
    pub aco: ACOConfig,
    /// 混合算法配置
    pub hybrid: HybridConfig,
    /// 默认算法类型
    pub default_algorithm: AlgorithmType,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 收敛阈值
    pub convergence_threshold: f64,
    /// 是否启用并行计算
    pub enable_parallel: bool,
}

/// 算法类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AlgorithmType {
    /// 粒子群算法
    ParticleSwarm,
    /// 遗传算法
    Genetic,
    /// 蚁群算法
    AntColony,
    /// 混合算法
    Hybrid,
}

/// 算法管理器
pub struct AlgorithmManager {
    config: AlgorithmConfig,
    pso: ParticleSwarmOptimizer,
    ga: GeneticAlgorithm,
    aco: AntColonyOptimizer,
    hybrid: HybridOptimizer,
}

impl AlgorithmManager {
    /// 创建新的算法管理器
    pub fn new(config: AlgorithmConfig) -> Self {
        Self {
            pso: ParticleSwarmOptimizer::new(config.pso.clone()),
            ga: GeneticAlgorithm::new(config.ga.clone()),
            aco: AntColonyOptimizer::new(config.aco.clone()),
            hybrid: HybridOptimizer::new(config.hybrid.clone()),
            config,
        }
    }
    
    /// 优化任务分配
    pub async fn optimize_allocation(
        &self,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        match self.config.default_algorithm {
            AlgorithmType::ParticleSwarm => {
                self.optimize_with_pso(task, nodes).await
            }
            AlgorithmType::Genetic => {
                self.optimize_with_ga(task, nodes).await
            }
            AlgorithmType::AntColony => {
                self.optimize_with_aco(task, nodes).await
            }
            AlgorithmType::Hybrid => {
                self.optimize_with_hybrid(task, nodes).await
            }
        }
    }
    
    /// 使用粒子群算法优化
    pub async fn optimize_with_pso(
        &self,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        let problem = self.create_optimization_problem(task, nodes);
        let solution = self.pso.optimize(&problem).await?;
        self.solution_to_allocation(&solution, task, nodes)
    }
    
    /// 使用遗传算法优化
    pub async fn optimize_with_ga(
        &self,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        let problem = self.create_optimization_problem(task, nodes);
        let solution = self.ga.optimize(&problem).await?;
        self.solution_to_allocation(&solution, task, nodes)
    }
    
    /// 使用蚁群算法优化
    pub async fn optimize_with_aco(
        &self,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        let problem = self.create_routing_problem(task, nodes);
        let routes = self.aco.find_routes(&problem).await?;
        self.routes_to_allocation(&routes, task, nodes)
    }
    
    /// 使用混合算法优化
    pub async fn optimize_with_hybrid(
        &self,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        // 第一阶段：使用GA进行粗粒度优化
        let coarse_solution = self.optimize_with_ga(task, nodes).await?;
        
        // 第二阶段：使用PSO进行细粒度优化
        let refined_solution = self.refine_with_pso(&coarse_solution, task, nodes).await?;
        
        // 第三阶段：使用ACO优化通信路由
        let final_solution = self.optimize_routes_with_aco(&refined_solution, task, nodes).await?;
        
        Ok(final_solution)
    }
    
    /// 创建优化问题
    fn create_optimization_problem(
        &self,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> OptimizationProblem {
        OptimizationProblem {
            task: task.clone(),
            nodes: nodes.to_vec(),
            constraints: task.constraints.clone(),
            objectives: task.objectives.clone(),
        }
    }
    
    /// 创建路由问题
    fn create_routing_problem(
        &self,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> RoutingProblem {
        RoutingProblem {
            task: task.clone(),
            nodes: nodes.to_vec(),
            distance_matrix: self.calculate_distance_matrix(nodes),
            constraints: task.constraints.clone(),
        }
    }
    
    /// 计算距离矩阵
    fn calculate_distance_matrix(&self, nodes: &[NodeInfo]) -> Vec<Vec<f64>> {
        let n = nodes.len();
        let mut matrix = vec![vec![0.0; n]; n];
        
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    matrix[i][j] = 0.0;
                } else {
                    // 简化距离计算
                    matrix[i][j] = self.calculate_node_distance(&nodes[i], &nodes[j]);
                }
            }
        }
        
        matrix
    }
    
    /// 计算节点距离
    fn calculate_node_distance(&self, node1: &NodeInfo, node2: &NodeInfo) -> f64 {
        // 简化实现：基于地理位置或网络延迟
        if let (Some(loc1), Some(loc2)) = (&node1.location, &node2.location) {
            // 如果有地理位置，计算地理距离
            self.calculate_geo_distance(loc1, loc2)
        } else {
            // 否则使用网络延迟估计
            self.estimate_network_latency(node1, node2)
        }
    }
    
    /// 计算地理距离
    fn calculate_geo_distance(&self, loc1: &GeoLocation, loc2: &GeoLocation) -> f64 {
        // 简化实现：欧几里得距离
        let dx = loc1.x - loc2.x;
        let dy = loc1.y - loc2.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// 估计网络延迟
    fn estimate_network_latency(&self, node1: &NodeInfo, node2: &NodeInfo) -> f64 {
        // 简化实现：基于网络类型
        let latency = match (&node1.network_type, &node2.network_type) {
            (NetworkType::Local, NetworkType::Local) => 1.0,
            (NetworkType::Local, _) => 10.0,
            (_, NetworkType::Local) => 10.0,
            (NetworkType::Wifi, NetworkType::Wifi) => 5.0,
            (NetworkType::Mobile, NetworkType::Mobile) => 20.0,
            _ => 15.0,
        };
        
        latency
    }
    
    /// 将解决方案转换为任务分配
    fn solution_to_allocation(
        &self,
        solution: &OptimizationSolution,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        // 根据解决方案中的分配索引选择节点
        let mut assigned_nodes = Vec::new();
        let mut total_computation = 0.0;
        
        for &node_idx in &solution.allocation {
            if node_idx < nodes.len() {
                let node = &nodes[node_idx];
                
                // 计算该节点分配的子任务
                let subtask_computation = task.requirements.get("computation_required")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0) / solution.allocation.len() as f64;
                
                let subtask = Subtask {
                    id: format!("{}-{}", task.id, node_idx),
                    data_size: task.requirements.get("data_size")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1000) / solution.allocation.len() as u64,
                    computation_required: subtask_computation,
                };
                
                // 估计完成时间（简化：基于节点能力和负载）
                let node_capability = node.capabilities.get("compute_power")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let estimated_time = (subtask_computation / node_capability * 1000.0) as u64;
                
                assigned_nodes.push(AssignedNode {
                    node_id: node.id.clone(),
                    subtasks: vec![subtask],
                    estimated_completion_time_ms: estimated_time,
                });
                
                total_computation += subtask_computation;
            }
        }
        
        // 计算总成本和估计完成时间
        let estimated_completion_time_ms = assigned_nodes.iter()
            .map(|n| n.estimated_completion_time_ms)
            .max()
            .unwrap_or(0);
        
        let total_cost = total_computation * 0.01; // 简化成本计算
        
        Ok(TaskAllocation {
            assigned_nodes,
            subtasks: Vec::new(), // 子任务已分配到节点
            estimated_completion_time_ms,
            total_cost,
        })
    }
    
    /// 将路由转换为任务分配
    fn routes_to_allocation(
        &self,
        routes: &[Route],
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        if routes.is_empty() {
            anyhow::bail!("没有可用的路由")
        }
        
        // 选择最佳路由（成本最低）
        let best_route = routes.iter()
            .min_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap())
            .unwrap();
        
        // 将路由路径转换为节点分配
        let mut assigned_nodes = Vec::new();
        
        for &node_idx in &best_route.path {
            if node_idx < nodes.len() {
                let node = &nodes[node_idx];
                
                // 为路径中的每个节点创建子任务
                let subtask = Subtask {
                    id: format!("{}-route-{}", task.id, node_idx),
                    data_size: task.requirements.get("data_size")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1000) / best_route.path.len() as u64,
                    computation_required: task.requirements.get("computation_required")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0) / best_route.path.len() as f64,
                };
                
                // 估计完成时间（考虑路由距离）
                let node_capability = node.capabilities.get("compute_power")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let base_time = (subtask.computation_required / node_capability * 1000.0) as u64;
                let routing_delay = (best_route.distance * 10.0) as u64; // 简化：距离转换为延迟
                
                assigned_nodes.push(AssignedNode {
                    node_id: node.id.clone(),
                    subtasks: vec![subtask],
                    estimated_completion_time_ms: base_time + routing_delay,
                });
            }
        }
        
        // 计算总成本和估计完成时间
        let estimated_completion_time_ms = assigned_nodes.iter()
            .map(|n| n.estimated_completion_time_ms)
            .max()
            .unwrap_or(0);
        
        let total_cost = best_route.cost;
        
        Ok(TaskAllocation {
            assigned_nodes,
            subtasks: Vec::new(),
            estimated_completion_time_ms,
            total_cost,
        })
    }
    
    /// 使用PSO细化解决方案
    async fn refine_with_pso(
        &self,
        allocation: &TaskAllocation,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        // 基于当前分配创建优化问题
        let problem = self.create_optimization_problem(task, nodes);
        
        // 运行PSO优化
        let solution = self.pso.optimize(&problem).await?;
        
        // 转换为任务分配
        self.solution_to_allocation(&solution, task, nodes)
    }
    
    /// 使用ACO优化路由
    async fn optimize_routes_with_aco(
        &self,
        allocation: &TaskAllocation,
        task: &OptimizationTask,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<TaskAllocation> {
        // 基于分配中的节点创建路由问题
        let assigned_node_indices: Vec<usize> = allocation.assigned_nodes.iter()
            .filter_map(|an| nodes.iter().position(|n| n.id == an.node_id))
            .collect();
        
        if assigned_node_indices.is_empty() {
            return Ok(allocation.clone());
        }
        
        // 创建只包含已分配节点的路由问题
        let filtered_nodes: Vec<NodeInfo> = assigned_node_indices.iter()
            .map(|&idx| nodes[idx].clone())
            .collect();
        
        let routing_problem = RoutingProblem {
            task: task.clone(),
            nodes: filtered_nodes,
            distance_matrix: self.calculate_distance_matrix(&filtered_nodes),
            constraints: task.constraints.clone(),
        };
        
        // 运行ACO优化
        let routes = self.aco.find_routes(&routing_problem).await?;
        
        // 转换为任务分配
        self.routes_to_allocation(&routes, task, nodes)
    }
}

// 类型定义（简化）
#[derive(Debug, Clone)]
pub struct OptimizationTask {
    pub id: String,
    pub requirements: TaskRequirements,
    pub constraints: Constraints,
    pub objectives: Objectives,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub capabilities: NodeCapabilities,
    pub location: Option<GeoLocation>,
    pub network_type: NetworkType,
    pub current_load: f64,
}

#[derive(Debug, Clone)]
pub struct TaskAllocation {
    pub assigned_nodes: Vec<AssignedNode>,
    pub subtasks: Vec<Subtask>,
    pub estimated_completion_time_ms: u64,
    pub total_cost: f64,
}

#[derive(Debug, Clone)]
pub struct AssignedNode {
    pub node_id: String,
    pub subtasks: Vec<Subtask>,
    pub estimated_completion_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Subtask {
    pub id: String,
    pub data_size: u64,
    pub computation_required: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationProblem {
    pub task: OptimizationTask,
    pub nodes: Vec<NodeInfo>,
    pub constraints: Constraints,
    pub objectives: Objectives,
}

#[derive(Debug, Clone)]
pub struct RoutingProblem {
    pub task: OptimizationTask,
    pub nodes: Vec<NodeInfo>,
    pub distance_matrix: Vec<Vec<f64>>,
    pub constraints: Constraints,
}

#[derive(Debug, Clone)]
pub struct OptimizationSolution {
    pub allocation: Vec<usize>,
    pub fitness: f64,
    pub convergence: f64,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub path: Vec<usize>,
    pub cost: f64,
    pub distance: f64,
}

pub type TaskRequirements = serde_json::Value;
pub type Constraints = serde_json::Value;
pub type Objectives = serde_json::Value;
pub type NodeCapabilities = serde_json::Value;
pub type GeoLocation = (f64, f64); // (x, y)坐标

#[derive(Debug, Clone)]
pub enum NetworkType {
    Local,
    Wifi,
    Mobile,
    Satellite,
}

/// WASM兼容的接口
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    
    #[wasm_bindgen]
    pub struct WasmAlgorithmManager {
        manager: AlgorithmManager,
    }
    
    #[wasm_bindgen]
    impl WasmAlgorithmManager {
        #[wasm_bindgen(constructor)]
        pub fn new(config_js: JsValue) -> Result<WasmAlgorithmManager, JsValue> {
            let config: AlgorithmConfig = serde_wasm_bindgen::from_value(config_js)
                .map_err(|e| JsValue::from_str(&format!("配置解析失败: {}", e)))?;
            
            let manager = AlgorithmManager::new(config);
            Ok(WasmAlgorithmManager { manager })
        }
        
        #[wasm_bindgen]
        pub async fn optimize(&self, task_js: JsValue, nodes_js: JsValue) -> Result<JsValue, JsValue> {
            let task: OptimizationTask = serde_wasm_bindgen::from_value(task_js)
                .map_err(|e| JsValue::from_str(&format!("任务解析失败: {}", e)))?;
            
            let nodes: Vec<NodeInfo> = serde_wasm_bindgen::from_value(nodes_js)
                .map_err(|e| JsValue::from_str(&format!("节点解析失败: {}", e)))?;
            
            let allocation = self.manager.optimize_allocation(&task, &nodes).await
                .map_err(|e| JsValue::from_str(&format!("优化失败: {}", e)))?;
            
            serde_wasm_bindgen::to_value(&allocation)
                .map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
    }
}

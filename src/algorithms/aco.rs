//! 蚁群算法（Ant Colony Optimization）
//! 
//! 用于地理路由和路径优化的蚁群算法实现

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// ACO配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACOConfig {
    /// 蚂蚁数量
    pub ant_count: usize,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 信息素挥发率
    pub evaporation_rate: f64,
    /// 信息素强度
    pub pheromone_intensity: f64,
    /// 启发式重要性
    pub heuristic_importance: f64,
    /// 信息素重要性
    pub pheromone_importance: f64,
    /// 初始信息素值
    pub initial_pheromone: f64,
    /// 收敛阈值
    pub convergence_threshold: f64,
}

impl Default for ACOConfig {
    fn default() -> Self {
        Self {
            ant_count: 20,
            max_iterations: 100,
            evaporation_rate: 0.5,
            pheromone_intensity: 1.0,
            heuristic_importance: 2.0,
            pheromone_importance: 1.0,
            initial_pheromone: 0.1,
            convergence_threshold: 1e-6,
        }
    }
}

/// 蚂蚁
#[derive(Debug, Clone)]
pub struct Ant {
    /// 当前路径
    pub path: Vec<usize>,
    /// 已访问节点
    pub visited: Vec<bool>,
    /// 当前节点
    pub current_node: usize,
    /// 路径成本
    pub path_cost: f64,
    /// 路径距离
    pub path_distance: f64,
}

impl Ant {
    /// 创建新蚂蚁
    pub fn new(start_node: usize, node_count: usize) -> Self {
        let mut visited = vec![false; node_count];
        visited[start_node] = true;
        
        Self {
            path: vec![start_node],
            visited,
            current_node: start_node,
            path_cost: 0.0,
            path_distance: 0.0,
        }
    }
    
    /// 选择下一个节点
    pub fn select_next_node(
        &mut self,
        pheromone: &Pheromone,
        distance_matrix: &[Vec<f64>],
        config: &ACOConfig,
    ) -> Option<usize> {
        let node_count = self.visited.len();
        let mut probabilities = Vec::new();
        let mut total_probability = 0.0;
        
        // 计算每个未访问节点的选择概率
        for next_node in 0..node_count {
            if !self.visited[next_node] {
                let pheromone_value = pheromone.get(self.current_node, next_node);
                let distance = distance_matrix[self.current_node][next_node];
                
                // 避免除零错误
                let heuristic = if distance > 0.0 {
                    1.0 / distance
                } else {
                    1.0
                };
                
                let probability = pheromone_value.powf(config.pheromone_importance) *
                    heuristic.powf(config.heuristic_importance);
                
                probabilities.push((next_node, probability));
                total_probability += probability;
            }
        }
        
        if probabilities.is_empty() {
            return None;
        }
        
        // 轮盘赌选择
        let mut rng = rand::thread_rng();
        let random_value = rng.gen_range(0.0..total_probability);
        
        let mut cumulative = 0.0;
        for (node, probability) in probabilities {
            cumulative += probability;
            if random_value <= cumulative {
                return Some(node);
            }
        }
        
        // 如果轮盘赌失败，返回第一个可用节点
        Some(probabilities[0].0)
    }
    
    /// 移动到下一个节点
    pub fn move_to_node(&mut self, next_node: usize, distance_matrix: &[Vec<f64>]) {
        let distance = distance_matrix[self.current_node][next_node];
        
        self.path.push(next_node);
        self.visited[next_node] = true;
        self.current_node = next_node;
        self.path_distance += distance;
        
        // 简化成本计算：距离 + 节点数量惩罚
        self.path_cost = self.path_distance + (self.path.len() as f64 * 0.1);
    }
    
    /// 是否完成路径
    pub fn is_tour_complete(&self) -> bool {
        self.visited.iter().all(|&v| v)
    }
}

/// 信息素矩阵
#[derive(Debug, Clone)]
pub struct Pheromone {
    /// 信息素值矩阵
    matrix: Vec<Vec<f64>>,
}

impl Pheromone {
    /// 创建新信息素矩阵
    pub fn new(node_count: usize, initial_value: f64) -> Self {
        let matrix = vec![vec![initial_value; node_count]; node_count];
        
        Self { matrix }
    }
    
    /// 获取信息素值
    pub fn get(&self, from: usize, to: usize) -> f64 {
        self.matrix[from][to]
    }
    
    /// 更新信息素值
    pub fn update(&mut self, from: usize, to: usize, delta: f64) {
        self.matrix[from][to] += delta;
        self.matrix[to][from] += delta; // 对称更新
    }
    
    /// 挥发信息素
    pub fn evaporate(&mut self, rate: f64) {
        for i in 0..self.matrix.len() {
            for j in 0..self.matrix[i].len() {
                self.matrix[i][j] *= (1.0 - rate);
            }
        }
    }
    
    /// 限制信息素值范围
    pub fn clamp(&mut self, min: f64, max: f64) {
        for i in 0..self.matrix.len() {
            for j in 0..self.matrix[i].len() {
                self.matrix[i][j] = self.matrix[i][j].clamp(min, max);
            }
        }
    }
}

/// 蚁群优化器
pub struct AntColonyOptimizer {
    config: ACOConfig,
}

impl AntColonyOptimizer {
    /// 创建新的优化器
    pub fn new(config: ACOConfig) -> Self {
        Self { config }
    }
    
    /// 寻找最优路径
    pub async fn find_routes(
        &self,
        problem: &RoutingProblem,
    ) -> Result<Vec<Route>> {
        let node_count = problem.nodes.len();
        let mut pheromone = Pheromone::new(node_count, self.config.initial_pheromone);
        let mut best_routes = Vec::new();
        let mut best_cost = f64::INFINITY;
        let mut convergence_history = Vec::new();
        
        // 运行迭代
        for iteration in 0..self.config.max_iterations {
            let mut iteration_best_cost = f64::INFINITY;
            let mut iteration_best_routes = Vec::new();
            
            // 每只蚂蚁构建路径
            for ant_id in 0..self.config.ant_count {
                let start_node = ant_id % node_count;
                let mut ant = Ant::new(start_node, node_count);
                
                // 构建完整路径
                while !ant.is_tour_complete() {
                    if let Some(next_node) = ant.select_next_node(
                        &pheromone,
                        &problem.distance_matrix,
                        &self.config,
                    ) {
                        ant.move_to_node(next_node, &problem.distance_matrix);
                    } else {
                        break;
                    }
                }
                
                // 记录最佳路径
                if ant.path_cost < iteration_best_cost {
                    iteration_best_cost = ant.path_cost;
                    iteration_best_routes = vec![Route {
                        path: ant.path.clone(),
                        cost: ant.path_cost,
                        distance: ant.path_distance,
                    }];
                }
            }
            
            // 更新信息素
            self.update_pheromone(&mut pheromone, &iteration_best_routes);
            
            // 记录全局最佳
            if iteration_best_cost < best_cost {
                best_cost = iteration_best_cost;
                best_routes = iteration_best_routes.clone();
            }
            
            convergence_history.push(best_cost);
            
            // 检查收敛
            if self.is_converged(&convergence_history) {
                break;
            }
        }
        
        Ok(best_routes)
    }
    
    /// 更新信息素
    fn update_pheromone(&self, pheromone: &mut Pheromone, routes: &[Route]) {
        // 信息素挥发
        pheromone.evaporate(self.config.evaporation_rate);
        
        // 添加新信息素
        for route in routes {
            let pheromone_delta = self.config.pheromone_intensity / route.cost;
            
            for i in 0..route.path.len() - 1 {
                let from = route.path[i];
                let to = route.path[i + 1];
                pheromone.update(from, to, pheromone_delta);
            }
        }
        
        // 限制信息素值范围
        pheromone.clamp(0.001, 10.0);
    }
    
    /// 检查是否收敛
    fn is_converged(&self, history: &[f64]) -> bool {
        if history.len() < 10 {
            return false;
        }
        
        let window_size = 5.min(history.len());
        let recent_improvement = (history[history.len() - 1] -
            history[history.len() - window_size]).abs();
        
        recent_improvement < self.config.convergence_threshold
    }
}

// 使用mod.rs中的类型定义
use super::{RoutingProblem, Route};

/// WASM兼容的接口
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    
    #[wasm_bindgen]
    pub struct WasmACO {
        optimizer: AntColonyOptimizer,
    }
    
    #[wasm_bindgen]
    impl WasmACO {
        #[wasm_bindgen(constructor)]
        pub fn new(config_js: JsValue) -> Result<WasmACO, JsValue> {
            let config: ACOConfig = serde_wasm_bindgen::from_value(config_js)
                .map_err(|e| JsValue::from_str(&format!("配置解析失败: {}", e)))?;
            
            let optimizer = AntColonyOptimizer::new(config);
            Ok(WasmACO { optimizer })
        }
        
        #[wasm_bindgen]
        pub async fn find_routes(&self, problem_js: JsValue) -> Result<JsValue, JsValue> {
            let problem: RoutingProblem = serde_wasm_bindgen::from_value(problem_js)
                .map_err(|e| JsValue::from_str(&format!("问题解析失败: {}", e)))?;
            
            let routes = self.optimizer.find_routes(&problem).await
                .map_err(|e| JsValue::from_str(&format!("路径查找失败: {}", e)))?;
            
            serde_wasm_bindgen::to_value(&routes)
                .map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
    }
}

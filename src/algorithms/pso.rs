//! 粒子群算法（Particle Swarm Optimization）
//! 
//! 用于任务分配和资源调度的粒子群算法实现

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// PSO配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PSOConfig {
    /// 粒子数量
    pub particle_count: usize,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 惯性权重
    pub inertia_weight: f64,
    /// 认知系数
    pub cognitive_coefficient: f64,
    /// 社会系数
    pub social_coefficient: f64,
    /// 速度限制
    pub velocity_limit: f64,
    /// 位置限制
    pub position_limit: f64,
    /// 收敛阈值
    pub convergence_threshold: f64,
}

impl Default for PSOConfig {
    fn default() -> Self {
        Self {
            particle_count: 50,
            max_iterations: 100,
            inertia_weight: 0.729,
            cognitive_coefficient: 1.49445,
            social_coefficient: 1.49445,
            velocity_limit: 0.5,
            position_limit: 1.0,
            convergence_threshold: 1e-6,
        }
    }
}

/// 粒子
#[derive(Debug, Clone)]
pub struct Particle {
    /// 位置向量
    pub position: Vec<f64>,
    /// 速度向量
    pub velocity: Vec<f64>,
    /// 个体最优位置
    pub best_position: Vec<f64>,
    /// 个体最优适应度
    pub best_fitness: f64,
    /// 当前适应度
    pub current_fitness: f64,
}

impl Particle {
    /// 创建新粒子
    pub fn new(dimensions: usize) -> Self {
        let mut rng = rand::thread_rng();
        
        let position: Vec<f64> = (0..dimensions)
            .map(|_| rng.gen_range(0.0..1.0))
            .collect();
        
        let velocity: Vec<f64> = (0..dimensions)
            .map(|_| rng.gen_range(-0.1..0.1))
            .collect();
        
        Self {
            position: position.clone(),
            velocity,
            best_position: position,
            best_fitness: f64::NEG_INFINITY,
            current_fitness: f64::NEG_INFINITY,
        }
    }
    
    /// 更新位置
    pub fn update_position(&mut self, config: &PSOConfig) {
        for i in 0..self.position.len() {
            // 限制位置范围
            self.position[i] = self.position[i].clamp(0.0, config.position_limit);
        }
    }
    
    /// 更新速度
    pub fn update_velocity(
        &mut self,
        config: &PSOConfig,
        global_best_position: &[f64],
    ) {
        let mut rng = rand::thread_rng();
        
        for i in 0..self.velocity.len() {
            let r1: f64 = rng.gen();
            let r2: f64 = rng.gen();
            
            // 速度更新公式
            let cognitive = config.cognitive_coefficient * r1 * 
                (self.best_position[i] - self.position[i]);
            let social = config.social_coefficient * r2 * 
                (global_best_position[i] - self.position[i]);
            
            self.velocity[i] = config.inertia_weight * self.velocity[i] + 
                cognitive + social;
            
            // 限制速度范围
            self.velocity[i] = self.velocity[i].clamp(
                -config.velocity_limit,
                config.velocity_limit,
            );
        }
    }
    
    /// 更新位置基于速度
    pub fn move_particle(&mut self) {
        for i in 0..self.position.len() {
            self.position[i] += self.velocity[i];
        }
    }
}

/// 粒子群
#[derive(Debug, Clone)]
pub struct Swarm {
    /// 粒子列表
    pub particles: Vec<Particle>,
    /// 全局最优位置
    pub global_best_position: Vec<f64>,
    /// 全局最优适应度
    pub global_best_fitness: f64,
    /// 迭代次数
    pub iteration: usize,
    /// 收敛历史
    pub convergence_history: Vec<f64>,
}

impl Swarm {
    /// 创建新粒子群
    pub fn new(config: &PSOConfig, dimensions: usize) -> Self {
        let particles: Vec<Particle> = (0..config.particle_count)
            .map(|_| Particle::new(dimensions))
            .collect();
        
        Self {
            particles,
            global_best_position: vec![0.0; dimensions],
            global_best_fitness: f64::NEG_INFINITY,
            iteration: 0,
            convergence_history: Vec::new(),
        }
    }
    
    /// 运行一次迭代
    pub fn iterate(&mut self, config: &PSOConfig, fitness_fn: &dyn Fn(&[f64]) -> f64) {
        // 更新每个粒子
        for particle in &mut self.particles {
            // 更新速度
            particle.update_velocity(config, &self.global_best_position);
            
            // 移动粒子
            particle.move_particle();
            
            // 更新位置限制
            particle.update_position(config);
            
            // 计算适应度
            particle.current_fitness = fitness_fn(&particle.position);
            
            // 更新个体最优
            if particle.current_fitness > particle.best_fitness {
                particle.best_fitness = particle.current_fitness;
                particle.best_position = particle.position.clone();
                
                // 更新全局最优
                if particle.best_fitness > self.global_best_fitness {
                    self.global_best_fitness = particle.best_fitness;
                    self.global_best_position = particle.best_position.clone();
                }
            }
        }
        
        self.iteration += 1;
        self.convergence_history.push(self.global_best_fitness);
    }
    
    /// 检查是否收敛
    pub fn is_converged(&self, config: &PSOConfig) -> bool {
        if self.iteration < 10 {
            return false;
        }
        
        if self.iteration >= config.max_iterations {
            return true;
        }
        
        // 检查最近几次迭代的改进是否小于阈值
        let window_size = 5.min(self.convergence_history.len());
        if window_size < 2 {
            return false;
        }
        
        let recent_improvement = (self.convergence_history[self.convergence_history.len() - 1] -
            self.convergence_history[self.convergence_history.len() - window_size]).abs();
        
        recent_improvement < config.convergence_threshold
    }
}

/// 粒子群优化器
pub struct ParticleSwarmOptimizer {
    config: PSOConfig,
}

impl ParticleSwarmOptimizer {
    /// 创建新的优化器
    pub fn new(config: PSOConfig) -> Self {
        Self { config }
    }
    
    /// 优化问题
    pub async fn optimize(
        &self,
        problem: &OptimizationProblem,
    ) -> Result<OptimizationSolution> {
        let dimensions = problem.nodes.len();
        let mut swarm = Swarm::new(&self.config, dimensions);
        
        // 定义适应度函数
        let fitness_fn = |position: &[f64]| -> f64 {
            self.calculate_fitness(position, problem)
        };
        
        // 运行优化
        while !swarm.is_converged(&self.config) {
            swarm.iterate(&self.config, &fitness_fn);
        }
        
        // 返回最优解
        Ok(OptimizationSolution {
            allocation: self.position_to_allocation(&swarm.global_best_position),
            fitness: swarm.global_best_fitness,
            convergence: self.calculate_convergence(&swarm.convergence_history),
        })
    }
    
    /// 计算适应度
    fn calculate_fitness(&self, position: &[f64], problem: &OptimizationProblem) -> f64 {
        // 简化适应度计算：考虑负载均衡和性能
        let mut fitness = 0.0;
        
        // 负载均衡分数
        let load_balance = self.calculate_load_balance(position);
        fitness += load_balance * 0.4;
        
        // 性能分数（假设位置值表示节点能力）
        let performance = position.iter().sum::<f64>() / position.len() as f64;
        fitness += performance * 0.3;
        
        // 成本分数（位置值越低成本越低）
        let cost = 1.0 - position.iter().sum::<f64>() / position.len() as f64;
        fitness += cost * 0.3;
        
        fitness
    }
    
    /// 计算负载均衡
    fn calculate_load_balance(&self, position: &[f64]) -> f64 {
        if position.is_empty() {
            return 0.0;
        }
        
        let mean = position.iter().sum::<f64>() / position.len() as f64;
        let variance = position.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / position.len() as f64;
        
        // 方差越小，负载越均衡
        1.0 / (1.0 + variance.sqrt())
    }
    
    /// 将位置向量转换为分配方案
    fn position_to_allocation(&self, position: &[f64]) -> Vec<usize> {
        // 简化：选择位置值最高的几个节点
        let mut indexed: Vec<(usize, f64)> = position.iter()
            .enumerate()
            .map(|(i, &val)| (i, val))
            .collect();
        
        // 按值排序
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // 选择前几个节点
        let selected_count = (position.len() as f64 * 0.3).ceil() as usize;
        indexed.iter()
            .take(selected_count.max(1))
            .map(|&(i, _)| i)
            .collect()
    }
    
    /// 计算收敛度
    fn calculate_convergence(&self, history: &[f64]) -> f64 {
        if history.len() < 2 {
            return 0.0;
        }
        
        let last = history[history.len() - 1];
        let first = history[0];
        
        if first.abs() < 1e-10 {
            return 1.0;
        }
        
        (last - first).abs() / first.abs()
    }
}

// 类型定义
#[derive(Debug, Clone)]
pub struct OptimizationProblem {
    pub nodes: Vec<NodeInfo>,
    pub task_requirements: TaskRequirements,
}

pub type NodeInfo = serde_json::Value;
pub type TaskRequirements = serde_json::Value;

#[derive(Debug, Clone)]
pub struct OptimizationSolution {
    pub allocation: Vec<usize>,
    pub fitness: f64,
    pub convergence: f64,
}

/// WASM兼容的接口
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    
    #[wasm_bindgen]
    pub struct WasmPSO {
        optimizer: ParticleSwarmOptimizer,
    }
    
    #[wasm_bindgen]
    impl WasmPSO {
        #[wasm_bindgen(constructor)]
        pub fn new(config_js: JsValue) -> Result<WasmPSO, JsValue> {
            let config: PSOConfig = serde_wasm_bindgen::from_value(config_js)
                .map_err(|e| JsValue::from_str(&format!("配置解析失败: {}", e)))?;
            
            let optimizer = ParticleSwarmOptimizer::new(config);
            Ok(WasmPSO { optimizer })
        }
        
        #[wasm_bindgen]
        pub async fn optimize(&self, problem_js: JsValue) -> Result<JsValue, JsValue> {
            let problem: OptimizationProblem = serde_wasm_bindgen::from_value(problem_js)
                .map_err(|e| JsValue::from_str(&format!("问题解析失败: {}", e)))?;
            
            let solution = self.optimizer.optimize(&problem).await
                .map_err(|e| JsValue::from_str(&format!("优化失败: {}", e)))?;
            
            serde_wasm_bindgen::to_value(&solution)
                .map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
    }
}

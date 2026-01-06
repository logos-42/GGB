//! 混合优化算法
//! 
//! 结合遗传算法、粒子群算法和蚁群算法的混合优化器

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 混合算法配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridConfig {
    /// 第一阶段：遗传算法配置
    pub ga_stage: GAStageConfig,
    /// 第二阶段：粒子群算法配置
    pub pso_stage: PSOStageConfig,
    /// 第三阶段：蚁群算法配置
    pub aco_stage: ACOStageConfig,
    /// 阶段切换阈值
    pub stage_switch_threshold: f64,
    /// 最大总迭代次数
    pub max_total_iterations: usize,
}

/// 遗传算法阶段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GAStageConfig {
    /// 种群大小
    pub population_size: usize,
    /// 最大代数
    pub max_generations: usize,
    /// 交叉概率
    pub crossover_rate: f64,
    /// 变异概率
    pub mutation_rate: f64,
}

/// 粒子群算法阶段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PSOStageConfig {
    /// 粒子数量
    pub particle_count: usize,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 惯性权重
    pub inertia_weight: f64,
}

/// 蚁群算法阶段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACOStageConfig {
    /// 蚂蚁数量
    pub ant_count: usize,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 信息素挥发率
    pub evaporation_rate: f64,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            ga_stage: GAStageConfig {
                population_size: 50,
                max_generations: 20,
                crossover_rate: 0.8,
                mutation_rate: 0.1,
            },
            pso_stage: PSOStageConfig {
                particle_count: 30,
                max_iterations: 15,
                inertia_weight: 0.729,
            },
            aco_stage: ACOStageConfig {
                ant_count: 20,
                max_iterations: 10,
                evaporation_rate: 0.5,
            },
            stage_switch_threshold: 0.01,
            max_total_iterations: 50,
        }
    }
}

/// 混合优化器状态
#[derive(Debug, Clone)]
pub enum HybridStage {
    /// 第一阶段：遗传算法
    GeneticAlgorithm,
    /// 第二阶段：粒子群算法
    ParticleSwarm,
    /// 第三阶段：蚁群算法
    AntColony,
    /// 完成
    Completed,
}

/// 混合优化器
pub struct HybridOptimizer {
    config: HybridConfig,
    current_stage: HybridStage,
    current_iteration: usize,
    best_solution: Option<OptimizationSolution>,
    best_routes: Option<Vec<Route>>,
}

impl HybridOptimizer {
    /// 创建新的混合优化器
    pub fn new(config: HybridConfig) -> Self {
        Self {
            config,
            current_stage: HybridStage::GeneticAlgorithm,
            current_iteration: 0,
            best_solution: None,
            best_routes: None,
        }
    }
    
    /// 优化问题
    pub async fn optimize(
        &mut self,
        problem: &OptimizationProblem,
        routing_problem: &RoutingProblem,
    ) -> Result<HybridSolution> {
        // 重置状态
        self.current_stage = HybridStage::GeneticAlgorithm;
        self.current_iteration = 0;
        self.best_solution = None;
        self.best_routes = None;
        
        // 第一阶段：遗传算法（粗粒度优化）
        let ga_solution = self.run_genetic_algorithm(problem).await?;
        self.best_solution = Some(ga_solution.clone());
        
        // 检查是否需要切换到下一阶段
        if self.should_switch_stage(&ga_solution) {
            self.current_stage = HybridStage::ParticleSwarm;
            
            // 第二阶段：粒子群算法（细粒度优化）
            let pso_solution = self.run_particle_swarm(problem, &ga_solution).await?;
            
            if pso_solution.fitness > ga_solution.fitness {
                self.best_solution = Some(pso_solution.clone());
            }
            
            // 第三阶段：蚁群算法（路由优化）
            self.current_stage = HybridStage::AntColony;
            let routes = self.run_ant_colony(routing_problem).await?;
            self.best_routes = Some(routes);
        }
        
        self.current_stage = HybridStage::Completed;
        
        Ok(HybridSolution {
            allocation: self.best_solution.as_ref().map(|s| s.allocation.clone()).unwrap_or_default(),
            fitness: self.best_solution.as_ref().map(|s| s.fitness).unwrap_or(0.0),
            routes: self.best_routes.clone().unwrap_or_default(),
            stages_completed: self.get_completed_stages(),
        })
    }
    
    /// 运行遗传算法阶段
    async fn run_genetic_algorithm(&self, problem: &OptimizationProblem) -> Result<OptimizationSolution> {
        use super::ga::{GeneticAlgorithm, GAConfig, SelectionMethod};
        
        let ga_config = GAConfig {
            population_size: self.config.ga_stage.population_size,
            max_generations: self.config.ga_stage.max_generations,
            crossover_rate: self.config.ga_stage.crossover_rate,
            mutation_rate: self.config.ga_stage.mutation_rate,
            selection_method: SelectionMethod::Tournament(3),
            elitism_count: 2,
            convergence_threshold: self.config.stage_switch_threshold,
        };
        
        let ga = GeneticAlgorithm::new(ga_config);
        ga.optimize(problem).await
    }
    
    /// 运行粒子群算法阶段
    async fn run_particle_swarm(
        &self,
        problem: &OptimizationProblem,
        initial_solution: &OptimizationSolution,
    ) -> Result<OptimizationSolution> {
        use super::pso::{ParticleSwarmOptimizer, PSOConfig};
        
        let pso_config = PSOConfig {
            particle_count: self.config.pso_stage.particle_count,
            max_iterations: self.config.pso_stage.max_iterations,
            inertia_weight: self.config.pso_stage.inertia_weight,
            cognitive_coefficient: 1.49445,
            social_coefficient: 1.49445,
            velocity_limit: 0.5,
            position_limit: 1.0,
            convergence_threshold: self.config.stage_switch_threshold,
        };
        
        let pso = ParticleSwarmOptimizer::new(pso_config);
        pso.optimize(problem).await
    }
    
    /// 运行蚁群算法阶段
    async fn run_ant_colony(&self, routing_problem: &RoutingProblem) -> Result<Vec<Route>> {
        use super::aco::{AntColonyOptimizer, ACOConfig};
        
        let aco_config = ACOConfig {
            ant_count: self.config.aco_stage.ant_count,
            max_iterations: self.config.aco_stage.max_iterations,
            evaporation_rate: self.config.aco_stage.evaporation_rate,
            pheromone_intensity: 1.0,
            heuristic_importance: 2.0,
            pheromone_importance: 1.0,
            initial_pheromone: 0.1,
            convergence_threshold: self.config.stage_switch_threshold,
        };
        
        let aco = AntColonyOptimizer::new(aco_config);
        aco.find_routes(routing_problem).await
    }
    
    /// 检查是否需要切换阶段
    fn should_switch_stage(&self, solution: &OptimizationSolution) -> bool {
        // 基于收敛度决定是否切换
        solution.convergence < self.config.stage_switch_threshold ||
            self.current_iteration >= self.config.max_total_iterations
    }
    
    /// 获取已完成的阶段
    fn get_completed_stages(&self) -> Vec<String> {
        match self.current_stage {
            HybridStage::GeneticAlgorithm => vec!["遗传算法".to_string()],
            HybridStage::ParticleSwarm => vec!["遗传算法".to_string(), "粒子群算法".to_string()],
            HybridStage::AntColony => vec!["遗传算法".to_string(), "粒子群算法".to_string(), "蚁群算法".to_string()],
            HybridStage::Completed => vec!["遗传算法".to_string(), "粒子群算法".to_string(), "蚁群算法".to_string()],
        }
    }
    
    /// 获取当前阶段
    pub fn get_current_stage(&self) -> &HybridStage {
        &self.current_stage
    }
    
    /// 获取当前迭代次数
    pub fn get_current_iteration(&self) -> usize {
        self.current_iteration
    }
    
    /// 获取最佳解决方案
    pub fn get_best_solution(&self) -> Option<&OptimizationSolution> {
        self.best_solution.as_ref()
    }
}

/// 混合解决方案
#[derive(Debug, Clone)]
pub struct HybridSolution {
    /// 节点分配方案
    pub allocation: Vec<usize>,
    /// 适应度值
    pub fitness: f64,
    /// 优化后的路由
    pub routes: Vec<Route>,
    /// 已完成的优化阶段
    pub stages_completed: Vec<String>,
}

// 使用mod.rs中的类型定义
use super::{OptimizationProblem, OptimizationSolution, RoutingProblem, Route};

/// WASM兼容的接口
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    
    #[wasm_bindgen]
    pub struct WasmHybridOptimizer {
        optimizer: HybridOptimizer,
    }
    
    #[wasm_bindgen]
    impl WasmHybridOptimizer {
        #[wasm_bindgen(constructor)]
        pub fn new(config_js: JsValue) -> Result<WasmHybridOptimizer, JsValue> {
            let config: HybridConfig = serde_wasm_bindgen::from_value(config_js)
                .map_err(|e| JsValue::from_str(&format!("配置解析失败: {}", e)))?;
            
            let optimizer = HybridOptimizer::new(config);
            Ok(WasmHybridOptimizer { optimizer })
        }
        
        #[wasm_bindgen]
        pub async fn optimize(
            &mut self,
            problem_js: JsValue,
            routing_problem_js: JsValue,
        ) -> Result<JsValue, JsValue> {
            let problem: OptimizationProblem = serde_wasm_bindgen::from_value(problem_js)
                .map_err(|e| JsValue::from_str(&format!("问题解析失败: {}", e)))?;
            
            let routing_problem: RoutingProblem = serde_wasm_bindgen::from_value(routing_problem_js)
                .map_err(|e| JsValue::from_str(&format!("路由问题解析失败: {}", e)))?;
            
            let solution = self.optimizer.optimize(&problem, &routing_problem).await
                .map_err(|e| JsValue::from_str(&format!("优化失败: {}", e)))?;
            
            serde_wasm_bindgen::to_value(&solution)
                .map_err(|e| JsValue::from_str(&format!("序列化失败: {}", e)))
        }
        
        #[wasm_bindgen]
        pub fn get_current_stage(&self) -> String {
            match self.optimizer.get_current_stage() {
                HybridStage::GeneticAlgorithm => "遗传算法".to_string(),
                HybridStage::ParticleSwarm => "粒子群算法".to_string(),
                HybridStage::AntColony => "蚁群算法".to_string(),
                HybridStage::Completed => "已完成".to_string(),
            }
        }
    }
}

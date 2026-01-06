//! 遗传算法（Genetic Algorithm）
//! 
//! 用于节点优化和任务分配的遗传算法实现

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// GA配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GAConfig {
    /// 种群大小
    pub population_size: usize,
    /// 最大代数
    pub max_generations: usize,
    /// 交叉概率
    pub crossover_rate: f64,
    /// 变异概率
    pub mutation_rate: f64,
    /// 选择方法
    pub selection_method: SelectionMethod,
    /// 精英保留数量
    pub elitism_count: usize,
    /// 收敛阈值
    pub convergence_threshold: f64,
}

/// 选择方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMethod {
    /// 轮盘赌选择
    Roulette,
    /// 锦标赛选择
    Tournament(usize), // 锦标赛大小
    /// 排名选择
    Rank,
}

impl Default for GAConfig {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_generations: 50,
            crossover_rate: 0.8,
            mutation_rate: 0.1,
            selection_method: SelectionMethod::Tournament(3),
            elitism_count: 2,
            convergence_threshold: 1e-6,
        }
    }
}

/// 染色体
#[derive(Debug, Clone)]
pub struct Chromosome {
    /// 基因序列
    pub genes: Vec<f64>,
    /// 适应度
    pub fitness: f64,
    /// 是否被选择
    pub selected: bool,
}

impl Chromosome {
    /// 创建新染色体
    pub fn new(length: usize) -> Self {
        let mut rng = rand::thread_rng();
        let genes: Vec<f64> = (0..length)
            .map(|_| rng.gen_range(0.0..1.0))
            .collect();
        
        Self {
            genes,
            fitness: 0.0,
            selected: false,
        }
    }
    
    /// 交叉操作
    pub fn crossover(&self, other: &Self, rate: f64) -> (Self, Self) {
        let mut rng = rand::thread_rng();
        
        if rng.gen::<f64>() > rate || self.genes.is_empty() {
            return (self.clone(), other.clone());
        }
        
        let point = rng.gen_range(0..self.genes.len());
        
        let mut child1_genes = Vec::new();
        let mut child2_genes = Vec::new();
        
        for i in 0..self.genes.len() {
            if i < point {
                child1_genes.push(self.genes[i]);
                child2_genes.push(other.genes[i]);
            } else {
                child1_genes.push(other.genes[i]);
                child2_genes.push(self.genes[i]);
            }
        }
        
        (
            Self {
                genes: child1_genes,
                fitness: 0.0,
                selected: false,
            },
            Self {
                genes: child2_genes,
                fitness: 0.0,
                selected: false,
            },
        )
    }
    
    /// 变异操作
    pub fn mutate(&mut self, rate: f64) {
        let mut rng = rand::thread_rng();
        
        for gene in &mut self.genes {
            if rng.gen::<f64>() < rate {
                // 高斯变异
                let mutation = rng.gen_range(-0.1..0.1);
                *gene = (*gene + mutation).clamp(0.0, 1.0);
            }
        }
    }
    
    /// 计算适应度
    pub fn calculate_fitness(&mut self, fitness_fn: &dyn Fn(&[f64]) -> f64) {
        self.fitness = fitness_fn(&self.genes);
    }
}

/// 种群
#[derive(Debug, Clone)]
pub struct Population {
    /// 染色体列表
    pub chromosomes: Vec<Chromosome>,
    /// 当前代数
    pub generation: usize,
    /// 最佳适应度历史
    pub best_fitness_history: Vec<f64>,
    /// 平均适应度历史
    pub avg_fitness_history: Vec<f64>,
    /// 最佳染色体
    pub best_chromosome: Option<Chromosome>,
}

impl Population {
    /// 创建新种群
    pub fn new(config: &GAConfig, chromosome_length: usize) -> Self {
        let chromosomes: Vec<Chromosome> = (0..config.population_size)
            .map(|_| Chromosome::new(chromosome_length))
            .collect();
        
        Self {
            chromosomes,
            generation: 0,
            best_fitness_history: Vec::new(),
            avg_fitness_history: Vec::new(),
            best_chromosome: None,
        }
    }
    
    /// 评估种群
    pub fn evaluate(&mut self, fitness_fn: &dyn Fn(&[f64]) -> f64) {
        for chromosome in &mut self.chromosomes {
            chromosome.calculate_fitness(fitness_fn);
        }
        
        // 更新最佳染色体
        if let Some(best) = self.chromosomes.iter().max_by(|a, b| {
            a.fitness.partial_cmp(&b.fitness).unwrap()
        }) {
            if self.best_chromosome.as_ref().map_or(true, |old| best.fitness > old.fitness) {
                self.best_chromosome = Some(best.clone());
            }
        }
        
        // 记录统计信息
        let total_fitness: f64 = self.chromosomes.iter().map(|c| c.fitness).sum();
        let avg_fitness = total_fitness / self.chromosomes.len() as f64;
        let best_fitness = self.best_chromosome.as_ref().map_or(0.0, |c| c.fitness);
        
        self.avg_fitness_history.push(avg_fitness);
        self.best_fitness_history.push(best_fitness);
    }
    
    /// 选择操作
    pub fn select(&mut self, config: &GAConfig) {
        match config.selection_method {
            SelectionMethod::Roulette => self.roulette_selection(),
            SelectionMethod::Tournament(size) => self.tournament_selection(size),
            SelectionMethod::Rank => self.rank_selection(),
        }
    }
    
    /// 轮盘赌选择
    fn roulette_selection(&mut self) {
        let total_fitness: f64 = self.chromosomes.iter().map(|c| c.fitness).sum();
        
        if total_fitness <= 0.0 {
            // 如果所有适应度都为0，随机选择
            for chromosome in &mut self.chromosomes {
                chromosome.selected = rand::thread_rng().gen_bool(0.5);
            }
            return;
        }
        
        for chromosome in &mut self.chromosomes {
            let probability = chromosome.fitness / total_fitness;
            chromosome.selected = rand::thread_rng().gen_bool(probability);
        }
    }
    
    /// 锦标赛选择
    fn tournament_selection(&mut self, tournament_size: usize) {
        let mut rng = rand::thread_rng();
        
        for chromosome in &mut self.chromosomes {
            // 随机选择锦标赛参与者
            let mut tournament: Vec<&Chromosome> = Vec::new();
            for _ in 0..tournament_size {
                let idx = rng.gen_range(0..self.chromosomes.len());
                tournament.push(&self.chromosomes[idx]);
            }
            
            // 选择适应度最高的
            let best = tournament.iter()
                .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
                .unwrap();
            
            chromosome.selected = chromosome.fitness >= best.fitness;
        }
    }
    
    /// 排名选择
    fn rank_selection(&mut self) {
        // 按适应度排序
        let mut sorted: Vec<&Chromosome> = self.chromosomes.iter().collect();
        sorted.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
        
        // 分配选择概率
        for (rank, chromosome) in sorted.iter().enumerate() {
            let probability = (rank + 1) as f64 / self.chromosomes.len() as f64;
            // 这里需要修改染色体，所以需要不同的实现
            // 简化处理：标记前50%为选中
            if rank >= self.chromosomes.len() / 2 {
                // 找到并标记染色体
                if let Some(chrom) = self.chromosomes.iter_mut()
                    .find(|c| std::ptr::eq(c, *chromosome)) {
                    chrom.selected = true;
                }
            }
        }
    }
    
    /// 产生下一代
    pub fn produce_next_generation(&mut self, config: &GAConfig) {
        let mut next_generation = Vec::new();
        
        // 精英保留
        let mut sorted_chromosomes = self.chromosomes.clone();
        sorted_chromosomes.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        
        for i in 0..config.elitism_count.min(sorted_chromosomes.len()) {
            next_generation.push(sorted_chromosomes[i].clone());
        }
        
        // 交叉和变异产生新个体
        let selected_chromosomes: Vec<&Chromosome> = self.chromosomes
            .iter()
            .filter(|c| c.selected)
            .collect();
        
        while next_generation.len() < config.population_size {
            if selected_chromosomes.len() >= 2 {
                let parent1 = selected_chromosomes[rand::thread_rng().gen_range(0..selected_chromosomes.len())];
                let parent2 = selected_chromosomes[rand::thread_rng().gen_range(0..selected_chromosomes.len())];
                
                let (mut child1, mut child2) = parent1.crossover(parent2, config.crossover_rate);
                
                child1.mutate(config.mutation_rate);
                child2.mutate(config.mutation_rate);
                
                next_generation.push(child1);
                if next_generation.len() < config.population_size {
                    next_generation.push(child2);
                }
            } else {
                // 如果没有足够的选中个体，创建新个体
                next_generation.push(Chromosome::new(self.chromosomes[0].genes.len()));
            }
        }
        
        self.chromosomes = next_generation;
        self.generation += 1;
    }
    
    /// 检查是否收敛
    pub fn is_converged(&self, config: &GAConfig) -> bool {
        if self.generation >= config.max_generations {
            return true;
        }
        
        if self.best_fitness_history.len() < 10 {
            return false;
        }
        
        // 检查最近几代的最佳适应度变化
        let window_size = 10.min(self.best_fitness_history.len());
        let recent_improvement = (self.best_fitness_history[self.best_fitness_history.len() - 1] -
            self.best_fitness_history[self.best_fitness_history.len() - window_size]).abs();
        
        recent_improvement < config.convergence_threshold
    }
}

/// 遗传算法优化器
pub struct GeneticAlgorithm {
    config: GAConfig,
}

impl GeneticAlgorithm {
    /// 创建新的优化器
    pub fn new(config: GAConfig) -> Self {
        Self { config }
    }
    
    /// 优化问题
    pub async fn optimize(
        &self,
        problem: &OptimizationProblem,
    ) -> Result<OptimizationSolution> {
        let dimensions = problem.nodes.len();
        let mut population = Population::new(&self.config, dimensions);
        
        // 定义适应度函数
        let fitness_fn = |genes: &[f64]| -> f64 {
            self.calculate_fitness(genes, problem)
        };
        
        // 初始评估
        population.evaluate(&fitness_fn);
        
        // 进化循环
        while !population.is_converged(&self.config) {
            // 选择
            population.select(&self.config);
            
            // 产生下一代
            population.produce_next_generation(&self.config);
            
            // 评估新种群
            population.evaluate(&fitness_fn);
        }
        
        // 返回最优解
        if let Some(best_chromosome) = population.best_chromosome {
            Ok(OptimizationSolution {
                allocation: self.genes_to_allocation(&best_chromosome.genes),
                fitness: best_chromosome.fitness,
                convergence: self.calculate_convergence(&population.best_fitness_history),
            })
        } else {
            anyhow::bail!("未找到最优解")
        }
    }
    
    /// 计算适应度
    fn calculate_fitness(&self, genes: &[f64], problem: &OptimizationProblem) -> f64 {
        // 简化适应度计算
        let mut fitness = 0.0;
        
        // 多样性分数（基因差异）
        let diversity = self.calculate_diversity(genes);
        fitness += diversity * 0.3;
        
        // 均衡性分数
        let balance = self.calculate_balance(genes);
        fitness += balance * 0.4;
        
        // 效率分数（基因值越高效率越高）
        let efficiency = genes.iter().sum::<f64>() / genes.len() as f64;
        fitness += efficiency * 0.3;
        
        fitness
    }
    
    /// 计算多样性
    fn calculate_diversity(&self, genes: &[f64]) -> f64 {
        if genes.len() < 2 {
            return 0.0;
        }
        
        let mean = genes.iter().sum::<f64>() / genes.len() as f64;
        let variance = genes.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / genes.len() as f64;
        
        // 适度的方差表示多样性
        let normalized_variance = variance.min(0.25) / 0.25;
        normalized_variance
    }
    
    /// 计算均衡性
    fn calculate_balance(&self, genes: &[f64]) -> f64 {
        if genes.is_empty() {
            return 0.0;
        }
        
        let max = genes.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = genes.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        
        if max.abs() < 1e-10 {
            return 0.0;
        }
        
        // 最大值和最小值越接近，均衡性越好
        1.0 - (max - min) / max
    }
    
    /// 将基因转换为分配方案
    fn genes_to_allocation(&self, genes: &[f64]) -> Vec<usize> {
        // 简化：选择基因值超过阈值的节点
        let threshold = 0.5;
        
        genes.iter()
            .enumerate()
            .filter(|&(_, &val)| val > threshold)
            .map(|(i, _)| i)
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

// 使用mod.rs中的类型定义
use super::{OptimizationProblem, OptimizationSolution, NodeInfo, TaskRequirements};

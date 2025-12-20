use crate::types::{decompress_indices, SparseUpdate, TensorSnapshot};
use anyhow::{anyhow, Result};
use ndarray::Array1;
use ndarray_npy::ReadNpyExt;
use parking_lot::RwLock;
use rand::Rng;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct InferenceConfig {
    pub model_dim: usize,
    pub model_path: Option<PathBuf>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_dim: 256,
            model_path: None,
        }
    }
}

#[derive(Clone)]
pub struct InferenceEngine {
    state: Arc<RwLock<ModelState>>,
    config: InferenceConfig,
    memory_pressure: Arc<RwLock<MemoryPressure>>,
}

struct MemoryPressure {
    current_usage_mb: usize,
    pressure_threshold_mb: usize,
}

struct ModelState {
    params: Array1<f32>,
    residual: Array1<f32>,
    version: u64,
    // 收敛度追踪
    previous_params: Option<Array1<f32>>,
    hash_history: Vec<String>,
}

impl InferenceEngine {
    pub fn new(config: InferenceConfig) -> Result<Self> {
        let params = load_or_random(config.model_dim, config.model_path.as_deref())?;
        let residual = Array1::<f32>::zeros(params.len());
        
        // 估算内存使用：参数 + residual，每个 f32 4 字节
        let estimated_mb = (params.len() * 2 * 4) / (1024 * 1024);
        
        Ok(Self {
            state: Arc::new(RwLock::new(ModelState {
                params: params.clone(),
                residual,
                version: 1,
                previous_params: Some(params),
                hash_history: Vec::new(),
            })),
            config,
            memory_pressure: Arc::new(RwLock::new(MemoryPressure {
                current_usage_mb: estimated_mb,
                pressure_threshold_mb: estimated_mb * 2, // 阈值设为当前使用的 2 倍
            })),
        })
    }

    pub fn model_dim(&self) -> usize {
        self.config.model_dim
    }

    pub fn embedding(&self) -> Vec<f32> {
        self.state.read().params.to_vec()
    }

    pub fn tensor_snapshot(&self) -> TensorSnapshot {
        let state = self.state.read();
        TensorSnapshot::new(state.params.to_vec(), state.version)
    }

    pub fn tensor_hash(&self) -> String {
        self.tensor_snapshot().hash()
    }

    pub fn make_sparse_update(&self, k: usize) -> SparseUpdate {
        // 检查内存压力，如果压力大则减少 Top-K
        let effective_k = if self.is_memory_pressured() {
            (k / 2).max(4) // 内存压力时减少到一半，最少 4
        } else {
            k
        };
        
        let mut state = self.state.write();
        let dim = state.params.len();
        if dim == 0 {
            return SparseUpdate {
                indices: Vec::new(),
                values: Vec::new(),
                version: state.version,
            };
        }
        let mut delta = vec![0f32; dim];
        for i in 0..dim {
            delta[i] = state.params[i] + state.residual[i];
        }
        let mut idx_val: Vec<(usize, f32)> =
            delta.iter().enumerate().map(|(i, v)| (i, *v)).collect();
        idx_val.sort_by(|a, b| {
            let av = a.1.abs();
            let bv = b.1.abs();
            bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
        });
        let take = effective_k.min(dim);
        let topk = &idx_val[..take];
        let mut sparse_vals = Vec::with_capacity(take);
        let mut sparse_idx = Vec::with_capacity(take);
        let mut last = 0usize;
        for (i, v) in topk {
            let diff = if sparse_idx.is_empty() {
                *i as u32
            } else {
                (*i - last) as u32
            };
            sparse_idx.push(diff);
            sparse_vals.push(*v);
            state.residual[*i] = delta[*i] - *v;
            last = *i;
        }
        state.version = state.version.saturating_add(1);
        SparseUpdate {
            indices: sparse_idx,
            values: sparse_vals,
            version: state.version,
        }
    }

    /// 检查是否处于内存压力状态
    pub fn is_memory_pressured(&self) -> bool {
        let pressure = self.memory_pressure.read();
        pressure.current_usage_mb >= pressure.pressure_threshold_mb
    }

    /// 获取当前内存使用（MB）
    #[allow(dead_code)]
    pub fn memory_usage_mb(&self) -> usize {
        self.memory_pressure.read().current_usage_mb
    }

    /// 更新内存压力阈值
    #[allow(dead_code)]
    pub fn set_memory_threshold(&self, threshold_mb: usize) {
        self.memory_pressure.write().pressure_threshold_mb = threshold_mb;
    }

    pub fn apply_sparse_update(&self, update: &SparseUpdate) {
        if update.indices.is_empty() {
            return;
        }
        let idxs = decompress_indices(&update.indices);
        let mut state = self.state.write();
        
        // 保存当前参数用于收敛度计算
        state.previous_params = Some(state.params.clone());
        
        for (pos, &v) in idxs.iter().zip(update.values.iter()) {
            if *pos < state.params.len() {
                let old = state.params[*pos];
                let merged = 0.5 * old + 0.5 * v;
                state.params[*pos] = merged;
                state.residual[*pos] += old - merged;
            }
        }
        state.version = state.version.max(update.version);
        
        // 更新 hash 历史
        drop(state);
        let hash = self.tensor_hash();
        state = self.state.write();
        state.hash_history.push(hash);
        if state.hash_history.len() > 10 {
            state.hash_history.remove(0);
        }
    }

    pub fn apply_dense_snapshot(&self, snapshot: &TensorSnapshot) {
        let mut state = self.state.write();
        
        // 保存当前参数用于收敛度计算
        state.previous_params = Some(state.params.clone());
        
        let len = state.params.len().min(snapshot.values.len());
        for i in 0..len {
            state.params[i] = 0.8 * state.params[i] + 0.2 * snapshot.values[i];
        }
        state.version = state.version.max(snapshot.version);
        
        // 更新 hash 历史
        drop(state);
        let hash = self.tensor_hash();
        state = self.state.write();
        state.hash_history.push(hash);
        if state.hash_history.len() > 10 {
            state.hash_history.remove(0);
        }
    }

    pub fn local_train_step(&self) {
        let mut rng = rand::thread_rng();
        let mut state = self.state.write();
        
        // 保存当前参数用于收敛度计算
        state.previous_params = Some(state.params.clone());
        
        for v in state.params.iter_mut() {
            *v += rng.gen_range(-1e-3..1e-3);
        }
        state.version = state.version.saturating_add(1);
        
        // 更新 hash 历史（保留最近 10 个）
        let hash = self.tensor_hash();
        state.hash_history.push(hash);
        if state.hash_history.len() > 10 {
            state.hash_history.remove(0);
        }
    }

    /// 计算模型收敛度（0.0-1.0）
    /// 1.0 表示完全收敛（参数不再变化），0.0 表示完全不收敛
    pub fn convergence_score(&self) -> f32 {
        let state = self.state.read();
        
        // 如果没有历史数据，返回 0.0
        if state.previous_params.is_none() || state.params.len() == 0 {
            return 0.0;
        }
        
        let prev = state.previous_params.as_ref().unwrap();
        
        // 计算参数的平均变化幅度
        let mut total_change = 0.0f32;
        let mut count = 0;
        for i in 0..state.params.len().min(prev.len()) {
            let change = (state.params[i] - prev[i]).abs();
            total_change += change;
            count += 1;
        }
        
        if count == 0 {
            return 0.0;
        }
        
        let avg_change = total_change / count as f32;
        
        // 计算参数的标准差（衡量参数分布）
        let mean = state.params.iter().sum::<f32>() / state.params.len() as f32;
        let variance = state.params.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / state.params.len() as f32;
        let std_dev = variance.sqrt();
        
        // 收敛度计算：
        // - 变化幅度越小，收敛度越高
        // - 标准差越小（参数更集中），收敛度越高
        // - hash 变化频率越低，收敛度越高
        
        let change_score = (1.0 - (avg_change * 1000.0).min(1.0)).max(0.0);
        let std_score = if std_dev > 0.0 {
            (1.0 - (std_dev * 10.0).min(1.0)).max(0.0)
        } else {
            1.0
        };
        
        // hash 稳定性（如果最近 5 个 hash 都相同，说明模型稳定）
        let hash_stability = if state.hash_history.len() >= 5 {
            let recent = &state.hash_history[state.hash_history.len() - 5..];
            let all_same = recent.windows(2).all(|w| w[0] == w[1]);
            if all_same { 1.0 } else { 0.5 }
        } else {
            0.0
        };
        
        // 加权平均
        (change_score * 0.4 + std_score * 0.3 + hash_stability * 0.3).clamp(0.0, 1.0)
    }

    /// 计算参数的平均变化幅度
    pub fn parameter_change_magnitude(&self) -> f32 {
        let state = self.state.read();
        if let Some(prev) = &state.previous_params {
            let mut total_change = 0.0f32;
            let mut count = 0;
            for i in 0..state.params.len().min(prev.len()) {
                total_change += (state.params[i] - prev[i]).abs();
                count += 1;
            }
            if count > 0 {
                total_change / count as f32
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// 计算参数的标准差
    pub fn parameter_std_dev(&self) -> f32 {
        let state = self.state.read();
        if state.params.len() == 0 {
            return 0.0;
        }
        let mean = state.params.iter().sum::<f32>() / state.params.len() as f32;
        let variance = state.params.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / state.params.len() as f32;
        variance.sqrt()
    }
}

/// 验证模型文件
pub fn validate_model_file(path: &Path, expected_dim: Option<usize>) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("模型文件不存在: {:?}", path));
    }
    
    // 检查文件扩展名
    if path.extension().and_then(|s| s.to_str()) != Some("npy") {
        return Err(anyhow!("模型文件必须是 .npy 格式"));
    }
    
    // 尝试加载文件
    let file = File::open(path)?;
    let arr: Array1<f32> = Array1::read_npy(file)?;
    
    // 检查维度
    if let Some(expected) = expected_dim {
        if arr.len() != expected {
            return Err(anyhow!(
                "模型维度不匹配: 期望 {}, 实际 {}",
                expected,
                arr.len()
            ));
        }
    }
    
    // 检查参数值是否在合理范围内
    let min_val = arr.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = arr.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    
    if min_val.is_infinite() || max_val.is_infinite() || min_val.is_nan() || max_val.is_nan() {
        return Err(anyhow!("模型包含无效值 (NaN 或 Infinity)"));
    }
    
    // 检查参数范围是否合理（通常在 -10 到 10 之间）
    if min_val < -100.0 || max_val > 100.0 {
        return Err(anyhow!(
            "模型参数值超出合理范围: [{}, {}]",
            min_val,
            max_val
        ));
    }
    
    Ok(())
}

fn load_or_random(dim: usize, path: Option<&Path>) -> Result<Array1<f32>> {
    if let Some(path) = path {
        if path.exists() {
            // 验证模型文件
            validate_model_file(path, Some(dim))?;
            
            let file = File::open(path)?;
            let arr: Array1<f32> = Array1::read_npy(file)?;
            
            // 再次检查维度（双重验证）
            if arr.len() != dim {
                return Err(anyhow!(
                    "模型维度不匹配: 配置 {}, 文件 {}",
                    dim,
                    arr.len()
                ));
            }
            
            return Ok(arr);
        } else {
            return Err(anyhow!("model file {:?} not found", path));
        }
    }
    let mut rng = rand::thread_rng();
    let data: Vec<f32> = (0..dim).map(|_| rng.gen_range(-0.1..0.1)).collect();
    Ok(Array1::from_vec(data))
}

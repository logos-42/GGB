//! 状态管理模块
//! 
//! 定义全局状态结构和线程安全管理

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

/// 全局训练状态
#[derive(Debug, Clone)]
pub struct TrainingState {
    pub is_running: bool,
    pub current_epoch: u32,
    pub total_epochs: u32,
    pub accuracy: f64,
    pub loss: f64,
    pub samples_processed: u64,
    pub current_model: String,
}

impl TrainingState {
    /// 创建新的训练状态
    pub fn new() -> Self {
        Self {
            is_running: false,
            current_epoch: 0,
            total_epochs: 100,
            accuracy: 0.0,
            loss: 1.0,
            samples_processed: 0,
            current_model: "default".to_string(),
        }
    }
    
    /// 更新训练进度
    pub fn update_progress(&mut self, epoch: u32, accuracy: f64, loss: f64, samples: u64) {
        self.current_epoch = epoch;
        self.accuracy = accuracy;
        self.loss = loss;
        self.samples_processed = samples;
    }
    
    /// 重置训练状态
    pub fn reset(&mut self) {
        self.is_running = false;
        self.current_epoch = 0;
        self.total_epochs = 100;
        self.accuracy = 0.0;
        self.loss = 1.0;
        self.samples_processed = 0;
    }
    
    /// 获取训练进度百分比
    pub fn progress_percentage(&self) -> f64 {
        if self.total_epochs == 0 {
            0.0
        } else {
            (self.current_epoch as f64 / self.total_epochs as f64) * 100.0
        }
    }
    
    /// 获取训练状态描述
    pub fn status_description(&self) -> &'static str {
        if self.is_running {
            "训练中"
        } else if self.current_epoch > 0 {
            "已完成"
        } else {
            "未开始"
        }
    }
}

/// 模型配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dimensions: usize,
    pub learning_rate: f64,
    pub batch_size: usize,
}

impl ModelConfig {
    /// 获取模型内存需求（GB）
    pub fn memory_requirement_gb(&self) -> f64 {
        (self.dimensions * self.batch_size * 4) as f64 / (1024.0 * 1024.0 * 1024.0)
    }
    
    /// 获取模型复杂度评分
    pub fn complexity_score(&self) -> f64 {
        let params_score = (self.dimensions as f64).log10() * 10.0;
        let batch_score = (self.batch_size as f64).log10() * 5.0;
        params_score + batch_score
    }
    
    /// 检查是否为大型模型
    pub fn is_large_model(&self) -> bool {
        self.dimensions > 1000 || self.batch_size > 32
    }
    
    /// 获取推荐的设备类型
    pub fn recommended_device_type(&self) -> &'static str {
        if self.is_large_model() {
            "桌面或高性能平板"
        } else if self.dimensions > 500 {
            "平板或高性能手机"
        } else {
            "手机"
        }
    }
}

/// API密钥条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub id: String,
    pub name: String,
    pub key: String,
    pub created_at: String,
    pub last_used: Option<String>,
    pub usage_count: u32,
}

impl ApiKeyEntry {
    /// 创建新的API密钥
    pub fn new(name: String, key: String) -> Self {
        Self {
            id: format!("sk-williw-{}", uuid::Uuid::new_v4()),
            name,
            key,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
            usage_count: 0,
        }
    }
    
    /// 记录使用
    pub fn record_usage(&mut self) {
        self.last_used = Some(chrono::Utc::now().to_rfc3339());
        self.usage_count += 1;
    }
    
    /// 检查密钥是否过期
    pub fn is_expired(&self, days: u64) -> bool {
        if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&self.created_at) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(created);
            duration.num_days() > days as i64
        } else {
            false
        }
    }
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub privacy_level: String,
    pub bandwidth_budget: u32,
    pub network_config: NetworkConfig,
    pub checkpoint_settings: CheckpointSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_peers: u32,
    pub bootstrap_nodes: Vec<String>,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSettings {
    pub enabled: bool,
    pub interval_minutes: u32,
    pub max_checkpoints: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            privacy_level: "medium".to_string(),
            bandwidth_budget: 10,
            network_config: NetworkConfig {
                max_peers: 10,
                bootstrap_nodes: vec![],
                port: 9000,
            },
            checkpoint_settings: CheckpointSettings {
                enabled: true,
                interval_minutes: 5,
                max_checkpoints: 10,
            },
        }
    }
}

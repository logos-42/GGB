use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::types::{DeviceType, GpuComputeApi, NetworkType};

/// 设备能力信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// 最大可用内存（MB）
    pub max_memory_mb: usize,
    /// CPU 核心数
    pub cpu_cores: usize,
    /// 是否有 GPU 支持
    pub has_gpu: bool,
    /// CPU 架构（如 "x86_64", "aarch64", "arm" 等）
    pub cpu_architecture: String,
    /// 支持的 GPU 计算 API 列表
    pub gpu_compute_apis: Vec<GpuComputeApi>,
    /// 是否有 TPU 支持（可选）
    pub has_tpu: Option<bool>,
    /// 当前网络类型
    pub network_type: NetworkType,
    /// 电池电量（0.0-1.0），None 表示无法检测或桌面设备
    pub battery_level: Option<f32>,
    /// 是否正在充电
    pub is_charging: bool,
    /// 设备类型
    pub device_type: DeviceType,
}

impl DeviceCapabilities {
    /// 创建默认设备能力（用于桌面/服务器环境）
    pub fn default_desktop() -> Self {
        Self {
            max_memory_mb: 2048,
            cpu_cores: 4,
            has_gpu: false,
            cpu_architecture: std::env::consts::ARCH.to_string(),
            gpu_compute_apis: Vec::new(),
            has_tpu: None,
            network_type: NetworkType::WiFi,
            battery_level: None,
            is_charging: true,
            device_type: DeviceType::Desktop,
        }
    }

    /// 创建低端移动设备能力
    pub fn low_end_mobile() -> Self {
        Self {
            max_memory_mb: 512,
            cpu_cores: 2,
            has_gpu: false,
            cpu_architecture: std::env::consts::ARCH.to_string(),
            gpu_compute_apis: Vec::new(),
            has_tpu: None,
            network_type: NetworkType::Cellular4G,
            battery_level: Some(0.5),
            is_charging: false,
            device_type: DeviceType::Phone,
        }
    }

    /// 创建中端移动设备能力
    pub fn mid_range_mobile() -> Self {
        Self {
            max_memory_mb: 1024,
            cpu_cores: 4,
            has_gpu: false,
            cpu_architecture: std::env::consts::ARCH.to_string(),
            gpu_compute_apis: Vec::new(),
            has_tpu: None,
            network_type: NetworkType::Cellular5G,
            battery_level: Some(0.7),
            is_charging: false,
            device_type: DeviceType::Phone,
        }
    }

    /// 创建高端移动设备能力
    pub fn high_end_mobile() -> Self {
        Self {
            max_memory_mb: 2048,
            cpu_cores: 8,
            has_gpu: true,
            cpu_architecture: std::env::consts::ARCH.to_string(),
            gpu_compute_apis: Vec::new(),
            has_tpu: None,
            network_type: NetworkType::WiFi,
            battery_level: Some(0.9),
            is_charging: true,
            device_type: DeviceType::Tablet,
        }
    }

    /// 根据内存计算推荐的模型维度
    /// 每个参数约 4 字节（f32），预留 50% 内存给系统和其他用途
    pub fn recommended_model_dim(&self) -> usize {
        // 预留 50% 内存给系统，剩余用于模型
        let available_mb = self.max_memory_mb / 2;
        // 每个参数 4 字节，加上 residual 需要 2 倍空间
        let bytes_per_param = 8; // params + residual
        let available_bytes = available_mb * 1024 * 1024;
        let max_params = available_bytes / bytes_per_param;
        // 限制在合理范围内
        max_params.min(4096).max(64)
    }

    /// 根据电池状态推荐训练频率（秒）
    pub fn recommended_tick_interval(&self) -> Duration {
        if let Some(level) = self.battery_level {
            if self.is_charging {
                Duration::from_secs(10) // 充电时正常频率
            } else if level > 0.5 {
                Duration::from_secs(10) // 高电量正常频率
            } else if level > 0.2 {
                Duration::from_secs(30) // 中电量降低频率
            } else {
                Duration::from_secs(60) // 低电量最低频率
            }
        } else {
            Duration::from_secs(10) // 桌面设备或无法检测时使用默认
        }
    }

    /// 判断是否应该暂停训练（低电量且未充电）
    pub fn should_pause_training(&self) -> bool {
        if let Some(level) = self.battery_level {
            !self.is_charging && level < 0.1
        } else {
            false
        }
    }

    /// 根据设备能力推荐邻居数量
    pub fn recommended_max_neighbors(&self) -> usize {
        match self.device_type {
            DeviceType::Tablet | DeviceType::Desktop => 8,
            DeviceType::Phone => {
                if self.max_memory_mb >= 1024 {
                    6
                } else {
                    4
                }
            }
            DeviceType::Unknown => 4,
        }
    }

    /// 根据设备能力推荐备份邻居数量
    pub fn recommended_failover_pool(&self) -> usize {
        self.recommended_max_neighbors() / 2
    }
}


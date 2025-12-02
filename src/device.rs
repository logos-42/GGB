use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// 网络类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    WiFi,
    Cellular4G,
    Cellular5G,
    Unknown,
}

impl NetworkType {
    /// 判断是否允许密集快照传输
    pub fn allows_dense_snapshot(&self) -> bool {
        matches!(self, NetworkType::WiFi)
    }

    /// 获取网络类型的带宽系数（用于调整带宽预算）
    pub fn bandwidth_factor(&self) -> f32 {
        match self {
            NetworkType::WiFi => 1.0,
            NetworkType::Cellular5G => 0.5,
            NetworkType::Cellular4G => 0.3,
            NetworkType::Unknown => 0.2,
        }
    }
}

/// 设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Phone,
    Tablet,
    Desktop,
    Unknown,
}

/// 设备能力信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// 最大可用内存（MB）
    pub max_memory_mb: usize,
    /// CPU 核心数
    pub cpu_cores: usize,
    /// 是否有 GPU 支持
    pub has_gpu: bool,
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

/// 设备能力检测器（用于未来平台特定实现）
pub struct DeviceDetector;

impl DeviceDetector {
    /// 检测当前设备能力
    /// 注意：在真正的移动端实现中，这里需要通过 FFI 调用平台 API
    /// 目前返回模拟数据，实际部署时需要替换为真实检测逻辑
    pub fn detect() -> DeviceCapabilities {
        // TODO: 实现真正的平台检测
        // 目前使用环境变量或默认值
        if let Ok(device_type) = std::env::var("GGS_DEVICE_TYPE") {
            match device_type.as_str() {
                "low" => DeviceCapabilities::low_end_mobile(),
                "mid" => DeviceCapabilities::mid_range_mobile(),
                "high" => DeviceCapabilities::high_end_mobile(),
                _ => DeviceCapabilities::default_desktop(),
            }
        } else {
            DeviceCapabilities::default_desktop()
        }
    }

    /// 检测网络类型
    /// 注意：需要平台特定实现
    pub fn detect_network_type() -> NetworkType {
        // TODO: 通过 FFI 调用平台网络检测 API
        // 目前返回默认值
        if let Ok(net_type) = std::env::var("GGS_NETWORK_TYPE") {
            match net_type.as_str() {
                "wifi" => NetworkType::WiFi,
                "5g" => NetworkType::Cellular5G,
                "4g" => NetworkType::Cellular4G,
                _ => NetworkType::Unknown,
            }
        } else {
            NetworkType::Unknown
        }
    }

    /// 检测电池状态
    /// 注意：需要平台特定实现
    pub fn detect_battery() -> (Option<f32>, bool) {
        // TODO: 通过 FFI 调用平台电池检测 API
        // 目前从环境变量读取或返回 None
        if let Ok(level) = std::env::var("GGS_BATTERY_LEVEL") {
            if let Ok(level_f) = level.parse::<f32>() {
                let charging = std::env::var("GGS_BATTERY_CHARGING")
                    .map(|v| v == "true")
                    .unwrap_or(false);
                return (Some(level_f.clamp(0.0, 1.0)), charging);
            }
        }
        (None, false)
    }
}

/// 设备能力管理器（支持运行时更新）
pub struct DeviceManager {
    capabilities: Arc<parking_lot::RwLock<DeviceCapabilities>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let caps = DeviceDetector::detect();
        Self {
            capabilities: Arc::new(parking_lot::RwLock::new(caps)),
        }
    }

    pub fn with_capabilities(capabilities: DeviceCapabilities) -> Self {
        Self {
            capabilities: Arc::new(parking_lot::RwLock::new(capabilities)),
        }
    }

    pub fn get(&self) -> DeviceCapabilities {
        self.capabilities.read().clone()
    }

    pub fn update_network_type(&self, network_type: NetworkType) {
        self.capabilities.write().network_type = network_type;
    }

    pub fn update_battery(&self, level: Option<f32>, is_charging: bool) {
        let mut caps = self.capabilities.write();
        caps.battery_level = level;
        caps.is_charging = is_charging;
    }

    pub fn refresh(&self) {
        let mut caps = self.capabilities.write();
        *caps = DeviceDetector::detect();
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}


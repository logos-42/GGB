use serde::{Deserialize, Serialize};

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

/// GPU 计算 API 枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum GpuComputeApi {
    CUDA,      // NVIDIA CUDA
    OpenCL,    // 跨平台 OpenCL
    Metal,     // Apple Metal
    Vulkan,    // Vulkan API
    DirectX,   // Windows DirectX 12
}


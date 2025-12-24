//! 设备能力检测模块
//! 
//! 此模块提供了跨平台的设备能力检测功能，包括：
//! - CPU、GPU、NPU/TPU 检测
//! - 网络类型检测（WiFi、4G、5G）
//! - 电池状态检测
//! - 设备能力管理和运行时更新

pub mod types;
pub mod capabilities;
pub mod detector;
pub mod manager;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub mod platform;

// 重新导出公共接口
#[allow(unused_imports)] // 这些是公共 API，供外部使用
pub use types::{DeviceType, GpuComputeApi, NetworkType};
pub use capabilities::DeviceCapabilities;
#[allow(unused_imports)] // 这是公共 API，供外部使用
pub use detector::DeviceDetector;
pub use manager::DeviceManager;


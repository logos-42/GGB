//! 设备检测模块
//! 
//! 提供跨平台的设备能力检测功能。

mod platform;
mod capabilities;

// 重新导出公共接口
pub use platform::*;
pub use capabilities::*;

/// 设备检测器
pub struct DeviceDetector;

impl DeviceDetector {
    /// 检测当前设备能力
    pub fn detect() -> DeviceCapabilities {
        // 检测 CPU 架构
        let cpu_architecture = Self::detect_cpu_architecture();
        
        // 检测 GPU API
        let gpu_compute_apis = Self::detect_gpu_apis();
        let has_gpu = !gpu_compute_apis.is_empty();
        
        // 检测 TPU/NPU
        let has_tpu = Self::detect_tpu();
        
        // 检测内存和 CPU 核心数
        let (max_memory_mb, cpu_cores) = Self::detect_memory_and_cpu();
        
        // 检测网络类型
        let network_type = Self::detect_network_type();
        
        // 检测电池状态
        let (battery_level, is_charging) = Self::detect_battery();
        
        // 判断设备类型
        let device_type = if battery_level.is_some() {
            if max_memory_mb >= 2048 {
                DeviceType::Tablet
            } else {
                DeviceType::Phone
            }
        } else {
            DeviceType::Desktop
        };
        
        DeviceCapabilities {
            max_memory_mb,
            cpu_cores,
            has_gpu,
            cpu_architecture,
            gpu_compute_apis,
            has_tpu,
            network_type,
            battery_level,
            is_charging,
            device_type,
        }
    }
    
    /// 检测 CPU 架构
    fn detect_cpu_architecture() -> String {
        use sysinfo::{System, SystemExt, CpuExt};
        
        let mut system = System::default();
        system.refresh_cpu();
        
        // 尝试获取 CPU 品牌字符串
        if let Some(cpu) = system.cpus().first() {
            let brand = cpu.brand();
            if !brand.is_empty() {
                return format!("{} ({})", std::env::consts::ARCH, brand);
            }
        }
        
        // 回退到基本架构
        std::env::consts::ARCH.to_string()
    }
    
    /// 检测 GPU 计算 API
    fn detect_gpu_apis() -> Vec<GpuComputeApi> {
        platform::detect_gpu_apis()
    }
    
    /// 检测 TPU/NPU 支持
    fn detect_tpu() -> Option<bool> {
        platform::detect_tpu()
    }
    
    /// 检测内存和 CPU 核心数
    fn detect_memory_and_cpu() -> (u64, u32) {
        use sysinfo::{System, SystemExt};
        
        let mut system = System::default();
        system.refresh_memory();
        system.refresh_cpu();
        
        let max_memory_mb = system.total_memory() / 1024;
        let cpu_cores = system.cpus().len() as u32;
        
        (max_memory_mb, cpu_cores)
    }
    
    /// 检测网络类型
    fn detect_network_type() -> NetworkType {
        platform::detect_network_type()
    }
    
    /// 检测电池状态
    fn detect_battery() -> (Option<f32>, Option<bool>) {
        platform::detect_battery()
    }
}

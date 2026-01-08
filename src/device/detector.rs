use super::capabilities::DeviceCapabilities;
use super::types::{DeviceType, GpuComputeApi, NetworkType};

/// 设备能力检测器（用于未来平台特定实现）
pub struct DeviceDetector;

impl DeviceDetector {
    /// 检测当前设备能力
    pub fn detect() -> DeviceCapabilities {
        // 检测 CPU 架构
        let cpu_architecture = Self::detect_cpu_architecture();
        
        // 检测 GPU API
        let gpu_compute_apis = Self::detect_gpu_apis();
        let has_gpu = !gpu_compute_apis.is_empty();
        
        // 检测 TPU
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

    /// 检测 CPU 架构（增强版 - 获取更详细信息）
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

    /// 检测 GPU 计算 API（平台特定）
    fn detect_gpu_apis() -> Vec<GpuComputeApi> {
        #[cfg(target_os = "windows")]
        {
            super::platform::windows::detect_gpu_apis()
        }
        #[cfg(target_os = "linux")]
        {
            super::platform::linux::detect_gpu_apis()
        }
        #[cfg(target_os = "macos")]
        {
            super::platform::macos::detect_gpu_apis()
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Vec::new()
        }
    }

    /// 检测 TPU/NPU 支持（增强版 - 检测多种 NPU/TPU 设备）
    fn detect_tpu() -> Option<bool> {
        // 检查 TensorFlow TPU 环境变量
        if std::env::var("TPU_CONFIG_GS").is_ok() || std::env::var("TPU_NAME").is_ok() {
            return Some(true);
        }
        
        #[cfg(target_os = "linux")]
        {
            // 检查 Edge TPU 设备
            if std::path::Path::new("/dev/apex_0").exists() {
                return Some(true);
            }
            // 检查 /sys/class/tpu
            if std::fs::read_dir("/sys/class/tpu").is_ok() {
                return Some(true);
            }
            // 检查 Qualcomm NPU (qcom-npu)
            if std::fs::read_dir("/sys/class/qcom-npu").is_ok() {
                return Some(true);
            }
            // 检查华为 NPU (hisi-npu)
            if std::fs::read_dir("/sys/class/hisi-npu").is_ok() {
                return Some(true);
            }
            // 检查 MediaTek NPU
            if std::fs::read_dir("/sys/class/mtk-npu").is_ok() {
                return Some(true);
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // 检查 Apple Neural Engine (ANE)
            use std::process::Command;
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "machdep.cpu.brand_string"])
                .output()
            {
                if let Ok(brand) = String::from_utf8(output.stdout) {
                    // Apple Silicon (M1/M2/M3 等) 通常有 Neural Engine
                    if brand.contains("Apple") && (brand.contains("M1") || brand.contains("M2") || brand.contains("M3")) {
                        return Some(true);
                    }
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // 检查 Windows 上的 NPU 设备
            use std::process::Command;
            if let Ok(output) = Command::new("wmic")
                .args(&["path", "win32_pnpentity", "get", "name", "/format:list"])
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let output_lower = output_str.to_lowercase();
                    if output_lower.contains("npu") || output_lower.contains("neural") || 
                       output_lower.contains("tpu") || output_lower.contains("ai accelerator") {
                        return Some(true);
                    }
                }
            }
        }
        
        None
    }

    /// 检测内存和 CPU 核心数
    fn detect_memory_and_cpu() -> (usize, usize) {
        use sysinfo::{System, SystemExt};
        
        let mut system = System::default();
        system.refresh_all();
        
        let total_memory = system.total_memory() / (1024 * 1024); // 转换为 MB
        let cpu_count = system.cpus().len();
        
        // 如果检测失败，使用默认值
        let memory_mb = if total_memory > 0 {
            total_memory as usize
        } else {
            2048 // 默认值
        };
        
        let cores = if cpu_count > 0 {
            cpu_count
        } else {
            4 // 默认值
        };
        
        (memory_mb, cores)
    }

    /// 检测网络类型（平台特定）
    fn detect_network_type() -> NetworkType {
        // 首先检查环境变量（用于测试）
        if let Ok(net_type) = std::env::var("WILLIW_NETWORK_TYPE") {
            match net_type.as_str() {
                "wifi" => return NetworkType::WiFi,
                "5g" => return NetworkType::Cellular5G,
                "4g" => return NetworkType::Cellular4G,
                _ => {}
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            super::platform::windows::detect_network_type()
        }
        #[cfg(target_os = "linux")]
        {
            super::platform::linux::detect_network_type()
        }
        #[cfg(target_os = "macos")]
        {
            super::platform::macos::detect_network_type()
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            NetworkType::Unknown
        }
    }

    /// 检测电池状态（平台特定）
    fn detect_battery() -> (Option<f32>, bool) {
        // 首先检查环境变量（用于测试）
        if let Ok(level) = std::env::var("WILLIW_BATTERY_LEVEL") {
            if let Ok(level_f) = level.parse::<f32>() {
                let charging = std::env::var("WILLIW_BATTERY_CHARGING")
                    .map(|v| v == "true")
                    .unwrap_or(false);
                return (Some(level_f.clamp(0.0, 1.0)), charging);
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            super::platform::windows::detect_battery()
        }
        #[cfg(target_os = "linux")]
        {
            super::platform::linux::detect_battery()
        }
        #[cfg(target_os = "macos")]
        {
            super::platform::macos::detect_battery()
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            (None, false)
        }
    }
}


use crate::device::types::{GpuComputeApi, NetworkType};
use std::process::Command;

/// 检查库是否存在
fn check_library_exists(lib_name: &str) -> bool {
    use libloading::Library;
    unsafe { Library::new(lib_name).is_ok() }
}

/// 检测 Windows GPU API（增强版 - 真实检测 GPU 设备）
pub fn detect_gpu_apis() -> Vec<GpuComputeApi> {
    let mut apis = Vec::new();
    
    // 方法1: 使用 wmic 命令检测 GPU 设备
    if let Ok(output) = Command::new("wmic")
        .args(&["path", "win32_VideoController", "get", "name", "/format:list"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            let output_lower = output_str.to_lowercase();
            
            // 检测 NVIDIA GPU
            if output_lower.contains("nvidia") {
                // 检查 CUDA 库
                if check_library_exists("nvcuda.dll") {
                    apis.push(GpuComputeApi::CUDA);
                }
            }
            
            // 检测 AMD GPU
            if output_lower.contains("amd") || output_lower.contains("radeon") {
                // AMD GPU 通常支持 Vulkan 和 OpenCL
            }
            
            // 检测 Intel GPU
            if output_lower.contains("intel") {
                // Intel GPU 通常支持 DirectX 和 Vulkan
            }
        }
    }
    
    // 方法2: 检测 DirectX 12（Windows 10+ 通常支持）
    if check_library_exists("dxgi.dll") {
        if check_library_exists("d3d12.dll") {
            apis.push(GpuComputeApi::DirectX);
        }
    }
    
    // 方法3: 检测 Vulkan（通过注册表或库文件）
    if check_library_exists("vulkan-1.dll") {
        apis.push(GpuComputeApi::Vulkan);
    }
    
    // 方法4: 检测 OpenCL
    if check_library_exists("OpenCL.dll") {
        apis.push(GpuComputeApi::OpenCL);
    }
    
    // 去重
    apis.sort();
    apis.dedup();
    apis
}

/// 检测 Windows 网络类型（增强版 - 真实检测网络类型）
pub fn detect_network_type() -> NetworkType {
    // 方法1: 使用 netsh 命令检测 WiFi 连接
    if let Ok(output) = Command::new("netsh")
        .args(&["wlan", "show", "interfaces"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            if output_str.contains("State") && output_str.contains("connected") {
                return NetworkType::WiFi;
            }
        }
    }
    
    // 方法2: 使用 wmic 检测网络适配器类型
    if let Ok(output) = Command::new("wmic")
        .args(&["path", "win32_networkadapter", "where", "netenabled=true", "get", "adaptertype,description", "/format:list"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            let output_lower = output_str.to_lowercase();
            
            // 检测 WiFi (AdapterType = 9 或描述包含 wireless/wifi)
            if output_lower.contains("wireless") || output_lower.contains("wifi") || 
               output_lower.contains("wi-fi") || output_lower.contains("adaptertype=9") {
                return NetworkType::WiFi;
            }
            
            // 检测移动网络 (AdapterType = 20 或描述包含 cellular/mobile)
            if output_lower.contains("cellular") || output_lower.contains("mobile") ||
               output_lower.contains("adaptertype=20") {
                // 尝试检测是 4G 还是 5G
                if output_lower.contains("5g") || output_lower.contains("lte advanced") {
                    return NetworkType::Cellular5G;
                }
                return NetworkType::Cellular4G;
            }
        }
    }
    
    // 方法3: 使用 PowerShell 检测网络连接类型（更准确）
    if let Ok(output) = Command::new("powershell")
        .args(&["-Command", "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Select-Object -ExpandProperty InterfaceDescription"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            let output_lower = output_str.to_lowercase();
            if output_lower.contains("wireless") || output_lower.contains("wifi") || 
               output_lower.contains("wi-fi") || output_lower.contains("802.11") {
                return NetworkType::WiFi;
            }
            if output_lower.contains("cellular") || output_lower.contains("mobile") ||
               output_lower.contains("lte") || output_lower.contains("5g") {
                if output_lower.contains("5g") {
                    return NetworkType::Cellular5G;
                }
                return NetworkType::Cellular4G;
            }
        }
    }
    
    NetworkType::Unknown
}

/// 检测 Windows 电池状态（增强版 - 真实检测电池状态）
pub fn detect_battery() -> (Option<f32>, bool) {
    // 方法1: 使用 battery 库（推荐）
    use battery::Manager;
    
    if let Ok(manager) = Manager::new() {
        if let Ok(batteries) = manager.batteries() {
            for battery_result in batteries {
                if let Ok(battery) = battery_result {
                    let state = battery.state();
                    let is_charging = matches!(
                        state,
                        battery::State::Charging | battery::State::Full
                    );
                    
                    let percentage = battery.state_of_charge();
                    let level = percentage.get::<battery::units::ratio::percent>() as f32 / 100.0;
                    if level >= 0.0 && level <= 1.0 {
                        return (Some(level), is_charging);
                    }
                }
            }
        }
    }
    
    // 方法2: 使用 wmic 命令作为备选
    if let Ok(output) = Command::new("wmic")
        .args(&["path", "win32_battery", "get", "batterystatus,estimatedchargeremaining", "/format:list"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            let mut level: Option<f32> = None;
            let mut is_charging = false;
            
            for line in output_str.lines() {
                if line.starts_with("EstimatedChargeRemaining=") {
                    if let Ok(value) = line.split('=').nth(1).unwrap_or("").trim().parse::<f32>() {
                        level = Some((value / 100.0).clamp(0.0, 1.0));
                    }
                }
                if line.starts_with("BatteryStatus=") {
                    if let Ok(status) = line.split('=').nth(1).unwrap_or("").trim().parse::<u32>() {
                        // BatteryStatus: 2 = Charging, 4 = AC Power (Full)
                        is_charging = status == 2 || status == 4;
                    }
                }
            }
            
            if let Some(level_val) = level {
                return (Some(level_val), is_charging);
            }
        }
    }
    
    // 方法3: 使用 PowerShell 作为最后备选
    if let Ok(output) = Command::new("powershell")
        .args(&["-Command", "Get-WmiObject -Class Win32_Battery | Select-Object -ExpandProperty EstimatedChargeRemaining"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            if let Ok(percentage) = output_str.trim().parse::<f32>() {
                let level = (percentage / 100.0).clamp(0.0, 1.0);
                // 检查是否在充电
                if let Ok(charging_output) = Command::new("powershell")
                    .args(&["-Command", "Get-WmiObject -Class Win32_Battery | Select-Object -ExpandProperty BatteryStatus"])
                    .output()
                {
                    if let Ok(charging_str) = String::from_utf8(charging_output.stdout) {
                        if let Ok(status) = charging_str.trim().parse::<u32>() {
                            let is_charging = status == 2 || status == 4;
                            return (Some(level), is_charging);
                        }
                    }
                }
                return (Some(level), false);
            }
        }
    }
    
    (None, false)
}


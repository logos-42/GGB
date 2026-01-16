//! 设备管理模块
//! 
//! 实现设备检测、配置调整和兼容性验证功能

use williw::device::{DeviceCapabilities, NetworkType};

/// 根据设备能力调整配置
pub fn adjust_config_for_device(
    mut config: williw::config::AppConfig,
    capabilities: &DeviceCapabilities
) -> williw::config::AppConfig {
    // 根据设备能力调整配置
    config.training_config.batch_size = std::cmp::min(
        config.training_config.batch_size,
        (capabilities.max_memory_mb / 1024) as usize * 16  // 每GB内存支持16个批次
    );
    
    // 根据CPU核心数调整并行度
    config.training_config.workers = Some(capabilities.cpu_cores);
    
    // 根据电池状态调整强度
    if let Some(battery_level) = capabilities.battery_level {
        if battery_level < 20.0 {
            config.privacy_config.level = williw::config::PrivacyLevel::High;
            log_d("Android", "🔋 电量低，切换到高隐私模式");
        } else if battery_level < 50.0 {
            config.privacy_config.level = williw::config::PrivacyLevel::Medium;
            log_d("Android", "🔋 电量中等，使用中等隐私模式");
        } else {
            config.privacy_config.level = williw::config::PrivacyLevel::Low;
            log_d("Android", "🔋 电量充足，使用低隐私模式");
        }
    }
    
    // 根据网络类型调整配置
    match capabilities.network_type {
        NetworkType::Cellular4G => {
            config.network_config.max_peers = std::cmp::min(config.network_config.max_peers, 5);
            log_d("Android", "📶 4G网络，限制连接数到5");
        }
        NetworkType::Cellular5G => {
            config.network_config.max_peers = std::cmp::min(config.network_config.max_peers, 8);
            log_d("Android", "📶 5G网络，限制连接数到8");
        }
        NetworkType::WiFi => {
            log_d("Android", "📶 WiFi网络，不限制连接数");
        }
        NetworkType::Unknown => {
            config.network_config.max_peers = std::cmp::min(config.network_config.max_peers, 3);
            log_d("Android", "📶 未知网络，保守限制连接数到3");
        }
    }
    
    // 根据设备类型调整配置
    match capabilities.device_type {
        williw::device::DeviceType::Phone => {
            config.device_config.max_memory_gb = std::cmp::min(config.device_config.max_memory_gb, 2.0);
            log_d("Android", "📱 手机设备，限制最大内存到2GB");
        }
        williw::device::DeviceType::Tablet => {
            config.device_config.max_memory_gb = std::cmp::min(config.device_config.max_memory_gb, 4.0);
            log_d("Android", "📱 平板设备，限制最大内存到4GB");
        }
        williw::device::DeviceType::Desktop => {
            log_d("Android", "🖥️ 桌面设备，使用完整内存配置");
        }
        _ => {}
    }
    
    // GPU配置调整
    if capabilities.gpu_compute_apis.is_empty() {
        config.device_config.use_gpu = false;
        log_d("Android", "🎮 未检测到GPU，禁用GPU加速");
    } else {
        log_d("Android", &format!("🎮 检测到GPU: {:?}", capabilities.gpu_compute_apis));
    }
    
    config
}

/// 检查模型兼容性
pub fn is_model_compatible(
    model: &crate::ModelConfig,
    capabilities: &DeviceCapabilities
) -> bool {
    log_d("Android", &format!("🔍 检查模型兼容性: {}", model.name));
    
    // 检查内存需求
    let required_memory_gb = (model.dimensions * model.batch_size * 4) as f64 / (1024.0 * 1024.0 * 1024.0);
    let available_memory_gb = capabilities.max_memory_mb as f64 / 1024.0;
    
    if required_memory_gb > available_memory_gb {
        log_d("Android", &format!("❌ 内存不足: 需要{:.2}GB, 可用{:.2}GB", 
            required_memory_gb, available_memory_gb));
        return false;
    }
    
    // 检查CPU要求
    let recommended_batch_size = capabilities.cpu_cores as usize * 4;
    if model.batch_size > recommended_batch_size {
        log_d("Android", &format!("❌ CPU不足: 批次大小{}, 推荐{}", 
            model.batch_size, recommended_batch_size));
        return false;
    }
    
    // 检查存储空间（假设）
    let required_storage_gb = required_memory_gb * 2.0; // 模型+数据
    if required_storage_gb > 8.0 { // 假设8GB可用存储
        log_d("Android", &format!("❌ 存储不足: 需要{:.2}GB", required_storage_gb));
        return false;
    }
    
    // 检查网络要求
    match capabilities.network_type {
        NetworkType::Unknown => {
            if model.dimensions > 1000 { // 大模型需要良好网络
                log_d("Android", "❌ 网络未知，不支持大模型");
                return false;
            }
        }
        _ => {} // 网络已知，支持
    }
    
    log_d("Android", "✅ 模型兼容性检查通过");
    true
}

/// 获取设备性能评分
pub fn get_performance_score(capabilities: &DeviceCapabilities) -> f64 {
    let mut score = 0.0;
    
    // CPU评分 (0-30分)
    score += (capabilities.cpu_cores as f64 / 8.0) * 30.0;
    
    // 内存评分 (0-25分)
    let memory_gb = capabilities.max_memory_mb as f64 / 1024.0;
    score += (memory_gb / 8.0).min(1.0) * 25.0;
    
    // GPU评分 (0-20分)
    if !capabilities.gpu_compute_apis.is_empty() {
        score += 20.0;
    }
    
    // 网络评分 (0-15分)
    match capabilities.network_type {
        NetworkType::WiFi => score += 15.0,
        NetworkType::Cellular5G => score += 12.0,
        NetworkType::Cellular4G => score += 8.0,
        NetworkType::Unknown => score += 0.0,
    }
    
    // 电池评分 (0-10分)
    if let Some(battery_level) = capabilities.battery_level {
        score += (battery_level / 100.0) * 10.0;
    }
    
    score.round()
}

/// 获取设备建议
pub fn get_device_recommendations(capabilities: &DeviceCapabilities) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // 基于CPU核心数的建议
    if capabilities.cpu_cores < 4 {
        recommendations.push("建议升级设备以获得更好的训练性能".to_string());
    }
    
    // 基于内存的建议
    let memory_gb = capabilities.max_memory_mb as f64 / 1024.0;
    if memory_gb < 2.0 {
        recommendations.push("建议增加内存以支持更大的模型".to_string());
    }
    
    // 基于GPU的建议
    if capabilities.gpu_compute_apis.is_empty() {
        recommendations.push("建议使用支持GPU加速的设备以提升训练速度".to_string());
    }
    
    // 基于网络的建议
    match capabilities.network_type {
        NetworkType::Cellular4G => {
            recommendations.push("建议使用WiFi网络以获得更好的训练体验".to_string());
        }
        NetworkType::Unknown => {
            recommendations.push("请检查网络连接状态".to_string());
        }
        _ => {}
    }
    
    // 基于电池的建议
    if let Some(battery_level) = capabilities.battery_level {
        if battery_level < 30.0 {
            recommendations.push("建议连接充电器以进行长时间训练".to_string());
        }
    }
    
    recommendations
}

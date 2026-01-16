//! 日志模块
//! 
//! 统一的Android日志记录功能

/// Android调试日志
pub fn log_d(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    android_log::d(tag, message);
    
    #[cfg(not(target_os = "android"))]
    println!("[{}] [DEBUG] {}", tag, message);
}

/// Android错误日志
pub fn log_e(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    android_log::e(tag, message);
    
    #[cfg(not(target_os = "android"))]
    eprintln!("[{}] [ERROR] {}", tag, message);
}

/// Android信息日志
pub fn log_i(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    android_log::i(tag, message);
    
    #[cfg(not(target_os = "android"))]
    println!("[{}] [INFO] {}", tag, message);
}

/// Android警告日志
pub fn log_w(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    android_log::w(tag, message);
    
    #[cfg(not(target_os = "android"))]
    println!("[{}] [WARN] {}", tag, message);
}

/// 性能日志（带时间戳）
pub fn log_perf(tag: &str, operation: &str, duration_ms: u64) {
    let message = format!("⏱️ {} 耗时: {}ms", operation, duration_ms);
    log_d(tag, &message);
}

/// 设备信息日志
pub fn log_device(tag: &str, device_info: &str) {
    let message = format!("📱 {}", device_info);
    log_i(tag, &message);
}

/// 训练进度日志
pub fn log_training_progress(tag: &str, epoch: u32, total: u32, accuracy: f64) {
    let progress = (epoch as f64 / total as f64) * 100.0;
    let message = format!("📊 训练进度: {}/{} ({:.1}%, 准确率: {:.4})", 
        epoch, total, progress, accuracy);
    log_i(tag, &message);
}

/// 网络状态日志
pub fn log_network(tag: &str, network_type: &str, status: &str) {
    let message = format!("📶 网络: {} - {}", network_type, status);
    log_i(tag, &message);
}

/// 电池状态日志
pub fn log_battery(tag: &str, level: f64, is_charging: bool) {
    let status = if is_charging { "充电中" } else { "使用中" };
    let message = format!("🔋 电池: {:.1}% ({})", level, status);
    log_i(tag, &message);
}

/// 模型加载日志
pub fn log_model_load(tag: &str, model_name: &str, success: bool) {
    let status = if success { "✅ 成功" } else { "❌ 失败" };
    let message = format!("🤖 模型加载: {} - {}", model_name, status);
    log_i(tag, &message);
}

/// JNI调用日志
pub fn log_jni_call(tag: &str, method: &str, success: bool) {
    let status = if success { "✅ 成功" } else { "❌ 失败" };
    let message = format!("🔗 JNI调用: {} - {}", method, status);
    log_d(tag, &message);
}

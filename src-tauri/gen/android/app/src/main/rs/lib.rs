//! Williw Android JNI实现
//! 
//! 连接Android Java层和Rust核心，实现桌面版的所有功能
//! 
//! 模块化架构：
//! - training: 训练控制功能
//! - model: 模型管理功能  
//! - device: 设备管理功能
//! - state: 状态管理功能
//! - logger: 日志记录功能

use jni::{JNIEnv, objects::{JClass, JObject, JString}};
use jni::sys::{jboolean, jstring};
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json;

// 导入williw核心功能
use williw::device::DeviceManager;
use williw::config::AppConfig;

// 导入模块
mod training;
mod model;
mod device;
mod state;
mod logger;
mod network;

// 导入类型和函数
use model::{initialize_model_registry, select_model_internal, get_available_models};
use device::{adjust_config_for_device, is_model_compatible, get_performance_score};
use state::{TrainingState, ModelConfig, ApiKeyEntry, AppSettings};
use logger::{log_d, log_e, log_i, log_w, log_jni_call};

// 全局状态存储
lazy_static::lazy_static! {
    static ref TRAINING_STATE: Arc<Mutex<TrainingState>> = Arc::new(Mutex::new(TrainingState::new()));
    static ref MODEL_REGISTRY: Arc<Mutex<HashMap<String, ModelConfig>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref DEVICE_MANAGER: Arc<Mutex<DeviceManager>> = Arc::new(Mutex::new(DeviceManager::new()));
    static ref API_KEYS: Arc<Mutex<Vec<ApiKeyEntry>>> = Arc::new(Mutex::new(Vec::new()));
}

/// 训练控制 - 启动训练
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_startTraining(
    env: JNIEnv,
    _class: JClass,
) -> jboolean {
    log_jni_call("Android", "startTraining", false);

    // 创建一个临时的tokio运行时来运行异步函数
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        training::start_training_internal().await
    });

    match result {
        Ok(()) => {
            log_jni_call("Android", "startTraining", true);
            jni::sys::JNI_TRUE
        }
        Err(e) => {
            log_e("Android", &format!("训练启动失败: {}", e));
            jni::sys::JNI_FALSE
        }
    }
}

/// 训练控制 - 停止训练
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_stopTraining(
    env: JNIEnv,
    _class: JClass,
) -> jboolean {
    log_jni_call("Android", "stopTraining", false);

    match training::stop_training_internal() {
        Ok(()) => {
            log_jni_call("Android", "stopTraining", true);
            jni::sys::JNI_TRUE
        }
        Err(e) => {
            log_e("Android", &format!("训练停止失败: {}", e));
            jni::sys::JNI_FALSE
        }
    }
}

/// 获取训练状态
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_getTrainingStatus(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    log_jni_call("Android", "getTrainingStatus", false);

    let status_str = training::get_training_status();
    match CString::new(status_str) {
        Ok(s) => {
            log_jni_call("Android", "getTrainingStatus", true);
            s.into_raw()
        }
        Err(_) => {
            log_jni_call("Android", "getTrainingStatus", false);
            ptr::null_mut()
        }
    }
}

/// 选择模型
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_selectModel(
    env: JNIEnv,
    _class: JClass,
    model_id: JString,
) -> jboolean {
    let model_id_str = match env.get_string(model_id) {
        Ok(s) => s,
        Err(_) => {
            log_e("Android", "模型ID参数无效");
            return jni::sys::JNI_FALSE;
        }
    };
    
    let model_id_safe = match CStr::from_ptr(model_id_str).to_str() {
        Ok(s) => s,
        Err(_) => {
            log_e("Android", "模型ID转换失败");
            return jni::sys::JNI_FALSE;
        }
    };
    
    log_jni_call("Android", "selectModel", false);
    
    match select_model_internal(model_id_safe) {
        Ok(()) => {
            log_jni_call("Android", "selectModel", true);
            jni::sys::JNI_TRUE
        }
        Err(e) => {
            log_e("Android", &format!("模型选择失败: {}", e));
            jni::sys::JNI_FALSE
        }
    }
}

/// 获取可用模型列表
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_getAvailableModels(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    log_jni_call("Android", "getAvailableModels", false);
    
    let models_str = get_available_models();
            
            registry.insert("llama2-7b".to_string(), ModelConfig {
                id: "llama2-7b".to_string(),
                name: "LLaMA 2 7B".to_string(),
                description: "Meta LLaMA 2 7B parameter model for text generation".to_string(),
                dimensions: 4096,
                learning_rate: 1e-5,
                batch_size: 8,
            });
            
            registry.insert("resnet50".to_string(), ModelConfig {
                id: "resnet50".to_string(),
                name: "ResNet-50".to_string(),
                description: "Microsoft ResNet-50 for image classification with 50 layers".to_string(),
                dimensions: 2048,
                learning_rate: 0.1,
                batch_size: 64,
            });
            
            registry.insert("stable-diffusion-v1-5".to_string(), ModelConfig {
                id: "stable-diffusion-v1-5".to_string(),
                name: "Stable Diffusion 1.5".to_string(),
                description: "Stability AI text-to-image model with CLIP text encoder".to_string(),
                dimensions: 768,
                learning_rate: 1e-4,
                batch_size: 4,
            });
            
            registry.insert("whisper-medium".to_string(), ModelConfig {
                id: "whisper-medium".to_string(),
                name: "Whisper Medium".to_string(),
                description: "OpenAI Whisper medium model for speech recognition".to_string(),
                dimensions: 1024,
                learning_rate: 1e-4,
                batch_size: 16,
            });
            
            registry.insert("t5-base".to_string(), ModelConfig {
                id: "t5-base".to_string(),
                name: "T5 Base".to_string(),
                description: "Google T5 (Text-to-Text Transfer Transformer) 220M parameters".to_string(),
                dimensions: 768,
                learning_rate: 3e-4,
                batch_size: 32,
            });
        }
    }
    
    // 获取模型列表
    let models = {
        let registry = MODEL_REGISTRY.lock().unwrap();
        let models: Vec<&ModelConfig> = registry.values().collect();
        serde_json::json!(models)
    };
    
    let models_str = models.to_string();
    match CString::new(models_str) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 获取设备信息
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_getDeviceInfo(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    log_jni_call("Android", "getDeviceInfo", false);
    
    let device_manager = DEVICE_MANAGER.lock().unwrap();
    let capabilities = device_manager.get();
    
    let device_info = serde_json::json!({
        "gpu_type": capabilities.gpu_compute_apis.first().map(|api| format!("{:?}", api)),
        "gpu_usage": None::<f64>, // Android GPU使用率检测较复杂
        "gpu_memory_total": None::<f64>,
        "gpu_memory_used": None::<f64>,
        "cpu_cores": capabilities.cpu_cores,
        "total_memory_gb": capabilities.max_memory_mb as f64 / 1024.0,
        "battery_level": capabilities.battery_level,
        "is_charging": capabilities.is_charging,
        "device_type": format!("{:?}", capabilities.device_type),
        "network_type": format!("{:?}", capabilities.network_type),
        "performance_score": get_performance_score(&capabilities)
    });
    
    let info_str = device_info.to_string();
    match CString::new(info_str) {
        Ok(s) => {
            log_jni_call("Android", "getDeviceInfo", true);
            s.into_raw()
        }
        Err(_) => {
            log_jni_call("Android", "getDeviceInfo", false);
            ptr::null_mut()
        }
    }
}

/// 获取电池状态
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_getBatteryStatus(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    log_jni_call("Android", "getBatteryStatus", false);
    
    let device_manager = DEVICE_MANAGER.lock().unwrap();
    let capabilities = device_manager.get();
    
    let battery_info = serde_json::json!({
        "level": capabilities.battery_level,
        "is_charging": capabilities.is_charging,
        "status": capabilities.battery_status()
    });
    
    let battery_str = battery_info.to_string();
    match CString::new(battery_str) {
        Ok(s) => {
            log_jni_call("Android", "getBatteryStatus", true);
            s.into_raw()
        }
        Err(_) => {
            log_jni_call("Android", "getBatteryStatus", false);
            ptr::null_mut()
        }
    }
}

/// 获取网络类型
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_getNetworkType(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    log_jni_call("Android", "getNetworkType", false);
    
    let device_manager = DEVICE_MANAGER.lock().unwrap();
    let capabilities = device_manager.get();
    
    let network_info = serde_json::json!({
        "type": format!("{:?}", capabilities.network_type),
        "bandwidth_factor": capabilities.network_type.bandwidth_factor(),
        "allows_dense_snapshot": capabilities.network_type.allows_dense_snapshot()
    });
    
    let network_str = network_info.to_string();
    match CString::new(network_str) {
        Ok(s) => {
            log_jni_call("Android", "getNetworkType", true);
            s.into_raw()
        }
        Err(_) => {
            log_jni_call("Android", "getNetworkType", false);
            ptr::null_mut()
        }
    }
}

/// 刷新设备信息
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_refreshDeviceInfo(
    env: JNIEnv,
    _class: JClass,
) -> jboolean {
    log_jni_call("Android", "refreshDeviceInfo", false);
    
    let device_manager = DEVICE_MANAGER.lock().unwrap();
    device_manager.refresh();
    
    log_jni_call("Android", "refreshDeviceInfo", true);
    jni::sys::JNI_TRUE
}

/// 更新设置
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_updateSettings(
    env: JNIEnv,
    _class: JClass,
    settings_json: JString,
) -> jboolean {
    let settings_str = match env.get_string(settings_json) {
        Ok(s) => s,
        Err(_) => return jni::sys::JNI_FALSE,
    };
    
    let settings_safe = match std::ffi::CStr::from_ptr(settings_str).to_str() {
        Ok(s) => s,
        Err(_) => return jni::sys::JNI_FALSE,
    };
    
    log_d("Android", &format!("更新设置: {}", settings_safe));
    
    // 这里可以实现设置更新逻辑
    // 目前返回成功
    jni::sys::JNI_TRUE
}

/// 获取设置
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_getSettings(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let settings = serde_json::json!({
        "privacy_level": "medium",
        "bandwidth_budget": 10,
        "network_config": {
            "max_peers": 10,
            "bootstrap_nodes": [],
            "port": 9000
        },
        "checkpoint_settings": {
            "enabled": true,
            "interval_minutes": 5,
            "max_checkpoints": 10
        }
    });
    
    let settings_str = settings.to_string();
    match CString::new(settings_str) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// ========== API密钥管理 ==========

/// 创建API密钥
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_createApiKey(
    env: JNIEnv,
    _class: JClass,
    name: JString,
) -> jstring {
    let name_str = match env.get_string(name) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let name_safe = match std::ffi::CStr::from_ptr(name_str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let new_key = serde_json::json!({
        "id": format!("sk-williw-{}", uuid::Uuid::new_v4()),
        "name": name_safe,
        "key": format!("mock-key-for-{}", name_safe),
        "created_at": chrono::Utc::now().to_rfc3339()
    });
    
    let key_str = new_key.to_string();
    match CString::new(key_str) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 删除API密钥
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_deleteApiKey(
    env: JNIEnv,
    _class: JClass,
    key_id: JString,
) -> jboolean {
    let key_id_str = match env.get_string(key_id) {
        Ok(s) => s,
        Err(_) => return jni::sys::JNI_FALSE,
    };
    
    let key_id_safe = match std::ffi::CStr::from_ptr(key_id_str).to_str() {
        Ok(s) => s,
        Err(_) => return jni::sys::JNI_FALSE,
    };
    
    log_d("Android", &format!("删除API密钥: {}", key_id_safe));
    
    // 这里可以实现删除逻辑
    jni::sys::JNI_TRUE
}

/// 获取所有API密钥
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_getApiKeys(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let keys = serde_json::json!([]);
    
    let keys_str = keys.to_string();
    match CString::new(keys_str) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 更新API密钥名称
#[no_mangle]
pub extern "C" fn Java_com_williw_mobile_WilliwJNI_updateApiKeyName(
    env: JNIEnv,
    _class: JClass,
    key_id: JString,
    new_name: JString,
) -> jboolean {
    let key_id_str = match env.get_string(key_id) {
        Ok(s) => s,
        Err(_) => return jni::sys::JNI_FALSE,
    };
    
    let new_name_str = match env.get_string(new_name) {
        Ok(s) => s,
        Err(_) => return jni::sys::JNI_FALSE,
    };
    
    let key_id_safe = match std::ffi::CStr::from_ptr(key_id_str).to_str() {
        Ok(s) => s,
        Err(_) => return jni::sys::JNI_FALSE,
    };
    
    let new_name_safe = match std::ffi::CStr::from_ptr(new_name_str).to_str() {
        Ok(s) => s,
        Err(_) => return jni::sys::JNI_FALSE,
    };
    
    log_d("Android", &format!("更新API密钥: {} -> {}", key_id_safe, new_name_safe));
    
    jni::sys::JNI_TRUE
}

// ========== 内部辅助函数 ==========

// 全局状态存储
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref TRAINING_STATE: Arc<Mutex<TrainingState>> = Arc::new(Mutex::new(TrainingState::new()));
    static ref MODEL_REGISTRY: Arc<Mutex<HashMap<String, ModelConfig>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref DEVICE_MANAGER: Arc<Mutex<DeviceManager>> = Arc::new(Mutex::new(DeviceManager::new()));
}

#[derive(Debug, Clone)]
struct TrainingState {
    is_running: bool,
    current_epoch: u32,
    total_epochs: u32,
    accuracy: f64,
    loss: f64,
    samples_processed: u64,
    current_model: String,
}

impl TrainingState {
    fn new() -> Self {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelConfig {
    id: String,
    name: String,
    description: String,
    dimensions: usize,
    learning_rate: f64,
    batch_size: usize,
}



fn select_model_internal(model_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_d("Android", &format!("🔄 选择模型: {}", model_id));
    
    // 1. 从注册表获取模型配置
    let model_config = {
        let registry = MODEL_REGISTRY.lock().unwrap();
        registry.get(model_id).cloned()
    };
    
    let model = model_config.ok_or_else(|| {
        format!("模型 '{}' 未找到", model_id)
    })?;
    
    // 2. 验证模型兼容性
    let device_manager = DEVICE_MANAGER.lock().unwrap();
    let capabilities = device_manager.get();
    
    if !is_model_compatible(&model, &capabilities) {
        return Err(format!("模型 '{}' 与当前设备不兼容", model_id).into());
    }
    
    // 3. 更新当前模型
    {
        let mut state = TRAINING_STATE.lock().unwrap();
        state.current_model = model_id.to_string();
    }
    
    log_d("Android", &format!("✅ 模型选择成功: {} ({}维)", model.name, model.dimensions));
    Ok(())
}

fn adjust_config_for_device(
    mut config: williw::config::AppConfig,
    capabilities: &williw::device::DeviceCapabilities
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
        }
    }
    
    // 根据网络类型调整配置
    match capabilities.network_type {
        williw::device::NetworkType::Cellular4G => {
            config.network_config.max_peers = std::cmp::min(config.network_config.max_peers, 5);
            log_d("Android", "📶 4G网络，限制连接数");
        }
        williw::device::NetworkType::Cellular5G => {
            config.network_config.max_peers = std::cmp::min(config.network_config.max_peers, 8);
            log_d("Android", "📶 5G网络，适度限制连接数");
        }
        _ => {} // WiFi不限制
    }
    
    config
}

fn is_model_compatible(
    model: &ModelConfig,
    capabilities: &williw::device::DeviceCapabilities
) -> bool {
    // 检查内存需求
    let required_memory_gb = (model.dimensions * model.batch_size * 4) as f64 / (1024.0 * 1024.0 * 1024.0);
    if required_memory_gb > capabilities.max_memory_mb as f64 / 1024.0 {
        log_d("Android", &format!("❌ 内存不足: 需要{}GB, 可用{}GB", 
            required_memory_gb, capabilities.max_memory_mb / 1024));
        return false;
    }
    
    // 检查CPU要求
    if model.batch_size > capabilities.cpu_cores as usize * 4 {
        log_d("Android", &format!("❌ CPU不足: 批次大小{}, 核心{}", 
            model.batch_size, capabilities.cpu_cores));
        return false;
    }
    
    true
}

// ========== 日志函数 ==========

fn log_d(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    android_log::d(tag, message);
    
    #[cfg(not(target_os = "android"))]
    println!("[{}] {}", tag, message);
}

fn log_e(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    android_log::e(tag, message);
    
    #[cfg(not(target_os = "android"))]
    eprintln!("[{}] ERROR: {}", tag, message);
}

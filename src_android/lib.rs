//! Android专用库 - 简化版本用于JNI集成
//! 避免复杂的依赖冲突，专注于设备检测功能

use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject, JValue};
use jni::sys::{jlong, jint, jboolean, jfloat, jstring, jobject};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use android_log::AndroidLogExt;

// 设备能力结构体
#[derive(Debug, serde::Serialize)]
pub struct DeviceCapabilities {
    pub max_memory_mb: u64,
    pub cpu_cores: u32,
    pub has_gpu: bool,
    pub cpu_architecture: String,
    pub gpu_compute_apis: Vec<String>,
    pub has_tpu: Option<bool>,
    pub network_type: String,
    pub battery_level: Option<f32>,
    pub is_charging: Option<bool>,
    pub device_type: String,
    pub device_brand: String,
    pub recommended_model_dim: u32,
    pub recommended_tick_interval: u64,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            max_memory_mb: 2048,
            cpu_cores: 4,
            has_gpu: true,
            cpu_architecture: "unknown".to_string(),
            gpu_compute_apis: vec!["opengl_es".to_string()],
            has_tpu: Some(false),
            network_type: "unknown".to_string(),
            battery_level: None,
            is_charging: None,
            device_type: "android".to_string(),
            device_brand: "unknown".to_string(),
            recommended_model_dim: 256,
            recommended_tick_interval: 10,
        }
    }
}

// 节点句柄
#[derive(Debug)]
pub struct NodeHandle {
    device_info: DeviceCapabilities,
    device_callback: Option<DeviceInfoCallback>,
}

// 设备信息回调类型
pub type DeviceInfoCallback = unsafe extern "C" fn(
    memory_mb: *mut u32,
    cpu_cores: *mut u32,
    network_type: *mut c_char,
    network_type_len: usize,
    battery_level: *mut f32,
    is_charging: *mut jint,
) -> jint;

// 错误类型
#[repr(i32)]
pub enum FfiError {
    Success = 0,
    InvalidArgument = -1,
    NullPointer = -2,
    InternalError = -3,
}

// JNI辅助函数
fn rust_string_to_jstring(env: &JNIEnv, rust_string: &str) -> Result<jstring, jni::errors::Error> {
    let c_string = CString::new(rust_string).map_err(|_| {
        jni::errors::Error::ThrowFailed(jni::errors::JniError::UnknownError("Failed to create CString".to_string()))
    })?;
    
    Ok(env.get_string_field(&JObject::from(c_string.as_ptr()), "value")?)
}

fn jstring_to_rust_string(env: &JNIEnv, j_string: JString) -> Result<String, jni::errors::Error> {
    let java_string: String = env.get_string(j_string.into())?.into();
    Ok(java_string)
}

// 创建节点
#[no_mangle]
pub unsafe extern "C" fn williw_node_create() -> *mut NodeHandle {
    log::info!("Creating Williw node");
    
    let handle = Box::new(NodeHandle {
        device_info: DeviceCapabilities::default(),
        device_callback: None,
    });
    
    Box::into_raw(handle)
}

// 销毁节点
#[no_mangle]
pub unsafe extern "C" fn williw_node_destroy(ptr: *mut NodeHandle) {
    if ptr.is_null() {
        return;
    }
    
    log::info!("Destroying Williw node");
    let _handle = Box::from_raw(ptr);
}

// 获取设备能力
#[no_mangle]
pub unsafe extern "C" fn williw_node_get_capabilities(
    ptr: *const NodeHandle,
) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    
    let handle = &*ptr;
    
    match serde_json::to_string(&handle.device_info) {
        Ok(json) => {
            match CString::new(json) {
                Ok(c_str) => c_str.into_raw(),
                Err(_) => ptr::null_mut(),
            }
        }
        Err(_) => ptr::null_mut(),
    }
}

// 设置设备回调
#[no_mangle]
pub unsafe extern "C" fn williw_node_set_device_callback(
    ptr: *mut NodeHandle,
    callback: Option<DeviceInfoCallback>,
) -> jint {
    if ptr.is_null() {
        return FfiError::InvalidArgument as jint;
    }
    
    let handle = &mut *ptr;
    *handle.device_callback.write() = callback;
    FfiError::Success as jint
}

// 更新网络类型
#[no_mangle]
pub unsafe extern "C" fn williw_node_update_network_type(
    ptr: *mut NodeHandle,
    network_type: *const c_char,
) -> jint {
    if ptr.is_null() || network_type.is_null() {
        return FfiError::InvalidArgument as jint;
    }
    
    let handle = &mut *ptr;
    
    if let Ok(net_str) = CStr::from_ptr(network_type).to_str() {
        handle.device_info.network_type = net_str.to_string();
        log::info!("Updated network type: {}", net_str);
        FfiError::Success as jint
    } else {
        FfiError::InvalidArgument as jint
    }
}

// 更新电池信息
#[no_mangle]
pub unsafe extern "C" fn williw_node_update_battery(
    ptr: *mut NodeHandle,
    battery_level: f32,
    is_charging: jboolean,
) -> jint {
    if ptr.is_null() {
        return FfiError::InvalidArgument as jint;
    }
    
    let handle = &mut *ptr;
    
    if battery_level >= 0.0 && battery_level <= 1.0 {
        handle.device_info.battery_level = Some(battery_level);
    } else {
        handle.device_info.battery_level = None;
    }
    
    handle.device_info.is_charging = Some(is_charging != 0);
    
    log::info!("Updated battery: level={:?}, charging={:?}", 
               handle.device_info.battery_level, 
               handle.device_info.is_charging);
    
    FfiError::Success as jint
}

// 更新硬件信息
#[no_mangle]
pub unsafe extern "C" fn williw_node_update_hardware(
    ptr: *mut NodeHandle,
    memory_mb: u32,
    cpu_cores: u32,
) -> jint {
    if ptr.is_null() {
        return FfiError::InvalidArgument as jint;
    }
    
    let handle = &mut *ptr;
    handle.device_info.max_memory_mb = memory_mb as u64;
    handle.device_info.cpu_cores = cpu_cores;
    
    log::info!("Updated hardware: memory={}MB, cores={}", memory_mb, cpu_cores);
    
    FfiError::Success as jint
}

// 获取推荐模型维度
#[no_mangle]
pub unsafe extern "C" fn williw_node_get_recommended_model_dim(
    ptr: *const NodeHandle,
) -> jint {
    if ptr.is_null() {
        return 256; // 默认值
    }
    
    let handle = &*ptr;
    handle.device_info.recommended_model_dim as jint
}

// 获取推荐tick间隔
#[no_mangle]
pub unsafe extern "C" fn williw_node_get_recommended_tick_interval(
    ptr: *const NodeHandle,
) -> jlong {
    if ptr.is_null() {
        return 10; // 默认10秒
    }
    
    let handle = &*ptr;
    handle.device_info.recommended_tick_interval as jlong
}

// 是否应该暂停训练
#[no_mangle]
pub unsafe extern "C" fn williw_node_should_pause_training(
    ptr: *const NodeHandle,
) -> jboolean {
    if ptr.is_null() {
        return 0;
    }
    
    let handle = &*ptr;
    
    // 如果电量低于20%且不在充电，建议暂停
    if let Some(battery_level) = handle.device_info.battery_level {
        if battery_level < 0.2 && handle.device_info.is_charging == Some(false) {
            return 1; // true
        }
    }
    
    0 // false
}

// JNI导出函数
#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeCreate(
    env: JNIEnv,
    _class: JClass,
) -> jlong {
    let ptr = williw_node_create();
    ptr as jlong
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    williw_node_destroy(ptr as *mut NodeHandle);
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeGetCapabilities(
    env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jstring {
    let json_ptr = williw_node_get_capabilities(ptr as *const NodeHandle);
    if json_ptr.is_null() {
        return ptr::null_mut();
    }
    
    let json_str = CStr::from_ptr(json_ptr).to_str().unwrap_or("{}");
    let result = env.new_string(json_str).unwrap_or(JObject::null()).into_raw() as jstring;
    
    // 释放C字符串
    let _ = CString::from_raw(json_ptr);
    
    result
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeUpdateNetworkType(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    network_type: JString,
) -> jint {
    let net_str = match jstring_to_rust_string(&_env, network_type) {
        Ok(s) => s,
        Err(_) => return FfiError::InvalidArgument as jint,
    };
    
    let c_str = match CString::new(net_str) {
        Ok(s) => s,
        Err(_) => return FfiError::InvalidArgument as jint,
    };
    
    williw_node_update_network_type(ptr as *mut NodeHandle, c_str.as_ptr())
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeUpdateBattery(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    battery_level: jfloat,
    is_charging: jboolean,
) -> jint {
    williw_node_update_battery(ptr as *mut NodeHandle, battery_level, is_charging)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeUpdateHardware(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    memory_mb: jint,
    cpu_cores: jint,
) -> jint {
    williw_node_update_hardware(ptr as *mut NodeHandle, memory_mb as u32, cpu_cores as u32)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeGetRecommendedModelDim(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jint {
    williw_node_get_recommended_model_dim(ptr as *const NodeHandle)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeGetRecommendedTickInterval(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jlong {
    williw_node_get_recommended_tick_interval(ptr as *const NodeHandle)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_williw_mobile_WilliwNode_nativeShouldPauseTraining(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jboolean {
    williw_node_should_pause_training(ptr as *const NodeHandle)
}

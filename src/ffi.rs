//! FFI 接口模块，用于移动端（Android/iOS）集成
//!
//! 这个模块提供了 C 兼容的 FFI 接口，允许移动端应用调用 Rust 核心功能

use crate::device::{DeviceCapabilities, DeviceManager, NetworkType};
use crate::types::GgsMessage;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::Arc;
use std::sync::Mutex;

/// FFI 错误代码
#[repr(C)]
pub enum FfiError {
    Success = 0,
    InvalidArgument = 1,
    OutOfMemory = 2,
    NetworkError = 3,
    Unknown = 99,
}

/// 节点句柄（不透明指针）
pub struct NodeHandle {
    // 这里可以存储实际的 Node 实例
    // 为了简化，暂时只存储设备管理器
    device_manager: DeviceManager,
}

/// 创建新的节点实例
///
/// # Safety
/// 返回的指针必须通过 `ggs_node_destroy` 释放
#[no_mangle]
pub unsafe extern "C" fn ggs_node_create() -> *mut NodeHandle {
    let device_manager = DeviceManager::new();
    let handle = Box::new(NodeHandle { device_manager });
    Box::into_raw(handle)
}

/// 销毁节点实例
///
/// # Safety
/// ptr 必须是通过 `ggs_node_create` 创建的有效指针
#[no_mangle]
pub unsafe extern "C" fn ggs_node_destroy(ptr: *mut NodeHandle) {
    if !ptr.is_null() {
        let _ = Box::from_raw(ptr);
    }
}

/// 获取设备能力信息（JSON 格式）
///
/// # Safety
/// ptr 必须是有效的节点句柄
/// 返回的字符串必须通过 `ggs_string_free` 释放
#[no_mangle]
pub unsafe extern "C" fn ggs_node_get_capabilities(
    ptr: *const NodeHandle,
) -> *mut c_char {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    
    let handle = &*ptr;
    let caps = handle.device_manager.get();
    
    // 序列化为 JSON
    match serde_json::to_string(&caps) {
        Ok(json) => {
            match CString::new(json) {
                Ok(c_str) => c_str.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// 更新网络类型
///
/// # Safety
/// ptr 必须是有效的节点句柄
/// network_type_str 必须是有效的 C 字符串
#[no_mangle]
pub unsafe extern "C" fn ggs_node_update_network_type(
    ptr: *mut NodeHandle,
    network_type_str: *const c_char,
) -> c_int {
    if ptr.is_null() || network_type_str.is_null() {
        return FfiError::InvalidArgument as c_int;
    }
    
    let handle = &mut *ptr;
    let network_type = match CStr::from_ptr(network_type_str).to_str() {
        Ok(s) => match s {
            "wifi" => NetworkType::WiFi,
            "5g" => NetworkType::Cellular5G,
            "4g" => NetworkType::Cellular4G,
            _ => NetworkType::Unknown,
        },
        Err(_) => return FfiError::InvalidArgument as c_int,
    };
    
    handle.device_manager.update_network_type(network_type);
    FfiError::Success as c_int
}

/// 更新电池状态
///
/// # Safety
/// ptr 必须是有效的节点句柄
#[no_mangle]
pub unsafe extern "C" fn ggs_node_update_battery(
    ptr: *mut NodeHandle,
    level: f32,      // 0.0-1.0
    is_charging: c_int, // 0 = false, 1 = true
) -> c_int {
    if ptr.is_null() {
        return FfiError::InvalidArgument as c_int;
    }
    
    let handle = &mut *ptr;
    let level_opt = if level >= 0.0 && level <= 1.0 {
        Some(level)
    } else {
        None
    };
    let charging = is_charging != 0;
    
    handle.device_manager.update_battery(level_opt, charging);
    FfiError::Success as c_int
}

/// 释放由 FFI 函数返回的字符串
///
/// # Safety
/// ptr 必须是通过 FFI 函数返回的有效字符串指针
#[no_mangle]
pub unsafe extern "C" fn ggs_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// 获取推荐的模型维度
///
/// # Safety
/// ptr 必须是有效的节点句柄
#[no_mangle]
pub unsafe extern "C" fn ggs_node_recommended_model_dim(
    ptr: *const NodeHandle,
) -> usize {
    if ptr.is_null() {
        return 256; // 默认值
    }
    
    let handle = &*ptr;
    let caps = handle.device_manager.get();
    caps.recommended_model_dim()
}

/// 获取推荐的训练间隔（秒）
///
/// # Safety
/// ptr 必须是有效的节点句柄
#[no_mangle]
pub unsafe extern "C" fn ggs_node_recommended_tick_interval(
    ptr: *const NodeHandle,
) -> u64 {
    if ptr.is_null() {
        return 10; // 默认 10 秒
    }
    
    let handle = &*ptr;
    let caps = handle.device_manager.get();
    caps.recommended_tick_interval().as_secs()
}

/// 检查是否应该暂停训练
///
/// # Safety
/// ptr 必须是有效的节点句柄
#[no_mangle]
pub unsafe extern "C" fn ggs_node_should_pause_training(
    ptr: *const NodeHandle,
) -> c_int {
    if ptr.is_null() {
        return 0; // false
    }
    
    let handle = &*ptr;
    let caps = handle.device_manager.get();
    if caps.should_pause_training() {
        1 // true
    } else {
        0 // false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_create_destroy() {
        unsafe {
            let ptr = ggs_node_create();
            assert!(!ptr.is_null());
            ggs_node_destroy(ptr);
        }
    }

    #[test]
    fn test_get_capabilities() {
        unsafe {
            let ptr = ggs_node_create();
            let json_ptr = ggs_node_get_capabilities(ptr);
            assert!(!json_ptr.is_null());
            
            let json = CStr::from_ptr(json_ptr).to_str().unwrap();
            assert!(json.contains("max_memory_mb"));
            
            ggs_string_free(json_ptr);
            ggs_node_destroy(ptr);
        }
    }

    #[test]
    fn test_update_network_type() {
        unsafe {
            let ptr = ggs_node_create();
            let wifi = CString::new("wifi").unwrap();
            let result = ggs_node_update_network_type(ptr, wifi.as_ptr());
            assert_eq!(result, FfiError::Success as c_int);
            ggs_node_destroy(ptr);
        }
    }

    #[test]
    fn test_update_battery() {
        unsafe {
            let ptr = ggs_node_create();
            let result = ggs_node_update_battery(ptr, 0.75, 1);
            assert_eq!(result, FfiError::Success as c_int);
            ggs_node_destroy(ptr);
        }
    }
}


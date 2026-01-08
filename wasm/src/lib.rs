//! williw WASM 模块
//!
//! 为Web环境提供的去中心化算力系统WASM绑定

use wasm_bindgen::prelude::*;
use rand::RngCore;
use sha3::Digest;

/// 初始化WASM模块
#[wasm_bindgen(start)]
pub fn init() {
    // 使用 web_sys 的 console 进行日志输出
    web_sys::console::log_1(&"williw WASM 模块已初始化".into());
}

/// williw WASM 应用实例
#[wasm_bindgen]
pub struct WilliwWasmApp {
    // 内部状态
    initialized: bool,
}

/// williw 隐私配置
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum PrivacyLevel {
    None = 0,
    Basic = 1,
    Standard = 2,
    High = 3,
    Maximum = 4,
}

/// 设备能力信息
#[wasm_bindgen]
#[derive(Clone)]
pub struct DeviceCapabilities {
    cpu_cores: u32,
    memory_bytes: u64,
    storage_bytes: u64,
    can_compute: bool,
    can_store: bool,
    can_communicate: bool,
}

#[wasm_bindgen]
impl DeviceCapabilities {
    #[wasm_bindgen(constructor)]
    pub fn new(cpu_cores: u32, memory_bytes: u64, storage_bytes: u64) -> DeviceCapabilities {
        DeviceCapabilities {
            cpu_cores,
            memory_bytes,
            storage_bytes,
            can_compute: true,
            can_store: true,
            can_communicate: true,
        }
    }
    
    #[wasm_bindgen(getter)]
    pub fn cpuCores(&self) -> u32 { self.cpu_cores }
    
    #[wasm_bindgen(getter)]
    pub fn memoryBytes(&self) -> u64 { self.memory_bytes }
    
    #[wasm_bindgen(getter)]
    pub fn storageBytes(&self) -> u64 { self.storage_bytes }
    
    #[wasm_bindgen(getter)]
    pub fn canCompute(&self) -> bool { self.can_compute }
    
    #[wasm_bindgen(getter)]
    pub fn canStore(&self) -> bool { self.can_store }
    
    #[wasm_bindgen(getter)]
    pub fn canCommunicate(&self) -> bool { self.can_communicate }
}

#[wasm_bindgen]
impl WilliwWasmApp {
    /// 创建新的williw WASM应用
    #[wasm_bindgen(constructor)]
    pub fn new() -> WilliwWasmApp {
        web_sys::console::log_1(&"创建新的williw WASM应用".into());
        WilliwWasmApp {
            initialized: false,
        }
    }
    
    /// 初始化应用
    #[wasm_bindgen]
    pub fn initialize(&mut self) -> bool {
        web_sys::console::log_1(&"初始化williw WASM应用...".into());
        self.initialized = true;
        web_sys::console::log_1(&"williw WASM应用初始化完成".into());
        true
    }
    
    /// 检查是否已初始化
    #[wasm_bindgen(getter)]
    pub fn initialized(&self) -> bool {
        self.initialized
    }
    
    /// 获取版本信息
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        "0.1.0".to_string()
    }
    
    /// 获取设备能力
    #[wasm_bindgen]
    pub fn getDeviceCapabilities(&self) -> DeviceCapabilities {
        // 在WASM环境中，我们无法获取真实的硬件信息
        // 返回模拟值
        DeviceCapabilities {
            cpu_cores: 4,
            memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            storage_bytes: 256 * 1024 * 1024 * 1024, // 256GB
            can_compute: true,
            can_store: true,
            can_communicate: true,
        }
    }
    
    /// 计算隐私得分
    #[wasm_bindgen]
    pub fn calculatePrivacyScore(&self, level: PrivacyLevel) -> f64 {
        match level {
            PrivacyLevel::None => 0.0,
            PrivacyLevel::Basic => 0.25,
            PrivacyLevel::Standard => 0.5,
            PrivacyLevel::High => 0.75,
            PrivacyLevel::Maximum => 1.0,
        }
    }
    
    /// 生成模拟节点ID
    #[wasm_bindgen]
    pub fn generateNodeId(&self) -> String {
        // 生成一个简单的随机节点ID
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
    
    /// 简单的哈希计算（用于测试）
    #[wasm_bindgen]
    pub fn simpleHash(&self, input: &str) -> String {
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
    
    /// 销毁应用
    #[wasm_bindgen]
    pub fn destroy(&mut self) {
        web_sys::console::log_1(&"销毁williw WASM应用".into());
        self.initialized = false;
    }
}

/// 工具函数
#[wasm_bindgen]
pub fn version() -> String {
    "0.1.0".to_string()
}

#[wasm_bindgen]
pub fn isWasm() -> bool {
    true
}

/// 计算两个数字的和（简单测试函数）
#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// 计算两个数字的乘积（简单测试函数）
#[wasm_bindgen]
pub fn multiply(a: u32, b: u32) -> u32 {
    a * b
}

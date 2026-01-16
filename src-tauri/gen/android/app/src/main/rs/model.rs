//! 模型管理模块
//! 
//! 实现模型注册表、选择和兼容性验证功能

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::{ModelConfig, MODEL_REGISTRY, TRAINING_STATE};

/// 初始化模型注册表
pub fn initialize_model_registry() {
    let mut registry = MODEL_REGISTRY.lock().unwrap();
    if registry.is_empty() {
        log_d("Android", " 初始化模型注册表");
        
        // 添加预定义模型
        registry.insert("bert-base-uncased".to_string(), ModelConfig {
            id: "bert-base-uncased".to_string(),
            name: "BERT Base".to_string(),
            description: "Google BERT (Bidirectional Encoder Representations from Transformers) 12-layer, 768-hidden".to_string(),
            dimensions: 768,
            learning_rate: 2e-5,
            batch_size: 32,
        });
        
        registry.insert("gpt2-medium".to_string(), ModelConfig {
            id: "gpt2-medium".to_string(),
            name: "GPT-2 Medium".to_string(),
            description: "OpenAI GPT-2 Medium model with 345M parameters".to_string(),
            dimensions: 1024,
            learning_rate: 5e-5,
            batch_size: 16,
        });
        
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

/// 选择模型
pub fn select_model_internal(model_id: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    let device_manager = crate::DEVICE_MANAGER.lock().unwrap();
    let capabilities = device_manager.get();
    
    if !super::device::is_model_compatible(&model, &capabilities) {
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

/// 获取可用模型列表
pub fn get_available_models() -> String {
    log_d("Android", "📋 获取可用模型列表");
    
    // 确保注册表已初始化
    initialize_model_registry();
    
    // 获取模型列表
    let registry = MODEL_REGISTRY.lock().unwrap();
    let models: Vec<&ModelConfig> = registry.values().collect();
    serde_json::json!(models).to_string()
}

/// 获取模型详情
pub fn get_model_details(model_id: &str) -> Option<ModelConfig> {
    let registry = MODEL_REGISTRY.lock().unwrap();
    registry.get(model_id).cloned()
}

/// 添加自定义模型
pub fn add_custom_model(model: ModelConfig) -> Result<(), String> {
    let mut registry = MODEL_REGISTRY.lock().unwrap();
    
    if registry.contains_key(&model.id) {
        return Err(format!("模型ID '{}' 已存在", model.id));
    }
    
    registry.insert(model.id.clone(), model);
    log_d("Android", &format!("➕ 添加自定义模型: {}", model.name));
    Ok(())
}

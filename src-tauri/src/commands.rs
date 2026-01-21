use crate::state::{AppState, ModelConfig, TrainingStatus, DeviceInfo, AppSettings, ApiKeyEntry};
use crate::api_client::TrainingConfigData;
use tauri::State;
use williw::Node;  // 导入真实的Node
use williw::config::AppConfig;

// 确保 uuid 和 chrono 被导入
use uuid::Uuid;
use chrono::Utc;

/// Start training node
#[tauri::command]
pub async fn start_training(
    state: State<'_, AppState>
) -> Result<String, String> {
    let model_config = {
        let models = state.available_models.lock();
        models.first().cloned().unwrap_or_default()
    };

    // 创建AppConfig
    let mut app_config = AppConfig::default();
    
    // 根据模型配置调整AppConfig
    app_config.training.model_dim = model_config.dimensions;
    app_config.training.learning_rate = model_config.learning_rate;
    app_config.training.batch_size = model_config.batch_size;

    // 创建并启动Node
    let node = Node::new(app_config)
        .await
        .map_err(|e| format!("Failed to create node: {}", e))?;

    let node_id = node.comms.node_id().to_string();

    // 存储Node
    *state.node.lock() = Some(node);

    // 更新训练状态
    let mut status = state.training_status.lock();
    status.is_running = true;
    status.current_epoch = 0;
    status.accuracy = 0.0;
    status.loss = 1.0;
    status.samples_processed = 0;

    Ok(format!("Training started with node: {}", node_id))
}

/// Stop training node
#[tauri::command]
pub async fn stop_training(
    state: State<'_, AppState>
) -> Result<String, String> {
    let mut node_guard = state.node.lock();
    
    if let Some(_node) = node_guard.take() {
        // Node会被自动drop，清理资源
        // 如果需要显式停止，可以调用node.shutdown()等方法
        
        // 更新训练状态
        let mut status = state.training_status.lock();
        status.is_running = false;
        
        Ok("Training stopped successfully".to_string())
    } else {
        Err("No training node is running".to_string())
    }
}

/// Get current training status
#[tauri::command]
pub fn get_training_status(
    state: State<'_, AppState>
) -> TrainingStatus {
    state.training_status.lock().clone()
}

/// Select a model for training
#[tauri::command]
pub fn select_model(
    model_id: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let models = state.available_models.lock();
    
    // Check if model exists
    let model = models.iter().find(|m| m.id == model_id)
        .ok_or_else(|| format!("Model '{}' not found", model_id))?;

    // Update settings with new model
    let mut settings = state.settings.lock();
    settings.network_config.max_peers = model.batch_size as u32; // Use batch_size for demo

    Ok(format!("Selected model: {}", model.name))
}

/// Get available models
#[tauri::command]
pub fn get_available_models(
    state: State<'_, AppState>
) -> Vec<ModelConfig> {
    state.available_models.lock().clone()
}

/// Get device information
#[tauri::command]
pub fn get_device_info(
    state: State<'_, AppState>
) -> Option<DeviceInfo> {
    // Refresh device info before returning
    state.refresh_device_info();
    state.device_info.lock().clone()
}

/// Get training statistics
#[tauri::command]
pub fn get_training_stats(
    state: State<'_, AppState>
) -> TrainingStatus {
    state.training_status.lock().clone()
}

/// Update application settings
#[tauri::command]
pub fn update_settings(
    new_settings: AppSettings,
    state: State<'_, AppState>
) -> Result<String, String> {
    *state.settings.lock() = new_settings;
    Ok("Settings updated successfully".to_string())
}

/// Get current settings
#[tauri::command]
pub fn get_settings(
    state: State<'_, AppState>
) -> AppSettings {
    state.settings.lock().clone()
}

/// Get all API keys
#[tauri::command]
pub fn get_api_keys(
    state: State<'_, AppState>
) -> Vec<ApiKeyEntry> {
    state.api_keys.lock().clone()
}

/// Create new API key
#[tauri::command]
pub fn create_api_key(
    name: String,
    state: State<'_, AppState>
) -> Result<ApiKeyEntry, String> {
    let new_key = format!("sk-williw-{}", Uuid::new_v4());
    let entry = ApiKeyEntry {
        id: Uuid::new_v4().to_string(),
        name,
        key: new_key.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    
    state.api_keys.lock().push(entry.clone());
    
    Ok(entry)
}

/// Delete API key
#[tauri::command]
pub fn delete_api_key(
    id: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let mut keys = state.api_keys.lock();
    let initial_len = keys.len();
    keys.retain(|k| k.id != id);
    
    if keys.len() < initial_len {
        Ok("API key deleted successfully".to_string())
    } else {
        Err("API key not found".to_string())
    }
}

/// Update API key name
#[tauri::command]
pub fn update_api_key_name(
    id: String,
    new_name: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let mut keys = state.api_keys.lock();
    
    if let Some(key) = keys.iter_mut().find(|k| k.id == id) {
        key.name = new_name;
        Ok("API key name updated successfully".to_string())
    } else {
        Err("API key not found".to_string())
    }
}

/// Get node information
#[tauri::command]
pub fn get_node_info(
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    let node_guard = state.node.lock();
    
    if let Some(node) = node_guard.as_ref() {
        let node_id = node.comms.node_id().to_string();
        let capabilities = node.device_manager.get();
        let stats = node.stats.lock().unwrap();
        
        Ok(serde_json::json!({
            "id": node_id,
            "is_running": true,
            "tick_counter": node.tick_counter,
            "device_capabilities": {
                "max_memory_mb": capabilities.max_memory_mb,
                "cpu_cores": capabilities.cpu_cores,
                "has_gpu": capabilities.has_gpu,
                "network_type": capabilities.network_type,
                "battery_level": capabilities.battery_level,
                "is_charging": capabilities.is_charging
            },
            "training_stats": {
                "total_ticks": stats.get_stats().tick_count,
                "accuracy": stats.get_stats().training_accuracy,
                "loss": stats.get_stats().training_loss,
                "samples_processed": stats.get_stats().samples_processed
            }
        }))
    } else {
        Ok(serde_json::json!({
            "id": null,
            "is_running": false,
            "message": "Node is not running"
        }))
    }
}

/// Get connected peers information
#[tauri::command]
pub fn get_connected_peers(
    state: State<'_, AppState>
) -> Result<Vec<serde_json::Value>, String> {
    let node_guard = state.node.lock();
    
    if let Some(node) = node_guard.as_ref() {
        let (primary_peers, backup_peers) = node.topology.neighbor_sets();
        
        let mut peers = Vec::new();
        
        for peer_id in primary_peers {
            if let Some(snapshot) = node.topology.peer_snapshot(&peer_id) {
                peers.push(serde_json::json!({
                    "id": peer_id,
                    "type": "primary",
                    "similarity": snapshot.similarity,
                    "geo_affinity": snapshot.geo_affinity,
                    "embedding_dim": snapshot.embedding_dim,
                    "position": {
                        "lat": snapshot.position.lat,
                        "lon": snapshot.position.lon
                    }
                }));
            }
        }
        
        for peer_id in backup_peers {
            if let Some(snapshot) = node.topology.peer_snapshot(&peer_id) {
                peers.push(serde_json::json!({
                    "id": peer_id,
                    "type": "backup",
                    "similarity": snapshot.similarity,
                    "geo_affinity": snapshot.geo_affinity,
                    "embedding_dim": snapshot.embedding_dim,
                    "position": {
                        "lat": snapshot.position.lat,
                        "lon": snapshot.position.lon
                    }
                }));
            }
        }
        
        Ok(peers)
    } else {
        Ok(vec![])
    }
}

/// Upload device info to workers backend (/api/node-info)
#[tauri::command]
pub async fn upload_device_info_to_workers(
    state: State<'_, AppState>
) -> Result<String, String> {
    // 获取设备信息
    let device_info = state.device_info.lock().clone()
        .ok_or_else(|| "No device info available".to_string())?;

    // 上传到workers后端的 /api/node-info 端点
    match state.api_client.upload_node_info(crate::api_client::NodeInfo {
        node_id: state.api_client.get_device_id(),
        endpoint: "localhost:8080".to_string(), // 可以从配置获取
        capabilities: crate::api_client::NodeCapabilities {
            max_memory_gb: device_info.total_memory_gb,
            gpu_type: device_info.gpu_type.clone(),
            gpu_memory_gb: device_info.gpu_memory_total,
            cpu_cores: device_info.cpu_cores,
            network_bandwidth_mbps: 1000, // 可以动态检测
            supported_models: vec!["bert-base".to_string(), "gpt-2".to_string()], // 可以从可用模型获取
        },
        current_load: 0.5, // 可以动态获取
        latency: Some(50), // 可以动态检测
        reliability: 0.95, // 可以基于历史数据计算
    }).await {
        Ok(response) => {
            if response.success {
                Ok("Device info uploaded successfully to /api/node-info".to_string())
            } else {
                Err(format!("Upload failed: {}", response.message))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

/// Request inference from workers backend (/api/request)
#[tauri::command]
pub async fn request_inference_from_workers(
    model_id: String,
    input_data: serde_json::Value,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    // 请求推理到workers后端的 /api/request 端点
    match state.api_client.request_inference(model_id, input_data).await {
        Ok(response) => {
            if response.success {
                Ok(serde_json::json!({
                    "success": true,
                    "request_id": response.request_id,
                    "selected_nodes": response.selected_nodes,
                    "model_split_plan": response.model_split_plan,
                    "estimated_total_time": response.estimated_total_time,
                    "fallback_nodes": response.fallback_nodes,
                    "message": response.message
                }))
            } else {
                Err(format!("Request failed: {}", response.message))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

/// Upload selected model to workers backend (/api/model)
#[tauri::command]
pub async fn upload_model_selection_to_workers(
    model_id: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    // 获取模型配置
    let model_config = {
        let models = state.available_models.lock();
        models.iter().find(|m| m.id == model_id)
            .ok_or_else(|| format!("Model '{}' not found", model_id))?
            .clone()
    };
    
    // 创建训练配置
    let training_config = TrainingConfigData {
        learning_rate: model_config.learning_rate,
        batch_size: model_config.batch_size,
        epochs: 100, // 默认值，可以从设置中读取
        enable_distributed: true,
    };

    // 上传到workers后端的 /api/model 端点
    match state.api_client.upload_selected_model(model_config, training_config).await {
        Ok(response) => {
            if response.success {
                Ok("Model selection uploaded successfully to /api/model".to_string())
            } else {
                Err(format!("Upload failed: {}", response.message))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

/// Upload training data to workers backend (/api/training-data)
#[tauri::command]
pub async fn upload_training_data_to_workers(
    state: State<'_, AppState>
) -> Result<String, String> {
    // 获取训练状态
    let training_status = {
        let status = state.training_status.lock();
        status.clone()
    };
    
    // 获取节点ID（如果有的话）
    let node_id = {
        let node_guard = state.node.lock();
        if let Some(_node) = node_guard.as_ref() {
            // 这里可以从Node获取ID，目前使用None
            None
        } else {
            None
        }
    };

    // 上传到workers后端的 /api/training-data 端点
    match state.api_client.upload_training_data(training_status, node_id).await {
        Ok(response) => {
            if response.success {
                Ok("Training data uploaded successfully to /api/training-data".to_string())
            } else {
                Err(format!("Upload failed: {}", response.message))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

/// Test connection to workers backend
#[tauri::command]
pub async fn test_workers_connection(
    state: State<'_, AppState>
) -> Result<bool, String> {
    match state.api_client.test_connection().await {
        Ok(is_connected) => Ok(is_connected),
        Err(e) => Err(format!("Connection test failed: {}", e)),
    }
}

/// Reassign nodes when some nodes are unreachable
#[tauri::command]
pub async fn reassign_node_from_workers(
    failed_nodes: Vec<String>,
    current_splits: Vec<crate::api_client::ModelSplit>,
    request_id: String,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    // 调用API客户端的节点重新分配方法
    match state.api_client.reassign_node(failed_nodes, current_splits, request_id).await {
        Ok(response) => {
            if response.success {
                Ok(serde_json::json!({
                    "success": true,
                    "new_splits": response.new_splits,
                    "reassigned_nodes": response.reassigned_nodes,
                    "message": response.message
                }))
            } else {
                Err(format!("Node reassignment failed: {}", response.message))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

/// Check node health status
#[tauri::command]
pub async fn check_node_health_from_workers(
    node_id: String,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    // 调用API客户端的节点健康检查方法
    match state.api_client.check_node_health(node_id).await {
        Ok(response) => {
            Ok(serde_json::json!({
                "success": response.success,
                "message": response.message,
                "node_id": response.node_id,
                "is_healthy": response.is_healthy,
                "last_seen": response.last_seen,
                "current_load": response.current_load,
                "issues": response.issues
            }))
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

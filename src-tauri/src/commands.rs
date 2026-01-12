use crate::state::{AppState, ModelConfig, TrainingStatus, DeviceInfo, AppSettings};
use tauri::State;
// use williw::node::Node;  // Commented out - node module doesn't exist
use williw::config::AppConfig;

/// Start training node
#[tauri::command]
pub async fn start_training(
    state: State<'_, AppState>
) -> Result<String, String> {
    let _settings = state.settings.lock().clone();
    let _model_config = {
        let models = state.available_models.lock();
        models.first().cloned().unwrap_or_default()
    };

    // Create AppConfig (simplified version for Tauri)
    // In production, you would use proper config based on model_config
    let _app_config = AppConfig::default();

    // Create and start node
    // let node = Node::new(app_config)
    //     .await
    //     .map_err(|e| format!("Failed to create node: {}", e))?;

    // let node_id = node.comms.node_id().to_string();
    let node_id = "demo-node-id".to_string();

    // Store node in state
    // *state.node.lock() = Some(node);
    // Store a placeholder instead
    // TODO: Implement actual Node when available

    // Update training status
    let mut status = state.training_status.lock();
    status.is_running = true;

    Ok(format!("Training started with node: {}", node_id))
}

/// Stop training node
#[tauri::command]
pub async fn stop_training(
    state: State<'_, AppState>
) -> Result<String, String> {
    let mut node_guard = state.node.lock();

    if let Some(_node) = node_guard.take() {
        // Node will be dropped here, which should gracefully stop it
        let mut status = state.training_status.lock();
        status.is_running = false;
        
        Ok("Training stopped successfully".to_string())
    } else {
        Ok("No training node was running".to_string())
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

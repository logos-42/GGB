// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod events;
mod api_client;

use tauri::Emitter;
use state::AppState;

#[tokio::main]
async fn main() {
    let app_state = AppState::new().await;

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::start_training,
            commands::stop_training,
            commands::get_training_status,
            commands::select_model,
            commands::get_available_models,
            commands::get_device_info,
            commands::get_training_stats,
            commands::update_settings,
            commands::get_settings,
            commands::get_api_keys,
            commands::create_api_key,
            commands::delete_api_key,
            commands::update_api_key_name,
            commands::get_node_info,
            commands::get_connected_peers,
            commands::upload_device_info_to_workers,
            commands::upload_model_selection_to_workers,
            commands::upload_training_data_to_workers,
            commands::test_workers_connection,
            commands::request_inference_from_workers,
            commands::reassign_node_from_workers,
            commands::check_node_health_from_workers,
        ])
        .setup(|app| {
            // Initialize event handlers
            events::setup_event_handlers(app.handle().clone())?;

            // Start background task to refresh device info every minute
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    // Emit event to refresh device info in frontend
                    let _ = app_handle.emit("device_info_refresh", ());
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

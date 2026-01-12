// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod events;

// use tauri::Manager;  // Unused import
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
        ])
        .setup(|app| {
            // Initialize event handlers
            events::setup_event_handlers(app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

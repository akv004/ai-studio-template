// ============================================
// AI STUDIO - TAURI COMMANDS
// IPC commands exposed to the React UI
// ============================================

mod commands;
mod sidecar;
mod system;

use commands::*;
use sidecar::*;
use system::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SidecarManager::default())
        .manage(ApprovalManager::default())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let sidecar = app.state::<SidecarManager>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let _ = sidecar.start(&app_handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            list_projects,
            save_project,
            sidecar_start,
            sidecar_stop,
            sidecar_status,
            sidecar_request,
            approve_tool_request
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app_handle = window.app_handle().clone();
                let sidecar = app_handle.state::<SidecarManager>().inner().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = sidecar.stop().await;
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

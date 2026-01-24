// ============================================
// AI STUDIO - TAURI COMMANDS
// IPC commands exposed to the React UI
// ============================================

mod commands;
mod system;

use commands::*;
use system::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            list_projects,
            save_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

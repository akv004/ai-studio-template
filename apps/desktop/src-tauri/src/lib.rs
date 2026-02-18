// ============================================
// AI STUDIO — Desktop Application
// Tauri 2 + SQLite + Python Sidecar
// ============================================

mod commands;
mod db;
mod routing;
mod sidecar;
mod system;

use commands::*;
use db::Database;
use sidecar::*;
use system::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Fix WebKitGTK GPU rendering crash on some Linux systems
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    // Initialize SQLite database before anything else
    let database = Database::init().expect("Failed to initialize database");

    tauri::Builder::default()
        .manage(database)
        .manage(SidecarManager::default())
        .manage(ApprovalManager::default())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let sidecar = app.state::<SidecarManager>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let _ = sidecar.start(&app_handle).await;
            });

            // Start event bridge (WebSocket → SQLite + UI)
            {
                let sidecar_ref = app.state::<SidecarManager>().inner().clone();
                let db_ref = app.state::<Database>().clone();
                spawn_event_bridge(app.handle(), &sidecar_ref, &db_ref);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Testing
            greet,
            get_system_info,
            // Agent CRUD
            list_agents,
            get_agent,
            create_agent,
            update_agent,
            delete_agent,
            // Session CRUD
            list_sessions,
            create_session,
            branch_session,
            get_session_messages,
            delete_session,
            // Chat
            send_message,
            // Inspector
            get_session_events,
            get_session_stats,
            // Runs
            list_runs,
            create_run,
            cancel_run,
            get_run,
            // DB
            wipe_database,
            // Settings
            get_all_settings,
            set_setting,
            // Budget
            get_budget_status,
            set_budget,
            // Provider Keys
            list_provider_keys,
            set_provider_key,
            delete_provider_key,
            // MCP Servers
            list_mcp_servers,
            add_mcp_server,
            update_mcp_server,
            remove_mcp_server,
            // Approval Rules
            list_approval_rules,
            create_approval_rule,
            update_approval_rule,
            delete_approval_rule,
            // Workflows (Node Editor)
            list_workflows,
            get_workflow,
            create_workflow,
            update_workflow,
            delete_workflow,
            duplicate_workflow,
            // Workflow Execution (Phase 3B)
            validate_workflow,
            run_workflow,
            // Workflow Templates (Phase 3C)
            list_templates,
            load_template,
            // Sidecar
            sidecar_start,
            sidecar_stop,
            sidecar_status,
            sidecar_request,
            approve_tool_request,
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

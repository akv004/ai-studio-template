// ============================================
// SYSTEM INFO
// OS and hardware information commands
// ============================================

use serde::Serialize;

/// System information structure
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub platform: String,
    pub os_version: String,
    pub arch: String,
    pub hostname: String,
}

/// Get system information
#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        platform: std::env::consts::OS.to_string(),
        os_version: std::env::consts::FAMILY.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
    }
}

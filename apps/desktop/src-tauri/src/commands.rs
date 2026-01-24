// ============================================
// TAURI COMMANDS
// IPC command handlers for UI communication
// ============================================

use serde::{Deserialize, Serialize};

/// Simple greeting command for testing IPC
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AI Studio.", name)
}

/// Project data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

/// List all projects from the data directory
/// Currently returns mock data
#[tauri::command]
pub fn list_projects() -> Vec<Project> {
    // TODO: Implement actual file system reading
    // For now, return mock data
    vec![
        Project {
            id: "1".to_string(),
            name: "Object Detection Pipeline".to_string(),
            description: "Real-time object detection for autonomous navigation".to_string(),
            created_at: "2024-01-15T10:00:00Z".to_string(),
            updated_at: "2024-01-20T14:30:00Z".to_string(),
        },
        Project {
            id: "2".to_string(),
            name: "Voice Assistant".to_string(),
            description: "Multi-language voice recognition and synthesis".to_string(),
            created_at: "2024-01-10T08:00:00Z".to_string(),
            updated_at: "2024-01-19T16:45:00Z".to_string(),
        },
    ]
}

/// Save project to disk
/// Currently a stub that returns success
#[tauri::command]
pub fn save_project(project: Project) -> Result<String, String> {
    // TODO: Implement actual file system writing
    println!("Saving project: {:?}", project);
    Ok(format!("Project '{}' saved successfully", project.name))
}

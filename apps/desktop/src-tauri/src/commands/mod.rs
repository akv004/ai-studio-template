pub mod agents;
pub mod approval_rules;
pub mod budget;
pub mod chat;
pub mod inspector;
pub mod mcp;
pub mod plugins;
pub mod providers;
pub mod runs;
pub mod sessions;
pub mod settings;
pub mod templates;
pub mod workflows;

// Re-export all commands for use in lib.rs invoke_handler
pub use agents::*;
pub use approval_rules::*;
pub use budget::{get_budget_status, set_budget};
pub use chat::*;
pub use inspector::*;
pub use mcp::*;
pub use plugins::*;
pub use providers::*;
pub use runs::*;
pub use sessions::*;
pub use settings::*;
pub use templates::*;
pub use workflows::*;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AI Studio.", name)
}

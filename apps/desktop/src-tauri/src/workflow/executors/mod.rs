pub mod input;
pub mod output;
pub mod llm;
pub mod transform;
pub mod router;
pub mod tool;
pub mod approval;

use crate::db::Database;
use crate::sidecar::SidecarManager;
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;

pub struct ExecutionContext<'a> {
    pub db: &'a Database,
    pub sidecar: &'a SidecarManager,
    pub app: &'a tauri::AppHandle,
    pub session_id: &'a str,
    pub all_settings: &'a HashMap<String, String>,
    pub node_outputs: &'a HashMap<String, serde_json::Value>,
    pub inputs: &'a HashMap<String, serde_json::Value>,
    pub outgoing_by_handle: &'a HashMap<(String, String), Vec<String>>,
    pub seq_counter: &'a AtomicI64,
}

pub struct NodeOutput {
    pub value: serde_json::Value,
    pub skip_nodes: Vec<String>,
}

impl NodeOutput {
    pub fn value(value: serde_json::Value) -> Self {
        Self { value, skip_nodes: Vec::new() }
    }

    pub fn with_skips(value: serde_json::Value, skip_nodes: Vec<String>) -> Self {
        Self { value, skip_nodes }
    }
}

#[async_trait::async_trait]
pub trait NodeExecutor: Send + Sync {
    fn node_type(&self) -> &str;

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String>;
}

pub struct ExecutorRegistry {
    executors: HashMap<String, Box<dyn NodeExecutor>>,
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        let mut executors: HashMap<String, Box<dyn NodeExecutor>> = HashMap::new();
        executors.insert("input".to_string(), Box::new(input::InputExecutor));
        executors.insert("output".to_string(), Box::new(output::OutputExecutor));
        executors.insert("llm".to_string(), Box::new(llm::LlmExecutor));
        executors.insert("transform".to_string(), Box::new(transform::TransformExecutor));
        executors.insert("router".to_string(), Box::new(router::RouterExecutor));
        executors.insert("tool".to_string(), Box::new(tool::ToolExecutor));
        executors.insert("approval".to_string(), Box::new(approval::ApprovalExecutor));
        Self { executors }
    }

    pub fn get(&self, node_type: &str) -> Option<&dyn NodeExecutor> {
        self.executors.get(node_type).map(|e| e.as_ref())
    }
}

pub mod input;
pub mod output;
pub mod llm;
pub mod transform;
pub mod router;
pub mod tool;
pub mod approval;
pub mod subworkflow;
pub mod http_request;
pub mod file_read;
pub mod file_glob;
pub mod file_write;
pub mod shell_exec;
pub mod validator;
pub mod iterator;
pub mod aggregator;
pub mod knowledge_base;
pub mod loop_node;
pub mod exit;
pub mod webhook_trigger;
pub mod cron_trigger;
pub mod email_send;

use crate::db::Database;
use crate::sidecar::SidecarManager;
use std::collections::{HashMap, HashSet};
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
    pub visited_workflows: &'a HashSet<String>,
    pub graph_json: &'a str,
    /// Unique per workflow run — used for LLM session conversation IDs
    pub workflow_run_id: &'a str,
    /// When true, skip DB writes (record_event) — used by live workflow mode
    pub ephemeral: bool,
}

pub struct NodeOutput {
    pub value: serde_json::Value,
    pub skip_nodes: Vec<String>,
    pub extra_outputs: HashMap<String, serde_json::Value>,
}

impl NodeOutput {
    pub fn value(value: serde_json::Value) -> Self {
        Self { value, skip_nodes: Vec::new(), extra_outputs: HashMap::new() }
    }

    pub fn with_skips(value: serde_json::Value, skip_nodes: Vec<String>) -> Self {
        Self { value, skip_nodes, extra_outputs: HashMap::new() }
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
        // Phase 3 core
        executors.insert("input".to_string(), Box::new(input::InputExecutor));
        executors.insert("output".to_string(), Box::new(output::OutputExecutor));
        executors.insert("llm".to_string(), Box::new(llm::LlmExecutor));
        executors.insert("transform".to_string(), Box::new(transform::TransformExecutor));
        executors.insert("router".to_string(), Box::new(router::RouterExecutor));
        executors.insert("tool".to_string(), Box::new(tool::ToolExecutor));
        executors.insert("approval".to_string(), Box::new(approval::ApprovalExecutor));
        // Phase 4A
        executors.insert("subworkflow".to_string(), Box::new(subworkflow::SubworkflowExecutor));
        executors.insert("http_request".to_string(), Box::new(http_request::HttpRequestExecutor));
        executors.insert("file_read".to_string(), Box::new(file_read::FileReadExecutor));
        executors.insert("file_glob".to_string(), Box::new(file_glob::FileGlobExecutor));
        executors.insert("file_write".to_string(), Box::new(file_write::FileWriteExecutor));
        executors.insert("shell_exec".to_string(), Box::new(shell_exec::ShellExecExecutor));
        executors.insert("validator".to_string(), Box::new(validator::ValidatorExecutor));
        // Phase 4B
        executors.insert("iterator".to_string(), Box::new(iterator::IteratorExecutor));
        executors.insert("aggregator".to_string(), Box::new(aggregator::AggregatorExecutor));
        // Phase 5A — RAG
        executors.insert("knowledge_base".to_string(), Box::new(knowledge_base::KnowledgeBaseExecutor));
        // Phase 5A — Loop & Feedback
        executors.insert("loop".to_string(), Box::new(loop_node::LoopExecutor));
        executors.insert("exit".to_string(), Box::new(exit::ExitExecutor));
        // Triggers
        executors.insert("webhook_trigger".to_string(), Box::new(webhook_trigger::WebhookTriggerExecutor));
        executors.insert("cron_trigger".to_string(), Box::new(cron_trigger::CronTriggerExecutor));
        // Communication
        executors.insert("email_send".to_string(), Box::new(email_send::EmailSendExecutor));
        Self { executors }
    }

    pub fn get(&self, node_type: &str) -> Option<&dyn NodeExecutor> {
        self.executors.get(node_type).map(|e| e.as_ref())
    }
}

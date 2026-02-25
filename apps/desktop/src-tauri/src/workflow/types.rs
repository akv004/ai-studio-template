use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunWorkflowRequest {
    pub workflow_id: String,
    pub inputs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunResult {
    pub session_id: String,
    pub status: String,
    pub outputs: HashMap<String, serde_json::Value>,
    /// All node outputs keyed by node_id. Used internally by Loop executor
    /// to access intermediate results (e.g., LLM answer when Router skips Exit).
    #[serde(skip_serializing)]
    pub node_outputs: HashMap<String, serde_json::Value>,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
    pub duration_ms: i64,
    pub node_count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

use crate::error::AppError;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_count: usize,
}

pub const TEMPLATES: &[(&str, &str, &str, &str)] = &[
    ("code-review", "Code Review", "Analyze PR, classify by severity, output structured review",
        include_str!("../../templates/code-review.json")),
    ("research", "Research Assistant", "Research a topic and produce a formatted report",
        include_str!("../../templates/research.json")),
    ("data-pipeline", "Data Pipeline", "Extract structured data from raw input using LLM",
        include_str!("../../templates/data-pipeline.json")),
    ("multi-model-compare", "Multi-Model Compare", "Send the same prompt to 3 models and compare outputs",
        include_str!("../../templates/multi-model-compare.json")),
    ("safe-executor", "Safe Executor", "Plan a shell command with LLM, approve, then execute",
        include_str!("../../templates/safe-executor.json")),
];

#[tauri::command]
pub fn list_templates() -> Vec<TemplateSummary> {
    TEMPLATES.iter().map(|(id, name, desc, json)| {
        let node_count = serde_json::from_str::<serde_json::Value>(json)
            .ok()
            .and_then(|v| v.get("nodes").and_then(|n| n.as_array()).map(|a| a.len()))
            .unwrap_or(0);
        TemplateSummary {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            node_count,
        }
    }).collect()
}

#[tauri::command]
pub fn load_template(template_id: String) -> Result<String, AppError> {
    TEMPLATES.iter()
        .find(|(id, _, _, _)| *id == template_id)
        .map(|(_, _, _, json)| json.to_string())
        .ok_or_else(|| AppError::NotFound(format!("Template '{template_id}' not found")))
}

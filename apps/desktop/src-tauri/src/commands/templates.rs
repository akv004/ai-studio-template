use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_count: usize,
    pub source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTemplateRequest {
    pub name: String,
    pub description: String,
    pub graph_json: String,
}

pub const TEMPLATES: &[(&str, &str, &str, &str)] = &[
    // Original 5
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
    // Community templates
    ("email-classifier", "Email Classifier", "Classify emails as urgent/normal/spam, auto-draft replies for urgent",
        include_str!("../../templates/email-classifier.json")),
    ("content-moderator", "Content Moderator", "Screen content for policy violations with human review for borderline cases",
        include_str!("../../templates/content-moderator.json")),
    ("translation-pipeline", "Translation Pipeline", "Detect language, translate to target language with formatting preserved",
        include_str!("../../templates/translation-pipeline.json")),
    ("meeting-notes", "Meeting Notes", "Summarize transcript and extract action items in parallel",
        include_str!("../../templates/meeting-notes.json")),
    // rag-search removed — superseded by Knowledge Q&A (uses Knowledge Base node instead of shell tool)
    // Hardware + AI templates
    ("webcam-monitor", "Webcam Monitor", "Capture webcam frame with YOLO detection, route on person detected, LLM describes scene",
        include_str!("../../templates/webcam-monitor.json")),
    // Hybrid intelligence
    ("hybrid-intelligence", "Hybrid Intelligence", "Two models think differently — local Qwen (engineer) and Gemini (architect) in parallel, then a synthesizer merges the best of both",
        include_str!("../../templates/hybrid-intelligence.json")),
    // DevOps
    ("smart-deployer", "Smart Deployer", "Natural language microservice deployment — LLM reads your config, builds a plan, you approve, it runs gh CLI",
        include_str!("../../templates/smart-deployer.json")),
    // RAG Knowledge Base
    ("knowledge-qa", "Knowledge Q&A", "Index any folder (docs, code, configs), ask questions, get answers with source citations",
        include_str!("../../templates/knowledge-qa.json")),
    ("smart-deployer-rag", "Smart Deployer + RAG", "RAG-powered deployment: Knowledge Base indexes deploy docs, LLM plans from context, approval gate, then execute",
        include_str!("../../templates/smart-deployer-rag.json")),
    // codebase-explorer removed — same graph as Knowledge Q&A with different config
    // Loop & Feedback
    ("self-refine", "Self-Refine", "Draft → critique → revise loop — LLM improves its own output iteratively (3 rounds)",
        include_str!("../../templates/self-refine.json")),
    ("agentic-search", "Agentic Search", "LLM decides what to search, evaluates results, searches again if needed (max 5 rounds)",
        include_str!("../../templates/agentic-search.json")),
];

fn templates_directory() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ai-studio")
        .join("templates")
}

fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn load_user_templates() -> Vec<TemplateSummary> {
    let dir = templates_directory();
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: skipping invalid template {:?}: {}", path, e);
                continue;
            }
        };
        let parsed: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Warning: skipping invalid JSON in {:?}: {}", path, e);
                continue;
            }
        };

        let slug = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let name = parsed.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&slug)
            .to_string();
        let description = parsed.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let node_count = parsed.get("nodes")
            .and_then(|n| n.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        results.push(TemplateSummary {
            id: format!("user:{slug}"),
            name,
            description,
            node_count,
            source: "user".to_string(),
        });
    }
    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    results
}

#[tauri::command]
pub fn list_templates() -> Vec<TemplateSummary> {
    let mut all: Vec<TemplateSummary> = TEMPLATES.iter().map(|(id, name, desc, json)| {
        let node_count = serde_json::from_str::<serde_json::Value>(json)
            .ok()
            .and_then(|v| v.get("nodes").and_then(|n| n.as_array()).map(|a| a.len()))
            .unwrap_or(0);
        TemplateSummary {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            node_count,
            source: "bundled".to_string(),
        }
    }).collect();

    all.extend(load_user_templates());
    all
}

#[tauri::command]
pub fn load_template(template_id: String) -> Result<String, AppError> {
    // User template: read from disk, strip metadata
    if let Some(slug) = template_id.strip_prefix("user:") {
        let path = templates_directory().join(format!("{slug}.json"));
        let content = std::fs::read_to_string(&path)
            .map_err(|e| AppError::NotFound(format!("User template '{slug}': {e}")))?;
        let mut parsed: serde_json::Value = serde_json::from_str(&content)?;

        // Strip metadata, return only graph data
        let obj = parsed.as_object_mut()
            .ok_or_else(|| AppError::Internal("Template is not a JSON object".into()))?;
        obj.remove("name");
        obj.remove("description");
        obj.remove("created_at");

        return Ok(serde_json::to_string(&parsed)?);
    }

    // Bundled template
    TEMPLATES.iter()
        .find(|(id, _, _, _)| *id == template_id)
        .map(|(_, _, _, json)| json.to_string())
        .ok_or_else(|| AppError::NotFound(format!("Template '{template_id}' not found")))
}

#[tauri::command]
pub fn save_as_template(request: SaveTemplateRequest) -> Result<TemplateSummary, AppError> {
    let graph: serde_json::Value = serde_json::from_str(&request.graph_json)
        .map_err(|e| AppError::Validation(format!("Invalid graph JSON: {e}")))?;

    let node_count = graph.get("nodes")
        .and_then(|n| n.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let slug = slugify(&request.name);
    if slug.is_empty() {
        return Err(AppError::Validation("Template name must contain at least one alphanumeric character".into()));
    }

    // Build template file: metadata + graph data
    let mut template = serde_json::Map::new();
    template.insert("name".into(), serde_json::Value::String(request.name.clone()));
    template.insert("description".into(), serde_json::Value::String(request.description.clone()));
    template.insert("created_at".into(), serde_json::Value::String(
        chrono::Utc::now().to_rfc3339()
    ));

    // Merge graph fields (nodes, edges, viewport) into template
    if let Some(obj) = graph.as_object() {
        for (k, v) in obj {
            template.insert(k.clone(), v.clone());
        }
    }

    let dir = templates_directory();
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Internal(format!("Failed to create templates directory: {e}")))?;

    let path = dir.join(format!("{slug}.json"));
    let content = serde_json::to_string_pretty(&template)?;
    std::fs::write(&path, content)
        .map_err(|e| AppError::Internal(format!("Failed to write template: {e}")))?;

    Ok(TemplateSummary {
        id: format!("user:{slug}"),
        name: request.name,
        description: request.description,
        node_count,
        source: "user".to_string(),
    })
}

#[tauri::command]
pub fn delete_user_template(template_id: String) -> Result<(), AppError> {
    let slug = template_id.strip_prefix("user:")
        .ok_or_else(|| AppError::Validation("Can only delete user templates".into()))?;

    let path = templates_directory().join(format!("{slug}.json"));
    if !path.exists() {
        return Err(AppError::NotFound(format!("Template '{slug}' not found")));
    }

    std::fs::remove_file(&path)
        .map_err(|e| AppError::Internal(format!("Failed to delete template: {e}")))?;

    Ok(())
}

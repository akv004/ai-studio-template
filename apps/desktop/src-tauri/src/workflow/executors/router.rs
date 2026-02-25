use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::events::record_event;

pub struct RouterExecutor;

#[async_trait::async_trait]
impl NodeExecutor for RouterExecutor {
    fn node_type(&self) -> &str { "router" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let branches = node_data.get("branches").and_then(|v| v.as_array());
        let branches = match branches {
            Some(b) if !b.is_empty() => b,
            _ => return Err(format!("Router node '{}' has no branches configured", node_id)),
        };

        let branch_names: Vec<String> = branches.iter()
            .filter_map(|b| {
                if let Some(s) = b.as_str() {
                    Some(s.to_string())
                } else {
                    b.get("name").and_then(|v| v.as_str()).map(|s| s.to_string())
                }
            })
            .collect();

        if branch_names.is_empty() {
            return Err(format!("Router node '{}' has no valid branch names", node_id));
        }

        let incoming_text = incoming.as_ref().map(|v| match v.as_str() {
            Some(s) => s.to_string(),
            None => serde_json::to_string(v).unwrap_or_default(),
        }).unwrap_or_default();

        let mode = node_data.get("mode").and_then(|v| v.as_str()).unwrap_or("pattern");

        let selected = if mode == "llm" {
            // LLM classification mode — ask an LLM to pick the branch
            let classify_prompt = format!(
                "Classify the following input into exactly one of these categories: {}.\n\n\
                 Input: {}\n\n\
                 Respond with ONLY the category name, nothing else.",
                branch_names.join(", "),
                incoming_text,
            );

            let provider_name = node_data.get("provider").and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .or_else(|| ctx.all_settings.get("default.provider").map(|s| s.trim_matches('"')))
                .unwrap_or("ollama");
            let model = node_data.get("model").and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .or_else(|| ctx.all_settings.get("default.model").map(|s| s.trim_matches('"')))
                .unwrap_or("");

            let mut body = serde_json::json!({
                "messages": [{ "role": "user", "content": classify_prompt }],
                "provider": provider_name,
                "model": model,
                "temperature": 0.0,
            });

            let prefix = format!("provider.{}.", provider_name);
            for (k, v) in ctx.all_settings {
                if let Some(field) = k.strip_prefix(&prefix) {
                    let clean_val = v.trim_matches('"').to_string();
                    match field {
                        "api_key" => { body["api_key"] = serde_json::Value::String(clean_val); }
                        "base_url" | "endpoint" => { body["base_url"] = serde_json::Value::String(clean_val); }
                        _ => {}
                    }
                }
            }

            let resp = ctx.sidecar.proxy_request("POST", "/chat/direct", Some(body)).await
                .map_err(|e| format!("Router LLM call failed: {}", e))?;

            let classification = resp.get("content").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();

            branch_names.iter().find(|name| {
                name.eq_ignore_ascii_case(&classification)
            }).cloned().unwrap_or_else(|| branch_names[0].clone())
        } else {
            // Pattern matching mode — check if input text contains each branch name
            let input_lower = incoming_text.to_lowercase();
            branch_names.iter().find(|name| {
                input_lower.contains(&name.to_lowercase())
            }).cloned().unwrap_or_else(|| branch_names.last().cloned().unwrap_or_default())
        };

        let selected_idx = branch_names.iter().position(|n| n == &selected);
        let mut skip_nodes = Vec::new();
        for (i, _branch_name) in branch_names.iter().enumerate() {
            if Some(i) == selected_idx {
                continue;
            }
            let handle_name = format!("branch-{}", i);
            let key = (node_id.to_string(), handle_name);
            if let Some(targets) = ctx.outgoing_by_handle.get(&key) {
                for target in targets {
                    eprintln!("[workflow] Router '{}': skipping downstream node '{}' (non-selected branch '{}')",
                        node_id, target, branch_names[i]);
                    skip_nodes.push(target.clone());
                }
            }
        }

        let _ = record_event(ctx.db, ctx.session_id, "workflow.node.completed", "desktop.workflow",
            serde_json::json!({
                "node_id": node_id, "node_type": "router",
                "mode": mode, "selected_branch": &selected,
            }));

        let output_value = serde_json::json!({
            "selectedBranch": &selected,
            "value": incoming.clone().unwrap_or(serde_json::Value::Null),
        });

        Ok(NodeOutput::with_skips(output_value, skip_nodes))
    }
}

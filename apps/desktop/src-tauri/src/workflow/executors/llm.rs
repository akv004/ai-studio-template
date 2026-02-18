use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::events::record_event;
use crate::workflow::engine::resolve_template;

pub struct LlmExecutor;

#[async_trait::async_trait]
impl NodeExecutor for LlmExecutor {
    fn node_type(&self) -> &str { "llm" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let provider_name = node_data.get("provider").and_then(|v| v.as_str()).unwrap_or("ollama");
        let model = node_data.get("model").and_then(|v| v.as_str()).unwrap_or("");
        let temperature = node_data.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7);
        let system_prompt = node_data.get("systemPrompt").and_then(|v| v.as_str()).unwrap_or("");
        eprintln!("[workflow] LLM node '{}': provider={}, model={}", node_id, provider_name, model);

        let prompt_template = node_data.get("prompt").and_then(|v| v.as_str()).unwrap_or("{{input}}");
        let prompt = if prompt_template.contains("{{") {
            resolve_template(prompt_template, ctx.node_outputs, ctx.inputs)
        } else if let Some(inc) = incoming {
            let inc_str = match inc.as_str() {
                Some(s) => s.to_string(),
                None => serde_json::to_string(inc).unwrap_or_default(),
            };
            format!("{}\n\n{}", prompt_template, inc_str)
        } else {
            prompt_template.to_string()
        };
        eprintln!("[workflow] LLM node '{}': prompt='{}' (template='{}')", node_id,
            &prompt[..prompt.len().min(200)], prompt_template);

        let prefix = format!("provider.{}.", provider_name);
        let mut api_key = String::new();
        let mut base_url = String::new();
        let mut extra_config = serde_json::Map::new();
        for (k, v) in ctx.all_settings {
            if let Some(field) = k.strip_prefix(&prefix) {
                let clean_val = v.trim_matches('"').to_string();
                match field {
                    "api_key" => api_key = clean_val,
                    "base_url" | "endpoint" => base_url = clean_val,
                    _ => { extra_config.insert(field.to_string(), serde_json::Value::String(clean_val)); }
                }
            }
        }

        eprintln!("[workflow] LLM node '{}': calling /chat/direct (api_key={}, base_url={})",
            node_id, if api_key.is_empty() { "none" } else { "set" }, if base_url.is_empty() { "none" } else { &base_url });

        let mut body = serde_json::json!({
            "messages": [{ "role": "user", "content": prompt }],
            "provider": provider_name,
            "model": model,
            "temperature": temperature,
        });
        if !system_prompt.is_empty() {
            body["system_prompt"] = serde_json::Value::String(system_prompt.to_string());
        }
        if !api_key.is_empty() {
            body["api_key"] = serde_json::Value::String(api_key);
        }
        if !base_url.is_empty() {
            body["base_url"] = serde_json::Value::String(base_url);
        }
        if !extra_config.is_empty() {
            body["extra_config"] = serde_json::Value::Object(extra_config);
        }

        let _ = record_event(ctx.db, ctx.session_id, "llm.request.started", "desktop.workflow",
            serde_json::json!({ "node_id": node_id, "model": model, "provider": provider_name }));

        let resp = ctx.sidecar.proxy_request("POST", "/chat/direct", Some(body)).await
            .map_err(|e| {
                eprintln!("[workflow] ERROR: LLM call failed for node '{}': {}", node_id, e);
                format!("LLM call failed for node '{}': {}", node_id, e)
            })?;

        let content = resp.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
        eprintln!("[workflow] LLM node '{}': response OK, content_len={}", node_id, content.len());
        let usage = resp.get("usage");
        let input_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
        let output_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
        let resp_model = resp.get("model").and_then(|v| v.as_str()).unwrap_or(model).to_string();

        let _ = record_event(ctx.db, ctx.session_id, "llm.response.completed", "desktop.workflow",
            serde_json::json!({
                "node_id": node_id, "model": resp_model, "provider": provider_name,
                "input_tokens": input_tokens, "output_tokens": output_tokens,
            }));

        Ok(NodeOutput::value(serde_json::json!({
            "content": content,
            "__usage": {
                "total_tokens": input_tokens + output_tokens,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "cost_usd": 0.0,
            }
        })))
    }
}

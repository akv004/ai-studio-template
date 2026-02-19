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

        eprintln!("[workflow] LLM node '{}': incoming={:?}", node_id,
            incoming.as_ref().map(|v| v.to_string()[..v.to_string().len().min(200)].to_string()));
        eprintln!("[workflow] LLM node '{}': node_data keys={:?}", node_id,
            node_data.as_object().map(|o| o.keys().collect::<Vec<_>>()));

        // Handle Inputs (System, Context, Prompt)
        // 1. System Prompt: Check incoming "system" -> config "systemPrompt" -> default
        let mut system_prompt = incoming.as_ref()
            .and_then(|inc| inc.get("system"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if system_prompt.is_empty() {
             system_prompt = node_data.get("systemPrompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
        }

        // 2. Context: Check incoming "context" -> injected into prompt or system
        let context_str = incoming.as_ref()
            .and_then(|inc| inc.get("context"))
            .map(|v| if let Some(s) = v.as_str() { s.to_string() } else { serde_json::to_string_pretty(v).unwrap_or_default() })
            .unwrap_or_default();

        // 3. Prompt resolution chain:
        //    a) incoming edge "prompt" handle (non-empty) → use directly
        //    b) incoming as bare string (single edge, non-empty) → use directly
        //    c) template from node config → resolve via template engine
        //    d) fallback: "{{input}}" template

        let incoming_prompt = incoming.as_ref()
            .and_then(|inc| inc.get("prompt"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let incoming_bare = incoming.as_ref()
            .and_then(|inc| inc.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let prompt_template = node_data.get("prompt").and_then(|v| v.as_str()).unwrap_or("{{input}}");

        eprintln!("[workflow] LLM node '{}': incoming_prompt={:?}, incoming_bare={:?}, template='{}'",
            node_id,
            incoming_prompt.as_ref().map(|s| &s[..s.len().min(80)]),
            incoming_bare.as_ref().map(|s| &s[..s.len().min(80)]),
            &prompt_template[..prompt_template.len().min(80)]);

        let mut prompt = if let Some(p) = incoming_prompt {
            eprintln!("[workflow] LLM node '{}': prompt from incoming 'prompt' handle", node_id);
            p
        } else if let Some(s) = incoming_bare {
            eprintln!("[workflow] LLM node '{}': prompt from incoming bare string", node_id);
            s
        } else if prompt_template.contains("{{") {
            let resolved = resolve_template(prompt_template, ctx.node_outputs, ctx.inputs);
            eprintln!("[workflow] LLM node '{}': prompt from template '{}' → '{}'",
                node_id, prompt_template, &resolved[..resolved.len().min(80)]);
            resolved
        } else {
            eprintln!("[workflow] LLM node '{}': prompt from literal template", node_id);
            prompt_template.to_string()
        };

        // Inject context if present and not already in prompt
        if !context_str.is_empty() && !prompt.contains(&context_str) {
            prompt = format!("Context:\n{}\n\nQuestion:\n{}", context_str, prompt);
        }

        if prompt.is_empty() {
            eprintln!("[workflow] LLM node '{}': WARNING — prompt is EMPTY after all resolution!", node_id);
        }

        eprintln!("[workflow] LLM node '{}': FINAL provider={}, model={}, prompt='{}', system='{}'",
            node_id, provider_name, model,
            &prompt[..prompt.len().min(100)], &system_prompt[..system_prompt.len().min(50)]);

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
        eprintln!("[workflow] LLM node '{}': settings → base_url='{}', api_key_len={}, extra_config={:?}",
            node_id, base_url, api_key.len(), extra_config);

        // Collect image data from upstream nodes (File Read binary mode, future File Glob)
        // resolve_source_handle strips metadata (encoding, mime_type) when extracting
        // a specific handle field, so we also scan ctx.node_outputs for the full output.
        let images: Vec<(String, String)> = {
            let extract_image = |obj: &serde_json::Map<String, serde_json::Value>| -> Option<(String, String)> {
                let encoding = obj.get("encoding").and_then(|v| v.as_str()).unwrap_or("");
                let mime = obj.get("mime_type").and_then(|v| v.as_str()).unwrap_or("");
                if encoding == "base64" && mime.starts_with("image/") {
                    let data = obj.get("content").and_then(|v| v.as_str()).unwrap_or("");
                    if !data.is_empty() {
                        return Some((data.to_string(), mime.to_string()));
                    }
                }
                None
            };

            let mut found: Vec<(String, String)> = Vec::new();

            // 1. Check incoming object (handles, nested values)
            if let Some(inc) = incoming.as_ref() {
                if let Some(obj) = inc.as_object() {
                    if let Some(img) = extract_image(obj) {
                        found.push(img);
                    }
                    // Check nested handle values
                    for (_key, val) in obj {
                        if let Some(inner) = val.as_object() {
                            if let Some(img) = extract_image(inner) {
                                found.push(img);
                            }
                        }
                        // Future: File Glob "files" array with multiple images
                        if let Some(arr) = val.as_array() {
                            for item in arr {
                                if let Some(item_obj) = item.as_object() {
                                    if let Some(img) = extract_image(item_obj) {
                                        found.push(img);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Fallback: scan upstream node outputs for image data
            if found.is_empty() {
                for (_nid, output) in ctx.node_outputs.iter() {
                    if let Some(obj) = output.as_object() {
                        if let Some(img) = extract_image(obj) {
                            eprintln!("[workflow] LLM node '{}': found image in upstream node '{}'", node_id, _nid);
                            found.push(img);
                        }
                        // Check "files" array (File Glob output)
                        if let Some(files) = obj.get("files").and_then(|v| v.as_array()) {
                            for file_entry in files {
                                if let Some(file_obj) = file_entry.as_object() {
                                    if let Some(img) = extract_image(file_obj) {
                                        found.push(img);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            found
        };

        // When images detected: if the prompt IS base64 data (leaked through from File Read
        // content → prompt wire), replace with the config prompt or default vision prompt.
        if !images.is_empty() {
            eprintln!("[workflow] LLM node '{}': detected {} image(s), building multimodal message", node_id, images.len());
            if prompt.len() > 100 && !prompt.contains(' ') {
                let config_prompt = node_data.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
                prompt = if !config_prompt.is_empty() && config_prompt != "{{input}}" {
                    config_prompt.to_string()
                } else {
                    "Describe what you see in this image in detail.".to_string()
                };
                eprintln!("[workflow] LLM node '{}': replaced base64 prompt with text: '{}'",
                    node_id, &prompt[..prompt.len().min(80)]);
            }
        }

        let mut body = serde_json::json!({
            "messages": [{ "role": "user", "content": prompt }],
            "provider": provider_name,
            "model": model,
            "temperature": temperature,
        });

        // Attach images for vision models (supports multiple)
        if !images.is_empty() {
            let img_arr: Vec<serde_json::Value> = images.iter().map(|(data, mime)| {
                serde_json::json!({"data": data, "mime_type": mime})
            }).collect();
            body["images"] = serde_json::Value::Array(img_arr);
        }
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

        eprintln!("[workflow] LLM node '{}': POST /chat/direct body={}", node_id,
            &body.to_string()[..body.to_string().len().min(300)]);

        let _ = record_event(ctx.db, ctx.session_id, "llm.request.started", "desktop.workflow",
            serde_json::json!({ "node_id": node_id, "model": model, "provider": provider_name }));

        let resp = ctx.sidecar.proxy_request("POST", "/chat/direct", Some(body)).await
            .map_err(|e| {
                eprintln!("[workflow] ERROR: LLM call failed for node '{}': {}", node_id, e);
                format!("LLM call failed for node '{}': {}", node_id, e)
            })?;

        let content = resp.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let usage = resp.get("usage");
        let input_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
        let output_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
        let resp_model = resp.get("model").and_then(|v| v.as_str()).unwrap_or(model).to_string();

        eprintln!("[workflow] LLM node '{}': response model={}, tokens={}/{}, content='{}'",
            node_id, resp_model, input_tokens, output_tokens,
            &content[..content.len().min(100)]);

        let _ = record_event(ctx.db, ctx.session_id, "llm.response.completed", "desktop.workflow",
            serde_json::json!({
                "node_id": node_id, "model": resp_model, "provider": provider_name,
                "input_tokens": input_tokens, "output_tokens": output_tokens,
            }));

        let cost_usd = (input_tokens as f64 * 0.00000015) + (output_tokens as f64 * 0.0000006);

        Ok(NodeOutput::value(serde_json::json!({
            "response": content,
            "content": content,
            "usage": {
                "total_tokens": input_tokens + output_tokens,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
            },
            "cost": format!("${:.6}", cost_usd),
            "__usage": {
                "total_tokens": input_tokens + output_tokens,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "cost_usd": cost_usd,
            }
        })))
    }
}

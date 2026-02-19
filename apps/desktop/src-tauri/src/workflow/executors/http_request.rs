use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;

pub struct HttpRequestExecutor;

/// Check if a hostname resolves to a private/internal IP range (SSRF protection)
fn is_private_host(host: &str) -> bool {
    let host_lower = host.to_lowercase();
    if host_lower == "localhost" || host_lower == "127.0.0.1" || host_lower == "::1"
        || host_lower == "0.0.0.0" || host_lower == "[::1]" {
        return true;
    }
    // Check common private IP patterns
    if host_lower.starts_with("10.")
        || host_lower.starts_with("192.168.")
        || host_lower.starts_with("169.254.") {
        return true;
    }
    // 172.16.0.0 - 172.31.255.255
    if host_lower.starts_with("172.") {
        if let Some(second) = host_lower.strip_prefix("172.").and_then(|s| s.split('.').next()) {
            if let Ok(n) = second.parse::<u8>() {
                if (16..=31).contains(&n) {
                    return true;
                }
            }
        }
    }
    false
}

#[async_trait::async_trait]
impl NodeExecutor for HttpRequestExecutor {
    fn node_type(&self) -> &str { "http_request" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        // Resolve URL: incoming "url" edge > config url
        let config_url = node_data.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let url = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("url").and_then(|v| v.as_str())
                    .unwrap_or(config_url)
                    .to_string()
            } else if let Some(s) = inc.as_str() {
                s.to_string()
            } else {
                config_url.to_string()
            }
        } else {
            config_url.to_string()
        };

        // Template-resolve URL
        let url = resolve_template(&url, ctx.node_outputs, ctx.inputs);

        if url.is_empty() {
            return Err("HTTP Request: URL is empty".into());
        }

        // SSRF protection: check hostname
        if let Ok(parsed) = url::Url::parse(&url) {
            if let Some(host) = parsed.host_str() {
                if is_private_host(host) {
                    return Err(format!(
                        "HTTP Request blocked: private/internal host '{}' (SSRF protection)",
                        host
                    ));
                }
            }
        }

        let method = node_data.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
        let timeout_secs = node_data.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);
        let max_response_bytes = node_data.get("maxResponseBytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(10_485_760); // 10MB default

        // Build headers: config headers merged with incoming edge headers
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(config_headers) = node_data.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in config_headers {
                if let (Ok(name), Some(val)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    v.as_str()
                ) {
                    if let Ok(hv) = reqwest::header::HeaderValue::from_str(val) {
                        headers.insert(name, hv);
                    }
                }
            }
        }
        if let Some(inc) = incoming {
            if let Some(inc_headers) = inc.as_object().and_then(|o| o.get("headers")).and_then(|v| v.as_object()) {
                for (k, v) in inc_headers {
                    if let (Ok(name), Some(val)) = (
                        reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                        v.as_str()
                    ) {
                        if let Ok(hv) = reqwest::header::HeaderValue::from_str(val) {
                            headers.insert(name, hv);
                        }
                    }
                }
            }
        }

        // Auth handling via settings key (no tokens in graph JSON)
        let auth_type = node_data.get("auth").and_then(|v| v.as_str()).unwrap_or("none");
        if auth_type != "none" {
            let settings_key = node_data.get("authTokenSettingsKey")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !settings_key.is_empty() {
                if let Some(token) = ctx.all_settings.get(settings_key) {
                    let token = token.trim_matches('"');
                    let header_name = node_data.get("authHeader")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Authorization");
                    let auth_value = match auth_type {
                        "bearer" => format!("Bearer {}", token),
                        "basic" => format!("Basic {}", token),
                        "api_key" => token.to_string(),
                        _ => token.to_string(),
                    };
                    if let (Ok(name), Ok(val)) = (
                        reqwest::header::HeaderName::from_bytes(header_name.as_bytes()),
                        reqwest::header::HeaderValue::from_str(&auth_value),
                    ) {
                        headers.insert(name, val);
                    }
                }
            }
        }

        // Build request body
        let body_str = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                if let Some(body) = obj.get("body") {
                    if let Some(s) = body.as_str() { s.to_string() }
                    else { body.to_string() }
                } else {
                    node_data.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string()
                }
            } else {
                node_data.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string()
            }
        } else {
            node_data.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string()
        };
        let body_str = resolve_template(&body_str, ctx.node_outputs, ctx.inputs);

        // Execute request
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| format!("HTTP client error: {e}"))?;

        let mut req = match method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "PATCH" => client.patch(&url),
            "DELETE" => client.delete(&url),
            "HEAD" => client.head(&url),
            _ => return Err(format!("Unsupported HTTP method: {}", method)),
        };
        req = req.headers(headers);

        if !body_str.is_empty() && matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") {
            req = req.body(body_str);
        }

        let response = req.send().await.map_err(|e| format!("HTTP request failed: {e}"))?;

        let status = response.status().as_u16();
        let resp_headers: serde_json::Map<String, serde_json::Value> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_str().unwrap_or("").to_string())))
            .collect();

        // Check content-length before reading body
        if let Some(cl) = response.content_length() {
            if cl > max_response_bytes {
                return Err(format!("Response too large: {} bytes > {} byte limit", cl, max_response_bytes));
            }
        }

        let body = response.text().await.map_err(|e| format!("Failed to read response body: {e}"))?;

        if body.len() as u64 > max_response_bytes {
            return Err(format!("Response too large: {} bytes > {} byte limit", body.len(), max_response_bytes));
        }

        Ok(NodeOutput::value(serde_json::json!({
            "body": body,
            "status": status,
            "headers": resp_headers,
        })))
    }
}

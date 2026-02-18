use crate::db::{Database, now_iso};
use crate::error::AppError;
use crate::events::record_event;
use super::budget::{get_budget_remaining_pct, get_current_month_cost};
use super::sessions::Message;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub session_id: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    pub user_message: Message,
    pub assistant_message: Message,
}

#[tauri::command]
pub async fn send_message(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    request: SendMessageRequest,
) -> Result<SendMessageResponse, AppError> {
    let now = now_iso();

    // 1. Load session + agent info + provider config + routing config from settings
    let (mut provider, mut model, system_prompt, tools_mode, tools, routing_mode, routing_rules, all_settings) = {
        let conn = db.conn.lock()?;
        let agent_id: String = conn
            .query_row(
                "SELECT agent_id FROM sessions WHERE id = ?1",
                params![request.session_id],
                |row| row.get(0),
            )
            .map_err(|_| AppError::NotFound("Session not found".into()))?;

        let (provider, model, system_prompt, tools_mode, tools_json, routing_mode, routing_rules_json): (String, String, String, String, String, String, String) = conn.query_row(
            "SELECT provider, model, system_prompt, tools_mode, tools, routing_mode, routing_rules FROM agents WHERE id = ?1",
            params![agent_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        )
        .map_err(|_| AppError::NotFound("Agent not found".into()))?;

        let tools: Vec<String> = serde_json::from_str(&tools_json).unwrap_or_default();
        let routing_rules: Vec<serde_json::Value> = serde_json::from_str(&routing_rules_json).unwrap_or_default();

        let mut all_settings = std::collections::HashMap::new();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, value))
            })?;
        for row in rows {
            let (key, value) = row?;
            all_settings.insert(key, value);
        }

        (provider, model, system_prompt, tools_mode, tools, routing_mode, routing_rules, all_settings)
    };

    // 1b. Smart Router — pick the best model for this request
    let available_providers = crate::routing::get_available_providers(&all_settings);
    let context_tokens = request.content.len() / 4;
    let budget_remaining_pct = get_budget_remaining_pct(db.inner(), &all_settings);

    let routing_decision = crate::routing::route(&crate::routing::RoutingInput {
        message: &request.content,
        context_tokens,
        has_images: false,
        tools: &tools,
        routing_mode: &routing_mode,
        routing_rules: &routing_rules,
        default_provider: &provider,
        default_model: &model,
        budget_remaining_pct,
        available_providers: &available_providers,
    });

    provider = routing_decision.provider.clone();
    model = routing_decision.model.clone();

    // 1c. Budget enforcement — block or override BEFORE calling sidecar
    if budget_remaining_pct <= 0.0 {
        let exhausted_behavior = all_settings
            .get("budget.exhausted_behavior")
            .map(|v| v.trim_matches('"').to_string())
            .unwrap_or_else(|| "none".to_string());

        match exhausted_behavior.as_str() {
            "local_only" => {
                if available_providers.iter().any(|p| p == "ollama") {
                    provider = "ollama".to_string();
                    model = "llama3.2".to_string();
                } else {
                    return Err(AppError::BudgetExhausted(
                        "Monthly budget exhausted. Local model (Ollama) not available.".into(),
                    ));
                }
            }
            "cheapest_cloud" => {
                if available_providers.iter().any(|p| p == "google") {
                    provider = "google".to_string();
                    model = "gemini-2.0-flash".to_string();
                } else if available_providers.iter().any(|p| p == "ollama") {
                    provider = "ollama".to_string();
                    model = "llama3.2".to_string();
                }
                // else: proceed with whatever the router picked
            }
            "ask" => {
                return Err(AppError::BudgetExhausted(
                    "Monthly budget exhausted. Please increase your budget limit or switch to a local model.".into(),
                ));
            }
            _ => {} // "none" — no enforcement, proceed normally
        }
    }

    let provider_config = {
        let prefix = format!("provider.{}.", provider);
        let mut config = serde_json::Map::new();
        for (k, v) in &all_settings {
            if let Some(field) = k.strip_prefix(&prefix) {
                let clean_value = v.trim_matches('"').to_string();
                config.insert(field.to_string(), serde_json::Value::String(clean_value));
            }
        }
        config
    };

    // 2. Get next sequence number
    let user_seq = {
        let conn = db.conn.lock()?;
        let max_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) FROM messages WHERE session_id = ?1",
                params![request.session_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        max_seq + 1
    };

    // 3. Persist user message
    let user_msg_id = Uuid::new_v4().to_string();
    {
        let conn = db.conn.lock()?;
        conn.execute(
            "INSERT INTO messages (id, session_id, seq, role, content, created_at)
             VALUES (?1, ?2, ?3, 'user', ?4, ?5)",
            params![user_msg_id, request.session_id, user_seq, request.content, now],
        )
        .map_err(|e| AppError::Db(format!("Failed to save user message: {e}")))?;
    }

    let user_message = Message {
        id: user_msg_id,
        session_id: request.session_id.clone(),
        seq: user_seq,
        role: "user".to_string(),
        content: request.content.clone(),
        model: None,
        provider: None,
        input_tokens: None,
        output_tokens: None,
        cost_usd: None,
        duration_ms: None,
        created_at: now.clone(),
    };

    // 4. Load full message history from SQLite
    let history: Vec<serde_json::Value> = {
        let conn = db.conn.lock()?;
        let mut stmt = conn.prepare(
                "SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY seq ASC",
            )?;
        let result = stmt.query_map(params![request.session_id], |row| {
            Ok(serde_json::json!({
                "role": row.get::<_, String>(0)?,
                "content": row.get::<_, String>(1)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        result
    };

    // 5. Record events
    record_event(db.inner(), &request.session_id, "message.user", "ui.user",
        serde_json::json!({ "content": request.content }))?;

    if routing_mode != "single" {
        record_event(db.inner(), &request.session_id, "llm.routed", "desktop.router",
            serde_json::json!({
                "chosen_model": routing_decision.model,
                "chosen_provider": routing_decision.provider,
                "reason": routing_decision.reason,
                "estimated_savings": routing_decision.estimated_savings,
                "alternatives_considered": routing_decision.alternatives_considered,
            }))?;
    }

    record_event(db.inner(), &request.session_id, "llm.request.started", "desktop.chat",
        serde_json::json!({ "model": model, "provider": provider }))?;

    // 6. Call sidecar for real LLM response
    let llm_start = std::time::Instant::now();
    let api_key = provider_config.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let base_url = provider_config.get("base_url")
        .or_else(|| provider_config.get("endpoint"))
        .and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut extra_config = serde_json::Map::new();
    for (k, v) in &provider_config {
        if k != "api_key" && k != "base_url" && k != "endpoint" {
            extra_config.insert(k.clone(), v.clone());
        }
    }

    let tools_enabled = tools_mode != "sandboxed";
    let mut chat_body = serde_json::json!({
        "message": request.content,
        "conversation_id": request.session_id,
        "provider": provider,
        "model": model,
        "system_prompt": system_prompt,
        "tools_enabled": tools_enabled,
        "history": history,
    });
    if !api_key.is_empty() {
        chat_body["api_key"] = serde_json::Value::String(api_key);
    }
    if !base_url.is_empty() {
        chat_body["base_url"] = serde_json::Value::String(base_url);
    }
    if !extra_config.is_empty() {
        chat_body["extra_config"] = serde_json::Value::Object(extra_config);
    }

    let resp = sidecar.proxy_request("POST", "/chat", Some(chat_body)).await
        .map_err(|e| {
            let _ = record_event(db.inner(), &request.session_id, "agent.error", "desktop.chat",
                serde_json::json!({ "error": format!("{e}"), "error_code": "SidecarRequestFailed", "severity": "error" }));
            AppError::Sidecar(format!("LLM call failed: {e}"))
        })?;

    let duration_ms = llm_start.elapsed().as_millis() as i64;
    let content = resp.get("content").and_then(|v| v.as_str()).unwrap_or("(no response)").to_string();
    let usage = resp.get("usage");
    let input_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let output_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let response_model = resp.get("model").and_then(|v| v.as_str()).unwrap_or(&model).to_string();

    // 6b. Record tool call events
    if let Some(tool_calls) = resp.get("tool_calls").and_then(|v| v.as_array()) {
        for tc in tool_calls {
            let tool_name = tc.get("tool_name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let tool_input = tc.get("tool_input").cloned().unwrap_or(serde_json::json!({}));
            let tool_output = tc.get("tool_output").and_then(|v| v.as_str()).unwrap_or("");
            let tool_duration = tc.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tool_error = tc.get("error").and_then(|v| v.as_str());
            let tool_call_id = tc.get("tool_call_id").and_then(|v| v.as_str()).unwrap_or("");

            record_event(db.inner(), &request.session_id, "tool.requested", "sidecar.chat",
                serde_json::json!({
                    "tool_call_id": tool_call_id,
                    "tool_name": tool_name,
                    "tool_input": tool_input,
                }))?;

            if let Some(err) = tool_error {
                record_event(db.inner(), &request.session_id, "tool.error", "sidecar.chat",
                    serde_json::json!({
                        "tool_call_id": tool_call_id,
                        "tool_name": tool_name,
                        "error": err,
                        "duration_ms": tool_duration,
                    }))?;
            } else {
                record_event(db.inner(), &request.session_id, "tool.completed", "sidecar.chat",
                    serde_json::json!({
                        "tool_call_id": tool_call_id,
                        "tool_name": tool_name,
                        "tool_output": tool_output,
                        "duration_ms": tool_duration,
                    }))?;
            }
        }
    }

    // 7. Persist assistant message
    let assistant_seq = user_seq + 1;
    let assistant_msg_id = Uuid::new_v4().to_string();
    let resp_now = now_iso();
    {
        let conn = db.conn.lock()?;
        conn.execute(
            "INSERT INTO messages (id, session_id, seq, role, content, model, provider,
                                   input_tokens, output_tokens, duration_ms, created_at)
             VALUES (?1, ?2, ?3, 'assistant', ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                assistant_msg_id, request.session_id, assistant_seq,
                content, response_model, provider,
                input_tokens, output_tokens, duration_ms, resp_now,
            ],
        )
        .map_err(|e| AppError::Db(format!("Failed to save assistant message: {e}")))?;

        conn.execute(
            "UPDATE sessions SET
                message_count = message_count + 2,
                total_input_tokens = total_input_tokens + ?1,
                total_output_tokens = total_output_tokens + ?2,
                updated_at = ?3
             WHERE id = ?4",
            params![input_tokens, output_tokens, resp_now, request.session_id],
        )
        .map_err(|e| AppError::Db(format!("Failed to update session: {e}")))?;
    }

    let assistant_message = Message {
        id: assistant_msg_id,
        session_id: request.session_id.clone(),
        seq: assistant_seq,
        role: "assistant".to_string(),
        content: content.clone(),
        model: Some(response_model.clone()),
        provider: Some(provider.clone()),
        input_tokens: Some(input_tokens),
        output_tokens: Some(output_tokens),
        cost_usd: None,
        duration_ms: Some(duration_ms),
        created_at: resp_now,
    };

    // 8. Record completion events
    record_event(db.inner(), &request.session_id, "llm.response.completed", "desktop.chat",
        serde_json::json!({
            "model": response_model, "provider": provider,
            "input_tokens": input_tokens, "output_tokens": output_tokens,
            "duration_ms": duration_ms,
        }))?;
    record_event(db.inner(), &request.session_id, "message.assistant", "desktop.chat",
        serde_json::json!({ "content": content, "model": response_model }))?;

    // 9. Check budget thresholds
    let budget_pct_after = get_budget_remaining_pct(db.inner(), &all_settings);
    if budget_pct_after < 100.0 {
        let used_pct = 100.0 - budget_pct_after;
        let threshold = if used_pct >= 100.0 {
            Some("100_percent")
        } else if used_pct >= 80.0 && budget_remaining_pct > 20.0 {
            Some("80_percent")
        } else if used_pct >= 50.0 && budget_remaining_pct > 50.0 {
            Some("50_percent")
        } else {
            None
        };

        if let Some(level) = threshold {
            let limit = all_settings
                .get("budget.monthly_limit")
                .and_then(|v| v.trim_matches('"').parse::<f64>().ok())
                .unwrap_or(0.0);
            let used_amount = get_current_month_cost(db.inner()).unwrap_or(0.0);
            let _ = record_event(db.inner(), &request.session_id, "budget.warning", "desktop.budget",
                serde_json::json!({
                    "level": level,
                    "budget": limit,
                    "used": used_amount,
                    "remaining": (limit - used_amount).max(0.0),
                }));
        }
    }

    Ok(SendMessageResponse { user_message, assistant_message })
}

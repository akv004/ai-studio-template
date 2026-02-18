use crate::db::{Database, now_iso};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: String,
    pub agent_id: String,
    pub session_id: Option<String>,
    pub name: String,
    pub input: String,
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub total_events: i64,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
    pub duration_ms: Option<i64>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub agent_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRunRequest {
    pub agent_id: String,
    pub input: String,
    pub name: Option<String>,
}

#[tauri::command]
pub fn list_runs(db: tauri::State<'_, Database>) -> Result<Vec<Run>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn.prepare(
            "SELECT r.id, r.agent_id, r.session_id, r.name, r.input, r.status,
                    r.output, r.error, r.total_events, r.total_tokens,
                    r.total_cost_usd, r.duration_ms, r.created_at,
                    r.started_at, r.completed_at, a.name
             FROM runs r
             LEFT JOIN agents a ON a.id = r.agent_id
             ORDER BY r.created_at DESC",
        )?;

    let runs = stmt
        .query_map([], |row| {
            Ok(Run {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                session_id: row.get(2)?,
                name: row.get(3)?,
                input: row.get(4)?,
                status: row.get(5)?,
                output: row.get(6)?,
                error: row.get(7)?,
                total_events: row.get(8)?,
                total_tokens: row.get(9)?,
                total_cost_usd: row.get(10)?,
                duration_ms: row.get(11)?,
                created_at: row.get(12)?,
                started_at: row.get(13)?,
                completed_at: row.get(14)?,
                agent_name: row.get(15)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(runs)
}

#[tauri::command]
pub async fn create_run(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    app: tauri::AppHandle,
    request: CreateRunRequest,
) -> Result<Run, AppError> {
    let run_id = Uuid::new_v4().to_string();
    let now = now_iso();

    let (agent_name, provider, model, system_prompt) = {
        let conn = db.conn.lock()?;
        conn.query_row(
            "SELECT name, provider, model, system_prompt FROM agents WHERE id = ?1 AND is_archived = 0",
            params![request.agent_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            )),
        )
        .map_err(|_| AppError::NotFound("Agent not found".into()))?
    };

    let session_id = Uuid::new_v4().to_string();
    {
        let conn = db.conn.lock()?;
        let session_title = format!("Run: {}", request.name.as_deref().unwrap_or(&request.input[..request.input.len().min(50)]));
        conn.execute(
            "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
            params![session_id, request.agent_id, session_title, now, now],
        )
        .map_err(|e| AppError::Db(format!("Failed to create run session: {e}")))?;
    }

    let run_name = request.name.unwrap_or_else(|| {
        if request.input.len() > 60 {
            format!("{}...", &request.input[..57])
        } else {
            request.input.clone()
        }
    });

    {
        let conn = db.conn.lock()?;
        conn.execute(
            "INSERT INTO runs (id, agent_id, session_id, name, input, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
            params![run_id, request.agent_id, session_id, run_name, request.input, now],
        )
        .map_err(|e| AppError::Db(format!("Failed to create run: {e}")))?;
    }

    let run = Run {
        id: run_id.clone(),
        agent_id: request.agent_id.clone(),
        session_id: Some(session_id.clone()),
        name: run_name.clone(),
        input: request.input.clone(),
        status: "pending".to_string(),
        output: None,
        error: None,
        total_events: 0,
        total_tokens: 0,
        total_cost_usd: 0.0,
        duration_ms: None,
        created_at: now.clone(),
        started_at: None,
        completed_at: None,
        agent_name: Some(agent_name),
    };

    let provider_config = {
        let conn = db.conn.lock()?;
        let prefix = format!("provider.{}.", provider);
        let mut stmt = conn.prepare("SELECT key, value FROM settings WHERE key LIKE ?1")?;
        let mut config = serde_json::Map::new();
        let rows = stmt.query_map(params![format!("{}%", prefix)], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, value))
            })?;
        for row in rows {
            let (key, value) = row?;
            let field = key.strip_prefix(&prefix).unwrap_or(&key);
            let clean_value = value.trim_matches('"').to_string();
            config.insert(field.to_string(), serde_json::Value::String(clean_value));
        }
        config
    };

    let db_clone = db.inner().clone();
    let sidecar_clone = sidecar.inner().clone();
    let run_id_bg = run_id.clone();
    let session_id_bg = session_id;
    let input_bg = request.input.clone();
    let agent_id_bg = request.agent_id;

    tauri::async_runtime::spawn(async move {
        execute_run(
            &db_clone, &sidecar_clone, &app,
            &run_id_bg, &session_id_bg, &agent_id_bg,
            &input_bg, &provider, &model, &system_prompt,
            &provider_config,
        ).await;
    });

    Ok(run)
}

// Background task — uses if-let pattern (no ? propagation needed)
async fn execute_run(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    app: &tauri::AppHandle,
    run_id: &str,
    session_id: &str,
    _agent_id: &str,
    input: &str,
    provider: &str,
    model: &str,
    system_prompt: &str,
    provider_config: &serde_json::Map<String, serde_json::Value>,
) {
    let started_at = now_iso();

    {
        if let Ok(conn) = db.conn.lock() {
            let _ = conn.execute(
                "UPDATE runs SET status = 'running', started_at = ?1 WHERE id = ?2",
                params![started_at, run_id],
            );
        }
    }

    let _ = app.emit("run_status_changed", serde_json::json!({
        "runId": run_id, "status": "running",
    }));

    let start_time = std::time::Instant::now();

    let api_key = provider_config.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let base_url = provider_config.get("base_url")
        .or_else(|| provider_config.get("endpoint"))
        .and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut extra_config = serde_json::Map::new();
    for (k, v) in provider_config {
        if k != "api_key" && k != "base_url" && k != "endpoint" {
            extra_config.insert(k.clone(), v.clone());
        }
    }

    let mut chat_body = serde_json::json!({
        "message": input,
        "conversation_id": session_id,
        "provider": provider,
        "model": model,
        "system_prompt": system_prompt,
        "tools_enabled": true,
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

    let result = sidecar.proxy_request("POST", "/chat", Some(chat_body)).await;
    let duration_ms = start_time.elapsed().as_millis() as i64;
    let completed_at = now_iso();

    match result {
        Ok(resp) => {
            let content = resp.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let usage = resp.get("usage");
            let input_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
            let output_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
            let total_tokens = input_tokens + output_tokens;

            if let Ok(conn) = db.conn.lock() {
                let _ = conn.execute(
                    "UPDATE runs SET status = 'completed', output = ?1, total_tokens = ?2,
                     duration_ms = ?3, completed_at = ?4
                     WHERE id = ?5 AND status = 'running'",
                    params![content, total_tokens, duration_ms, completed_at, run_id],
                );
            }

            let _ = app.emit("run_status_changed", serde_json::json!({
                "runId": run_id, "status": "completed",
            }));
        }
        Err(e) => {
            if let Ok(conn) = db.conn.lock() {
                let _ = conn.execute(
                    "UPDATE runs SET status = 'failed', error = ?1, duration_ms = ?2,
                     completed_at = ?3
                     WHERE id = ?4 AND status = 'running'",
                    params![e.to_string(), duration_ms, completed_at, run_id],
                );
            }

            let _ = app.emit("run_status_changed", serde_json::json!({
                "runId": run_id, "status": "failed", "error": e.to_string(),
            }));
        }
    }
}

#[tauri::command]
pub fn cancel_run(db: tauri::State<'_, Database>, id: String) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    let now = now_iso();
    let rows = conn
        .execute(
            "UPDATE runs SET status = 'cancelled', completed_at = ?1 WHERE id = ?2 AND status IN ('pending', 'running')",
            params![now, id],
        )
        .map_err(|e| AppError::Db(format!("Failed to cancel run: {e}")))?;
    if rows == 0 {
        return Err(AppError::NotFound("Run not found or already completed".into()));
    }
    Ok(())
}

#[tauri::command]
pub fn get_run(db: tauri::State<'_, Database>, id: String) -> Result<Run, AppError> {
    let conn = db.conn.lock()?;
    conn.query_row(
        "SELECT r.id, r.agent_id, r.session_id, r.name, r.input, r.status,
                r.output, r.error, r.total_events, r.total_tokens,
                r.total_cost_usd, r.duration_ms, r.created_at,
                r.started_at, r.completed_at, a.name
         FROM runs r
         LEFT JOIN agents a ON a.id = r.agent_id
         WHERE r.id = ?1",
        params![id],
        |row| {
            Ok(Run {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                session_id: row.get(2)?,
                name: row.get(3)?,
                input: row.get(4)?,
                status: row.get(5)?,
                output: row.get(6)?,
                error: row.get(7)?,
                total_events: row.get(8)?,
                total_tokens: row.get(9)?,
                total_cost_usd: row.get(10)?,
                duration_ms: row.get(11)?,
                created_at: row.get(12)?,
                started_at: row.get(13)?,
                completed_at: row.get(14)?,
                agent_name: row.get(15)?,
            })
        },
    )
    .map_err(|e| AppError::NotFound(format!("Run not found: {e}")))
}

use crate::db::{Database, now_iso};
use crate::error::AppError;
use crate::sidecar::SidecarManager;
use crate::webhook::auth::AuthMode;
use crate::webhook::server::{ResponseMode, WebhookRoute};
use crate::webhook::{TriggerManager, WebhookServerStatus};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub id: String,
    pub workflow_id: String,
    pub trigger_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub last_fired: Option<String>,
    pub fire_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerLogEntry {
    pub id: String,
    pub trigger_id: String,
    pub run_id: Option<String>,
    pub fired_at: String,
    pub status: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTriggerRequest {
    pub workflow_id: String,
    pub trigger_type: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTriggerRequest {
    pub trigger_id: String,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[tauri::command]
pub fn create_trigger(
    db: tauri::State<'_, Database>,
    request: CreateTriggerRequest,
) -> Result<Trigger, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let config_str = serde_json::to_string(&request.config)
        .map_err(|e| AppError::Validation(format!("Invalid config: {e}")))?;

    let conn = db.conn.lock()?;
    // Verify workflow exists
    conn.query_row(
        "SELECT id FROM workflows WHERE id = ?1 AND is_archived = 0",
        params![request.workflow_id],
        |_| Ok(()),
    ).map_err(|_| AppError::NotFound("Workflow not found".into()))?;

    conn.execute(
        "INSERT INTO triggers (id, workflow_id, trigger_type, config, enabled, fire_count, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 1, 0, ?5, ?6)",
        params![id, request.workflow_id, request.trigger_type, config_str, now, now],
    ).map_err(|e| AppError::Db(format!("Failed to create trigger: {e}")))?;

    Ok(Trigger {
        id,
        workflow_id: request.workflow_id,
        trigger_type: request.trigger_type,
        config: request.config,
        enabled: true,
        last_fired: None,
        fire_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_trigger(
    db: tauri::State<'_, Database>,
    request: UpdateTriggerRequest,
) -> Result<Trigger, AppError> {
    let now = now_iso();
    let conn = db.conn.lock()?;

    // Load current
    let (mut config_str, mut enabled): (String, bool) = conn.query_row(
        "SELECT config, enabled FROM triggers WHERE id = ?1",
        params![request.trigger_id],
        |row| Ok((row.get(0)?, row.get::<_, bool>(1)?)),
    ).map_err(|_| AppError::NotFound("Trigger not found".into()))?;

    if let Some(new_config) = &request.config {
        config_str = serde_json::to_string(new_config)
            .map_err(|e| AppError::Validation(format!("Invalid config: {e}")))?;
    }
    if let Some(new_enabled) = request.enabled {
        enabled = new_enabled;
    }

    conn.execute(
        "UPDATE triggers SET config = ?1, enabled = ?2, updated_at = ?3 WHERE id = ?4",
        params![config_str, enabled, now, request.trigger_id],
    ).map_err(|e| AppError::Db(format!("Failed to update trigger: {e}")))?;

    // Re-read full record
    drop(conn);
    get_trigger_by_id(&db, &request.trigger_id)
}

#[tauri::command]
pub async fn delete_trigger(
    db: tauri::State<'_, Database>,
    trigger_mgr: tauri::State<'_, TriggerManager>,
    trigger_id: String,
) -> Result<(), AppError> {
    // Disarm if armed
    let path = {
        let conn = db.conn.lock()?;
        let config_str: String = conn.query_row(
            "SELECT config FROM triggers WHERE id = ?1",
            params![trigger_id],
            |row| row.get(0),
        ).map_err(|_| AppError::NotFound("Trigger not found".into()))?;
        let config: serde_json::Value = serde_json::from_str(&config_str).unwrap_or_default();
        config.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string()
    };

    if !path.is_empty() && trigger_mgr.is_armed(&path) {
        trigger_mgr.disarm_webhook(&path)
            .map_err(|e| AppError::Workflow(e))?;
    }

    let conn = db.conn.lock()?;
    conn.execute("DELETE FROM triggers WHERE id = ?1", params![trigger_id])
        .map_err(|e| AppError::Db(format!("Failed to delete trigger: {e}")))?;

    Ok(())
}

#[tauri::command]
pub fn list_triggers(
    db: tauri::State<'_, Database>,
    workflow_id: Option<String>,
) -> Result<Vec<Trigger>, AppError> {
    let conn = db.conn.lock()?;
    let mut triggers = Vec::new();

    let (sql, param): (String, Vec<String>) = match &workflow_id {
        Some(wid) => (
            "SELECT id, workflow_id, trigger_type, config, enabled, last_fired, fire_count, created_at, updated_at FROM triggers WHERE workflow_id = ?1 ORDER BY created_at DESC".into(),
            vec![wid.clone()],
        ),
        None => (
            "SELECT id, workflow_id, trigger_type, config, enabled, last_fired, fire_count, created_at, updated_at FROM triggers ORDER BY created_at DESC".into(),
            vec![],
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = if param.is_empty() {
        stmt.query_map([], row_to_trigger)?
    } else {
        stmt.query_map(params![param[0]], row_to_trigger)?
    };

    for row in rows {
        triggers.push(row?);
    }
    Ok(triggers)
}

fn row_to_trigger(row: &rusqlite::Row) -> rusqlite::Result<Trigger> {
    let config_str: String = row.get(3)?;
    let config: serde_json::Value = serde_json::from_str(&config_str).unwrap_or_default();
    Ok(Trigger {
        id: row.get(0)?,
        workflow_id: row.get(1)?,
        trigger_type: row.get(2)?,
        config,
        enabled: row.get(4)?,
        last_fired: row.get(5)?,
        fire_count: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

#[tauri::command]
pub fn get_trigger_log(
    db: tauri::State<'_, Database>,
    trigger_id: String,
    limit: Option<u32>,
) -> Result<Vec<TriggerLogEntry>, AppError> {
    let limit = limit.unwrap_or(50).min(500);
    let conn = db.conn.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, trigger_id, run_id, fired_at, status, metadata
         FROM trigger_log WHERE trigger_id = ?1
         ORDER BY fired_at DESC LIMIT ?2"
    )?;

    let rows = stmt.query_map(params![trigger_id, limit], |row| {
        let meta_str: String = row.get(5)?;
        let metadata: serde_json::Value = serde_json::from_str(&meta_str).unwrap_or_default();
        Ok(TriggerLogEntry {
            id: row.get(0)?,
            trigger_id: row.get(1)?,
            run_id: row.get(2)?,
            fired_at: row.get(3)?,
            status: row.get(4)?,
            metadata,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

#[tauri::command]
pub async fn arm_trigger(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, SidecarManager>,
    trigger_mgr: tauri::State<'_, TriggerManager>,
    app: tauri::AppHandle,
    trigger_id: String,
) -> Result<(), AppError> {
    let trigger = get_trigger_by_id(&db, &trigger_id)?;

    if !trigger.enabled {
        return Err(AppError::Validation("Trigger is disabled".into()));
    }
    if trigger.trigger_type != "webhook" {
        return Err(AppError::Validation(format!("Cannot arm trigger type '{}' — only webhook supported", trigger.trigger_type)));
    }

    let path = trigger.config.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Webhook config missing 'path' field".into()))?
        .to_string();

    if path.is_empty() {
        return Err(AppError::Validation("Webhook path cannot be empty".into()));
    }

    // Check if port override exists in settings
    {
        let conn = db.conn.lock()?;
        if let Ok(port_str) = conn.query_row(
            "SELECT value FROM settings WHERE key = 'webhook.port'",
            [], |row| row.get::<_, String>(0),
        ) {
            if let Ok(port) = port_str.trim_matches('"').parse::<u16>() {
                trigger_mgr.set_port(port);
            }
        }
    }

    let methods: Vec<String> = trigger.config.get("methods")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec!["POST".to_string()]);

    let timeout_secs = trigger.config.get("timeoutSecs")
        .and_then(|v| v.as_u64())
        .unwrap_or(30);

    let max_per_minute = trigger.config.get("maxPerMinute")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let response_mode = trigger.config.get("responseMode")
        .and_then(|v| v.as_str())
        .unwrap_or("immediate");

    // Validate auth secrets are not empty when auth is configured
    let auth_mode_str = trigger.config.get("authMode").and_then(|v| v.as_str()).unwrap_or("none");
    match auth_mode_str {
        "token" => {
            let token = trigger.config.get("authToken").and_then(|v| v.as_str()).unwrap_or("");
            if token.is_empty() {
                return Err(AppError::Validation("Cannot arm: authMode is 'token' but authToken is empty".into()));
            }
        }
        "hmac" => {
            let secret = trigger.config.get("hmacSecret").and_then(|v| v.as_str()).unwrap_or("");
            if secret.is_empty() {
                return Err(AppError::Validation("Cannot arm: authMode is 'hmac' but hmacSecret is empty".into()));
            }
        }
        _ => {}
    }

    let route = WebhookRoute {
        trigger_id: trigger.id.clone(),
        workflow_id: trigger.workflow_id.clone(),
        auth_mode: AuthMode::from_config(&trigger.config),
        response_mode: ResponseMode::from_str(response_mode),
        timeout_secs,
        methods,
        max_per_minute,
    };

    trigger_mgr.arm_webhook(&path, route, db.inner(), sidecar.inner(), &app).await
        .map_err(|e| AppError::Workflow(e))?;

    eprintln!("[triggers] Armed webhook: trigger_id={}, path={}", trigger.id, path);
    Ok(())
}

#[tauri::command]
pub async fn disarm_trigger(
    db: tauri::State<'_, Database>,
    trigger_mgr: tauri::State<'_, TriggerManager>,
    trigger_id: String,
) -> Result<(), AppError> {
    let trigger = get_trigger_by_id(&db, &trigger_id)?;

    let path = trigger.config.get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if path.is_empty() {
        return Err(AppError::Validation("Trigger has no path configured".into()));
    }

    trigger_mgr.disarm_webhook(&path)
        .map_err(|e| AppError::Workflow(e))?;

    eprintln!("[triggers] Disarmed webhook: trigger_id={}, path={}", trigger_id, path);
    Ok(())
}

#[tauri::command]
pub async fn test_trigger(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, SidecarManager>,
    app: tauri::AppHandle,
    trigger_id: String,
) -> Result<serde_json::Value, AppError> {
    let trigger = get_trigger_by_id(&db, &trigger_id)?;

    // Build mock webhook inputs
    let mut inputs = std::collections::HashMap::new();
    inputs.insert("__webhook_body".to_string(), serde_json::json!({"test": true, "trigger_id": trigger.id}));
    inputs.insert("__webhook_headers".to_string(), serde_json::json!({"content-type": "application/json", "x-test": "true"}));
    inputs.insert("__webhook_query".to_string(), serde_json::json!({}));
    inputs.insert("__webhook_method".to_string(), serde_json::json!("POST"));
    inputs.insert("input".to_string(), serde_json::json!({"test": true, "trigger_id": trigger.id}));

    // Load workflow
    let (graph_json, agent_id) = {
        let conn = db.conn.lock()?;
        let (graph, wf_agent_id): (String, Option<String>) = conn.query_row(
            "SELECT graph_json, agent_id FROM workflows WHERE id = ?1 AND is_archived = 0",
            params![trigger.workflow_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| AppError::NotFound(format!("Workflow not found: {e}")))?;

        let agent = wf_agent_id.filter(|id| !id.is_empty()).unwrap_or_else(|| {
            conn.query_row(
                "SELECT id FROM agents WHERE is_archived = 0 ORDER BY created_at LIMIT 1",
                [], |row| row.get::<_, String>(0),
            ).unwrap_or_default()
        });

        (graph, agent)
    };

    // Load settings
    let all_settings = {
        let conn = db.conn.lock()?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let mut settings = std::collections::HashMap::<String, String>::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (k, v) = row?;
            settings.insert(k, v);
        }
        settings
    };

    // Create test session
    let session_id = Uuid::new_v4().to_string();
    let now = now_iso();
    {
        let conn = db.conn.lock()?;
        conn.execute(
            "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
             VALUES (?1, ?2, 'Webhook Test', 'active', ?3, ?4)",
            params![session_id, agent_id, now, now],
        ).map_err(|e| AppError::Db(format!("Failed to create session: {e}")))?;
    }

    // Execute
    let db_clone = db.inner().clone();
    let sidecar_clone = sidecar.inner().clone();
    let result = crate::workflow::engine::execute_workflow_ephemeral(
        &db_clone, &sidecar_clone, &app,
        &session_id, &graph_json, &inputs, &all_settings, false,
    ).await.map_err(|e| AppError::Workflow(e))?;

    Ok(serde_json::json!({
        "sessionId": session_id,
        "status": result.status,
        "outputs": result.outputs,
        "durationMs": result.duration_ms,
        "error": result.error,
    }))
}

#[tauri::command]
pub fn get_webhook_server_status(
    trigger_mgr: tauri::State<'_, TriggerManager>,
) -> Result<WebhookServerStatus, AppError> {
    Ok(trigger_mgr.status())
}

fn get_trigger_by_id(db: &Database, trigger_id: &str) -> Result<Trigger, AppError> {
    let conn = db.conn.lock()?;
    conn.query_row(
        "SELECT id, workflow_id, trigger_type, config, enabled, last_fired, fire_count, created_at, updated_at FROM triggers WHERE id = ?1",
        params![trigger_id],
        row_to_trigger,
    ).map_err(|_| AppError::NotFound("Trigger not found".into()))
}

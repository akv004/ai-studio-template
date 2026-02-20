pub mod types;
pub mod validation;
pub mod engine;
pub mod executors;
pub mod live;

use crate::db::{Database, now_iso};
use crate::error::AppError;
use types::{RunWorkflowRequest, WorkflowRunResult, ValidationResult};
use validation::validate_graph_json;
use engine::execute_workflow;
use rusqlite::params;
use uuid::Uuid;

#[tauri::command]
pub fn validate_workflow(db: tauri::State<'_, Database>, id: String) -> Result<ValidationResult, AppError> {
    let conn = db.conn.lock()?;
    let graph_json: String = conn
        .query_row(
            "SELECT graph_json FROM workflows WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::NotFound(format!("Workflow not found: {e}")))?;

    validate_graph_json(&graph_json).map_err(|e| AppError::Validation(e))
}

#[tauri::command]
pub async fn run_workflow(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    app: tauri::AppHandle,
    request: RunWorkflowRequest,
) -> Result<WorkflowRunResult, AppError> {
    eprintln!("[workflow] === RUN START === workflow_id={}, input_keys={:?}",
        request.workflow_id, request.inputs.keys().collect::<Vec<_>>());

    // 1. Load workflow
    let (workflow_name, graph_json, workflow_agent_id) = {
        let conn = db.conn.lock()?;
        conn.query_row(
            "SELECT name, graph_json, agent_id FROM workflows WHERE id = ?1 AND is_archived = 0",
            params![request.workflow_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?)),
        )
        .map_err(|e| {
            eprintln!("[workflow] ERROR: Workflow not found: {e}");
            AppError::NotFound(format!("Workflow not found: {e}"))
        })?
    };
    eprintln!("[workflow] Loaded '{}', agent_id={:?}", workflow_name, workflow_agent_id);

    // 2. Validate
    let validation = validate_graph_json(&graph_json).map_err(|e| AppError::Validation(e))?;
    if !validation.valid {
        eprintln!("[workflow] ERROR: Validation failed: {}", validation.errors.join("; "));
        return Err(AppError::Validation(format!("Invalid workflow: {}", validation.errors.join("; "))));
    }

    // 3. Create a session for this workflow run
    let agent_id = match workflow_agent_id {
        Some(ref id) if !id.is_empty() => {
            eprintln!("[workflow] Using workflow agent_id: {}", id);
            id.clone()
        }
        _ => {
            let conn = db.conn.lock()?;
            let id = conn.query_row(
                "SELECT id FROM agents WHERE is_archived = 0 ORDER BY created_at LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            ).map_err(|_| {
                eprintln!("[workflow] ERROR: No agents in database");
                AppError::NotFound("No agents available. Create an agent first before running workflows.".into())
            })?;
            eprintln!("[workflow] Using fallback agent_id: {}", id);
            id
        }
    };
    let session_id = Uuid::new_v4().to_string();
    let now = now_iso();
    {
        let conn = db.conn.lock()?;
        conn.execute(
            "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
            params![session_id, agent_id, format!("Workflow: {}", workflow_name), now, now],
        )
        .map_err(|e| {
            eprintln!("[workflow] ERROR: Session creation failed: {e} (agent_id={})", agent_id);
            AppError::Db(format!("Failed to create workflow session: {e}"))
        })?;
    }
    eprintln!("[workflow] Created session {}", session_id);

    // 4. Load provider config
    let all_settings = {
        let conn = db.conn.lock()?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let mut settings = std::collections::HashMap::<String, String>::new();
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })?;
        for row in rows {
            let (k, v) = row?;
            settings.insert(k, v);
        }
        settings
    };
    eprintln!("[workflow] Loaded {} settings", all_settings.len());

    // 4b. Budget enforcement — block workflow if budget exhausted
    let budget_pct = crate::commands::budget::get_budget_remaining_pct(db.inner(), &all_settings);
    if budget_pct <= 0.0 {
        let exhausted_behavior = all_settings
            .get("budget.exhausted_behavior")
            .map(|v| v.trim_matches('"').to_string())
            .unwrap_or_else(|| "none".to_string());
        if exhausted_behavior == "ask" {
            return Err(AppError::BudgetExhausted(
                "Monthly budget exhausted. Cannot run workflow.".into(),
            ));
        }
        // local_only and cheapest_cloud: individual LLM nodes will be routed by the engine
        // none: proceed without enforcement
    }

    // 5. Execute workflow
    let db_clone = db.inner().clone();
    let sidecar_clone = sidecar.inner().clone();
    let session_id_clone = session_id.clone();
    let inputs = request.inputs.clone();

    let result_handle = tauri::async_runtime::spawn(async move {
        execute_workflow(
            &db_clone, &sidecar_clone, &app,
            &session_id_clone, &graph_json, &inputs, &all_settings,
        ).await
    });

    match result_handle.await {
        Ok(result) => {
            match &result {
                Ok(r) => eprintln!("[workflow] === RUN DONE === status={}, tokens={}, cost=${:.4}, duration={}ms",
                    r.status, r.total_tokens, r.total_cost_usd, r.duration_ms),
                Err(e) => eprintln!("[workflow] === RUN FAILED === {}", e),
            }
            result.map_err(|e| AppError::Workflow(e))
        }
        Err(e) => {
            eprintln!(
                "[workflow.run] panic workflow_id={} session_id={} error={}",
                request.workflow_id, session_id, e
            );
            Err(AppError::Workflow(format!("Workflow execution panicked: {e}")))
        }
    }
}

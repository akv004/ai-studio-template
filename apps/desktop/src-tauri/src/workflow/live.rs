use crate::db::{Database, now_iso};
use crate::error::AppError;
use super::engine::{execute_workflow_ephemeral, extract_primary_text};
use super::validation::validate_graph_json;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use uuid::Uuid;

/// Manages live (continuous loop) workflow executions.
/// Each workflow_id can have at most one active live run.
#[derive(Clone)]
pub struct LiveWorkflowManager {
    active: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

impl Default for LiveWorkflowManager {
    fn default() -> Self {
        Self {
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl LiveWorkflowManager {
    /// Register a new live run. Returns the cancel token.
    /// Errors if a live run is already active for this workflow.
    pub fn start(&self, workflow_id: &str) -> Result<Arc<AtomicBool>, String> {
        let mut map = self.active.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        if map.contains_key(workflow_id) {
            return Err(format!("Live run already active for workflow {}", workflow_id));
        }
        let token = Arc::new(AtomicBool::new(false));
        map.insert(workflow_id.to_string(), token.clone());
        Ok(token)
    }

    /// Signal a live run to stop.
    pub fn stop(&self, workflow_id: &str) -> Result<(), String> {
        let map = self.active.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        if let Some(token) = map.get(workflow_id) {
            token.store(true, Ordering::Relaxed);
            Ok(())
        } else {
            Err(format!("No active live run for workflow {}", workflow_id))
        }
    }

    /// Remove a workflow from the active map (called when loop exits).
    pub fn remove(&self, workflow_id: &str) {
        if let Ok(mut map) = self.active.lock() {
            map.remove(workflow_id);
        }
    }

    /// Stop all active live runs (for app shutdown).
    pub fn stop_all(&self) {
        if let Ok(map) = self.active.lock() {
            for token in map.values() {
                token.store(true, Ordering::Relaxed);
            }
        }
    }

    /// Check if a workflow has an active live run.
    pub fn is_active(&self, workflow_id: &str) -> bool {
        self.active
            .lock()
            .map(|map| map.contains_key(workflow_id))
            .unwrap_or(false)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartLiveRequest {
    pub workflow_id: String,
    pub inputs: HashMap<String, serde_json::Value>,
    #[serde(default = "default_interval")]
    pub interval_ms: u64,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u64,
    #[serde(default = "default_error_policy")]
    pub error_policy: String,
}

fn default_interval() -> u64 { 5000 }
fn default_max_iterations() -> u64 { 1000 }
fn default_error_policy() -> String { "skip".to_string() }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartLiveResponse {
    pub live_run_id: String,
    pub session_id: String,
}

#[tauri::command]
pub async fn start_live_workflow(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    live_mgr: tauri::State<'_, LiveWorkflowManager>,
    app: tauri::AppHandle,
    request: StartLiveRequest,
) -> Result<StartLiveResponse, AppError> {
    let workflow_id = request.workflow_id.clone();
    eprintln!("[live] Starting live workflow: {}", workflow_id);

    // 1. Load workflow
    let (workflow_name, graph_json, workflow_agent_id) = {
        let conn = db.conn.lock()?;
        conn.query_row(
            "SELECT name, graph_json, agent_id FROM workflows WHERE id = ?1 AND is_archived = 0",
            params![workflow_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?)),
        )
        .map_err(|e| AppError::NotFound(format!("Workflow not found: {e}")))?
    };

    // 2. Validate
    let validation = validate_graph_json(&graph_json).map_err(|e| AppError::Validation(e))?;
    if !validation.valid {
        return Err(AppError::Validation(format!("Invalid workflow: {}", validation.errors.join("; "))));
    }

    // 3. Acquire cancel token (errors if already running)
    let cancel_token = live_mgr.start(&workflow_id)
        .map_err(|e| AppError::Workflow(e))?;

    // 4. Create a single session for this live run
    let agent_id = match workflow_agent_id {
        Some(ref id) if !id.is_empty() => id.clone(),
        _ => {
            let conn = db.conn.lock()?;
            conn.query_row(
                "SELECT id FROM agents WHERE is_archived = 0 ORDER BY created_at LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            ).map_err(|_| AppError::NotFound("No agents available".into()))?
        }
    };
    let session_id = Uuid::new_v4().to_string();
    let live_run_id = Uuid::new_v4().to_string();
    let now = now_iso();
    {
        let conn = db.conn.lock()?;
        conn.execute(
            "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
            params![session_id, agent_id, format!("Live: {}", workflow_name), now, now],
        ).map_err(|e| AppError::Db(format!("Failed to create session: {e}")))?;
    }

    // 5. Load settings
    let all_settings = {
        let conn = db.conn.lock()?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let mut settings = HashMap::<String, String>::new();
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

    // 6. Spawn the live loop
    let db_clone = db.inner().clone();
    let sidecar_clone = sidecar.inner().clone();
    let live_mgr_clone = app.state::<LiveWorkflowManager>().inner().clone();
    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let live_run_id_clone = live_run_id.clone();
    let workflow_id_clone = workflow_id.clone();
    let inputs = request.inputs.clone();
    let interval_ms = request.interval_ms;
    let max_iterations = request.max_iterations;
    let error_policy = request.error_policy.clone();
    let graph_json_clone = graph_json.clone();

    tauri::async_runtime::spawn(async move {
        live_loop(
            &db_clone, &sidecar_clone, &app_clone, &live_mgr_clone,
            &cancel_token, &session_id_clone, &live_run_id_clone,
            &workflow_id_clone, &graph_json_clone, &inputs, &all_settings,
            interval_ms, max_iterations, &error_policy,
        ).await;
    });

    eprintln!("[live] Spawned live loop: live_run_id={}, session_id={}", live_run_id, session_id);

    Ok(StartLiveResponse {
        live_run_id,
        session_id,
    })
}

#[tauri::command]
pub async fn stop_live_workflow(
    live_mgr: tauri::State<'_, LiveWorkflowManager>,
    workflow_id: String,
) -> Result<(), AppError> {
    eprintln!("[live] Stopping live workflow: {}", workflow_id);
    live_mgr.stop(&workflow_id)
        .map_err(|e| AppError::Workflow(e))
}

/// The main live execution loop. Runs on a spawned async task.
async fn live_loop(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    app: &tauri::AppHandle,
    live_mgr: &LiveWorkflowManager,
    cancel: &AtomicBool,
    session_id: &str,
    live_run_id: &str,
    workflow_id: &str,
    graph_json: &str,
    inputs: &HashMap<String, serde_json::Value>,
    all_settings: &HashMap<String, String>,
    interval_ms: u64,
    max_iterations: u64,
    error_policy: &str,
) {
    // Emit live.started
    let _ = app.emit("live_workflow_feed", serde_json::json!({
        "type": "live.started",
        "liveRunId": live_run_id,
        "workflowId": workflow_id,
        "intervalMs": interval_ms,
    }));

    let mut iteration: u64 = 0;
    let mut consecutive_errors: u32 = 0;
    let mut total_tokens: i64 = 0;
    let mut total_cost: f64 = 0.0;
    let stop_reason;

    loop {
        // Check cancel
        if cancel.load(Ordering::Relaxed) {
            stop_reason = "user_stopped";
            break;
        }

        // Check max iterations
        if iteration >= max_iterations {
            stop_reason = "max_iterations";
            break;
        }

        iteration += 1;
        let iter_start = std::time::Instant::now();

        // Execute one iteration (ephemeral = true, skip DB writes)
        let result = execute_workflow_ephemeral(
            db, sidecar, app, session_id, graph_json, inputs, all_settings, true,
        ).await;

        match result {
            Ok(run_result) => {
                consecutive_errors = 0;
                let duration_ms = iter_start.elapsed().as_millis() as i64;
                let tokens = run_result.total_tokens;
                let cost = run_result.total_cost_usd;
                total_tokens += tokens;
                total_cost += cost;

                // Extract output summary from the first output node
                let output_summary = run_result.outputs.values().next()
                    .map(|v| {
                        let text = extract_primary_text(v);
                        if text.len() > 300 {
                            format!("{}...", &text[..text.char_indices().nth(300).map(|(i,_)|i).unwrap_or(text.len())])
                        } else {
                            text
                        }
                    })
                    .unwrap_or_else(|| run_result.status.clone());

                let _ = app.emit("live_workflow_feed", serde_json::json!({
                    "type": "live.iteration.completed",
                    "liveRunId": live_run_id,
                    "iteration": iteration,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "outputSummary": output_summary,
                    "tokens": tokens,
                    "costUsd": cost,
                    "durationMs": duration_ms,
                    "status": run_result.status,
                }));
            }
            Err(err) => {
                consecutive_errors += 1;
                let _ = app.emit("live_workflow_feed", serde_json::json!({
                    "type": "live.iteration.error",
                    "liveRunId": live_run_id,
                    "iteration": iteration,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "error": err,
                }));

                if error_policy == "stop" {
                    stop_reason = "error_policy_stop";
                    break;
                }

                if consecutive_errors >= 5 {
                    eprintln!("[live] 5 consecutive errors, auto-stopping");
                    stop_reason = "consecutive_errors";
                    break;
                }
            }
        }

        // Check cancel before sleeping
        if cancel.load(Ordering::Relaxed) {
            stop_reason = "user_stopped";
            break;
        }

        // Sleep with cancel checking every 100ms
        let sleep_chunks = interval_ms / 100;
        for _ in 0..sleep_chunks {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        // Sleep remaining sub-100ms portion
        let remainder = interval_ms % 100;
        if remainder > 0 && !cancel.load(Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_millis(remainder)).await;
        }

        if cancel.load(Ordering::Relaxed) {
            stop_reason = "user_stopped";
            break;
        }

        continue;
    }

    // Emit live.stopped
    let _ = app.emit("live_workflow_feed", serde_json::json!({
        "type": "live.stopped",
        "liveRunId": live_run_id,
        "totalIterations": iteration,
        "totalTokens": total_tokens,
        "totalCostUsd": total_cost,
        "reason": stop_reason,
    }));

    // Cleanup
    live_mgr.remove(workflow_id);
    eprintln!("[live] Live loop ended: workflow_id={}, iterations={}, reason={}",
        workflow_id, iteration, stop_reason);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_manager_start_stop() {
        let mgr = LiveWorkflowManager::default();

        // Start should succeed
        let token = mgr.start("wf-1").unwrap();
        assert!(mgr.is_active("wf-1"));
        assert!(!token.load(Ordering::Relaxed));

        // Double start should fail
        assert!(mgr.start("wf-1").is_err());

        // Stop should set flag
        mgr.stop("wf-1").unwrap();
        assert!(token.load(Ordering::Relaxed));

        // Remove should clean up
        mgr.remove("wf-1");
        assert!(!mgr.is_active("wf-1"));

        // Start again should work after remove
        let _token2 = mgr.start("wf-1").unwrap();
        assert!(mgr.is_active("wf-1"));
    }

    #[test]
    fn test_live_manager_stop_nonexistent() {
        let mgr = LiveWorkflowManager::default();
        assert!(mgr.stop("no-such-workflow").is_err());
    }

    #[test]
    fn test_live_manager_stop_all() {
        let mgr = LiveWorkflowManager::default();
        let t1 = mgr.start("wf-1").unwrap();
        let t2 = mgr.start("wf-2").unwrap();
        let t3 = mgr.start("wf-3").unwrap();

        assert!(!t1.load(Ordering::Relaxed));
        assert!(!t2.load(Ordering::Relaxed));
        assert!(!t3.load(Ordering::Relaxed));

        mgr.stop_all();

        assert!(t1.load(Ordering::Relaxed));
        assert!(t2.load(Ordering::Relaxed));
        assert!(t3.load(Ordering::Relaxed));
    }

    #[test]
    fn test_live_manager_concurrent_workflows() {
        let mgr = LiveWorkflowManager::default();

        // Multiple workflows can run concurrently
        let _t1 = mgr.start("wf-a").unwrap();
        let _t2 = mgr.start("wf-b").unwrap();

        assert!(mgr.is_active("wf-a"));
        assert!(mgr.is_active("wf-b"));
        assert!(!mgr.is_active("wf-c"));

        mgr.stop("wf-a").unwrap();
        mgr.remove("wf-a");
        assert!(!mgr.is_active("wf-a"));
        assert!(mgr.is_active("wf-b"));
    }
}

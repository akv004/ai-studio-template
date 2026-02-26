pub mod auth;
pub mod rate_limit;
pub mod server;

use crate::db::{Database, now_iso};
use crate::sidecar::SidecarManager;
use crate::workflow::engine::execute_workflow_ephemeral;
use crate::workflow::validation::validate_graph_json;
use rate_limit::RateLimiter;
use server::{WebhookRoute, WebhookState};
use rusqlite::params;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use uuid::Uuid;

/// A single armed cron schedule entry.
#[derive(Clone)]
pub struct CronScheduleEntry {
    pub trigger_id: String,
    pub workflow_id: String,
    pub expression: String,
    pub timezone: String,
    pub static_input: serde_json::Value,
    pub max_concurrent: u32,
    pub active_runs: Arc<AtomicU32>,
    pub fire_count: Arc<AtomicI64>,
    /// Track last fired minute to prevent double-fires within the same minute
    pub last_fired_minute: Arc<Mutex<Option<i64>>>,
}

/// Manages webhook + cron trigger lifecycle.
/// Follows the same pattern as LiveWorkflowManager.
#[derive(Clone)]
pub struct TriggerManager {
    routes: Arc<Mutex<HashMap<String, WebhookRoute>>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    port: Arc<Mutex<u16>>,
    rate_limiter: RateLimiter,
    // Cron scheduler
    cron_schedules: Arc<Mutex<HashMap<String, CronScheduleEntry>>>,
    cron_shutdown: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl Default for TriggerManager {
    fn default() -> Self {
        Self {
            routes: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(9876)),
            rate_limiter: RateLimiter::new(60),
            cron_schedules: Arc::new(Mutex::new(HashMap::new())),
            cron_shutdown: Arc::new(Mutex::new(None)),
        }
    }
}

impl TriggerManager {
    /// Set the port (from settings). Must be called before first arm.
    pub fn set_port(&self, port: u16) {
        if let Ok(mut p) = self.port.lock() {
            *p = port;
        }
    }

    /// Register a webhook route and start the server if it's the first trigger.
    /// Checks server state atomically to prevent concurrent arm calls from
    /// both trying to start the server.
    pub async fn arm_webhook(
        &self,
        path: &str,
        route: WebhookRoute,
        db: &Database,
        sidecar: &SidecarManager,
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        let needs_server = {
            let mut routes = self.routes.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
            routes.insert(path.to_string(), route);
            // Check if server is already running while we hold the routes lock
            let has_server = self.shutdown_tx.lock()
                .map(|s| s.is_some())
                .unwrap_or(false);
            !has_server
        };

        if needs_server {
            let port = *self.port.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
            let state = WebhookState {
                routes: self.routes.clone(),
                rate_limiter: self.rate_limiter.clone(),
                db: db.clone(),
                sidecar: sidecar.clone(),
                app_handle: app.clone(),
            };
            let tx = server::start_server(state, port).await?;
            // Re-check under lock: another arm call may have started server first
            let mut shutdown = self.shutdown_tx.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
            if shutdown.is_none() {
                *shutdown = Some(tx);
                eprintln!("[webhook] Server started on port {}", port);
            }
            // If shutdown.is_some(), another call won the race — our server will fail
            // to bind or we just drop our tx (harmless)
        }

        Ok(())
    }

    /// Remove a webhook route. Stops the server if no routes remain.
    pub fn disarm_webhook(&self, path: &str) -> Result<(), String> {
        let should_stop = {
            let mut routes = self.routes.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
            routes.remove(path);
            self.rate_limiter.remove(path);
            routes.is_empty()
        };

        if should_stop {
            self.stop_server()?;
        }

        Ok(())
    }

    /// Stop the Axum server if running.
    fn stop_server(&self) -> Result<(), String> {
        let mut shutdown = self.shutdown_tx.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        if let Some(tx) = shutdown.take() {
            let _ = tx.send(());
            eprintln!("[webhook] Server stopped");
        }
        Ok(())
    }

    /// Stop all webhooks and the server (for app shutdown).
    pub fn stop_all(&self) {
        if let Ok(mut routes) = self.routes.lock() {
            routes.clear();
        }
        let _ = self.stop_server();
        self.stop_cron_scheduler();
    }

    /// Check if a specific path is armed.
    pub fn is_armed(&self, path: &str) -> bool {
        self.routes
            .lock()
            .map(|r| r.contains_key(path))
            .unwrap_or(false)
    }

    /// Check if a cron trigger is armed by trigger_id.
    pub fn is_cron_armed(&self, trigger_id: &str) -> bool {
        self.cron_schedules
            .lock()
            .map(|s| s.contains_key(trigger_id))
            .unwrap_or(false)
    }

    /// Get server status.
    pub fn status(&self) -> WebhookServerStatus {
        let routes = self.routes.lock().map(|r| r.len()).unwrap_or(0);
        let running = self.shutdown_tx.lock().map(|s| s.is_some()).unwrap_or(false);
        let port = self.port.lock().map(|p| *p).unwrap_or(9876);
        WebhookServerStatus {
            running,
            port,
            active_hooks: routes,
        }
    }

    /// Get cron scheduler status.
    pub fn cron_status(&self) -> CronSchedulerStatus {
        let schedules = self.cron_schedules.lock().map(|s| s.len()).unwrap_or(0);
        let running = self.cron_shutdown.lock().map(|s| s.is_some()).unwrap_or(false);
        CronSchedulerStatus {
            running,
            active_schedules: schedules,
        }
    }

    // ---------- Cron scheduler methods ----------

    /// Arm a cron schedule. Starts the tick loop if this is the first schedule.
    pub async fn arm_cron(
        &self,
        entry: CronScheduleEntry,
        db: &Database,
        sidecar: &SidecarManager,
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        // Validate the cron expression parses
        use std::str::FromStr;
        cron::Schedule::from_str(&entry.expression)
            .map_err(|e| format!("Invalid cron expression '{}': {e}", entry.expression))?;

        let needs_loop = {
            let mut schedules = self.cron_schedules.lock()
                .map_err(|e| format!("Lock poisoned: {e}"))?;
            let trigger_id = entry.trigger_id.clone();
            schedules.insert(trigger_id, entry);
            let has_loop = self.cron_shutdown.lock()
                .map(|s| s.is_some())
                .unwrap_or(false);
            !has_loop
        };

        if needs_loop {
            self.start_cron_scheduler(db, sidecar, app)?;
        }

        Ok(())
    }

    /// Disarm a cron schedule. Stops the tick loop if no schedules remain.
    pub fn disarm_cron(&self, trigger_id: &str) -> Result<(), String> {
        let should_stop = {
            let mut schedules = self.cron_schedules.lock()
                .map_err(|e| format!("Lock poisoned: {e}"))?;
            schedules.remove(trigger_id);
            schedules.is_empty()
        };

        if should_stop {
            self.stop_cron_scheduler();
        }

        Ok(())
    }

    /// Start the cron tick loop (1-second interval).
    fn start_cron_scheduler(
        &self,
        db: &Database,
        sidecar: &SidecarManager,
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        let schedules = self.cron_schedules.clone();
        let db = db.clone();
        let sidecar = sidecar.clone();
        let app = app.clone();

        tauri::async_runtime::spawn(async move {
            eprintln!("[cron] Scheduler started");
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        eprintln!("[cron] Scheduler shutting down");
                        break;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                        let now = chrono::Utc::now();
                        // Truncate to current minute for matching
                        let current_minute = now.timestamp() / 60;

                        let entries: Vec<CronScheduleEntry> = {
                            match schedules.lock() {
                                Ok(s) => s.values().cloned().collect(),
                                Err(_) => continue,
                            }
                        };

                        for entry in &entries {
                            use std::str::FromStr;
                            let schedule = match cron::Schedule::from_str(&entry.expression) {
                                Ok(s) => s,
                                Err(_) => continue,
                            };

                            // Check if we already fired for this minute
                            let already_fired = entry.last_fired_minute.lock()
                                .map(|m| m.map(|lm| lm == current_minute).unwrap_or(false))
                                .unwrap_or(false);
                            if already_fired {
                                continue;
                            }

                            // Check if current time matches the schedule
                            // Use timezone-aware matching
                            let tz: chrono_tz::Tz = entry.timezone.parse().unwrap_or(chrono_tz::Tz::UTC);
                            let local_now = now.with_timezone(&tz);

                            // Get the upcoming event — if the next event is within this same minute, we should fire
                            let upcoming = schedule.after(&(local_now - chrono::Duration::seconds(60)));
                            let should_fire = upcoming.take(1).any(|next| {
                                next.timestamp() / 60 == current_minute
                            });

                            if !should_fire {
                                continue;
                            }

                            // Check max concurrent
                            let active = entry.active_runs.load(Ordering::Relaxed);
                            if active >= entry.max_concurrent {
                                eprintln!("[cron] Skipping '{}': max concurrent ({}) reached",
                                    entry.trigger_id, entry.max_concurrent);
                                continue;
                            }

                            // Mark as fired for this minute
                            if let Ok(mut m) = entry.last_fired_minute.lock() {
                                *m = Some(current_minute);
                            }

                            // Increment fire count
                            let iteration = entry.fire_count.fetch_add(1, Ordering::Relaxed) + 1;

                            // Build cron inputs
                            let mut inputs = HashMap::new();
                            inputs.insert("__cron_timestamp".to_string(), serde_json::json!(now.to_rfc3339()));
                            inputs.insert("__cron_iteration".to_string(), serde_json::json!(iteration));
                            inputs.insert("__cron_input".to_string(), entry.static_input.clone());
                            inputs.insert("__cron_schedule".to_string(), serde_json::json!(entry.expression));
                            // Also inject static_input as "input" for standard Input nodes
                            inputs.insert("input".to_string(), entry.static_input.clone());

                            let active_runs = entry.active_runs.clone();
                            let trigger_id = entry.trigger_id.clone();
                            let workflow_id = entry.workflow_id.clone();
                            let db_clone = db.clone();
                            let sidecar_clone = sidecar.clone();
                            let app_clone = app.clone();

                            active_runs.fetch_add(1, Ordering::Relaxed);

                            tauri::async_runtime::spawn(async move {
                                Self::execute_cron_run(
                                    &db_clone, &sidecar_clone, &app_clone,
                                    &trigger_id, &workflow_id, &inputs,
                                ).await;
                                active_runs.fetch_sub(1, Ordering::Relaxed);
                            });
                        }
                    }
                }
            }
        });

        let mut shutdown = self.cron_shutdown.lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        if shutdown.is_none() {
            *shutdown = Some(shutdown_tx);
        }

        Ok(())
    }

    /// Stop the cron scheduler.
    fn stop_cron_scheduler(&self) {
        if let Ok(mut shutdown) = self.cron_shutdown.lock() {
            if let Some(tx) = shutdown.take() {
                let _ = tx.send(());
                eprintln!("[cron] Scheduler stopped");
            }
        }
    }

    /// Execute a single cron-triggered workflow run.
    async fn execute_cron_run(
        db: &Database,
        sidecar: &SidecarManager,
        app: &tauri::AppHandle,
        trigger_id: &str,
        workflow_id: &str,
        inputs: &HashMap<String, serde_json::Value>,
    ) {
        // Load workflow
        let (graph_json, all_settings, workflow_name, agent_id) = {
            let conn = match db.conn.lock() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[cron] DB lock error: {e}");
                    return;
                }
            };

            let wf = conn.query_row(
                "SELECT name, graph_json, agent_id FROM workflows WHERE id = ?1 AND is_archived = 0",
                params![workflow_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?)),
            );
            let (name, graph, wf_agent_id) = match wf {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[cron] Workflow not found: {e}");
                    return;
                }
            };

            let agent = wf_agent_id.filter(|id| !id.is_empty()).unwrap_or_else(|| {
                conn.query_row(
                    "SELECT id FROM agents WHERE is_archived = 0 ORDER BY created_at LIMIT 1",
                    [], |row| row.get::<_, String>(0),
                ).unwrap_or_default()
            });

            let mut settings = HashMap::<String, String>::new();
            if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM settings") {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                }) {
                    for row in rows.flatten() {
                        settings.insert(row.0, row.1);
                    }
                }
            }

            (graph, settings, name, agent)
        };

        // Validate
        match validate_graph_json(&graph_json) {
            Ok(v) if !v.valid => {
                eprintln!("[cron] Invalid workflow: {}", v.errors.join("; "));
                return;
            }
            Err(e) => {
                eprintln!("[cron] Validation error: {e}");
                return;
            }
            _ => {}
        }

        // Create session
        let session_id = Uuid::new_v4().to_string();
        let now = now_iso();
        {
            let conn = match db.conn.lock() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[cron] DB lock error: {e}");
                    return;
                }
            };
            if let Err(e) = conn.execute(
                "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
                params![session_id, agent_id, format!("Cron: {}", workflow_name), now, now],
            ) {
                eprintln!("[cron] Failed to create session: {e}");
                return;
            }
        }

        // Log trigger fire
        let log_id = Uuid::new_v4().to_string();
        {
            if let Ok(conn) = db.conn.lock() {
                let _ = conn.execute(
                    "INSERT INTO trigger_log (id, trigger_id, run_id, fired_at, status) VALUES (?1, ?2, ?3, ?4, 'fired')",
                    params![log_id, trigger_id, session_id, now],
                );
                let _ = conn.execute(
                    "UPDATE triggers SET last_fired = ?1, fire_count = fire_count + 1, updated_at = ?1 WHERE id = ?2",
                    params![now, trigger_id],
                );
            }
        }

        // Execute workflow
        eprintln!("[cron] Firing workflow '{}' for trigger '{}'", workflow_id, trigger_id);
        let result = execute_workflow_ephemeral(
            db, sidecar, app, &session_id, &graph_json, inputs, &all_settings, false,
        ).await;

        // Update log
        if let Ok(conn) = db.conn.lock() {
            let status = match &result {
                Ok(_) => "completed",
                Err(_) => "error",
            };
            let _ = conn.execute(
                "UPDATE trigger_log SET status = ?1 WHERE id = ?2",
                params![status, log_id],
            );
        }

        match result {
            Ok(r) => eprintln!("[cron] Workflow completed: trigger={}, status={}", trigger_id, r.status),
            Err(e) => eprintln!("[cron] Workflow error: trigger={}, error={}", trigger_id, e),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookServerStatus {
    pub running: bool,
    pub port: u16,
    pub active_hooks: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CronSchedulerStatus {
    pub running: bool,
    pub active_schedules: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let mgr = TriggerManager::default();
        let status = mgr.status();
        assert!(!status.running);
        assert_eq!(status.port, 9876);
        assert_eq!(status.active_hooks, 0);
        let cron_status = mgr.cron_status();
        assert!(!cron_status.running);
        assert_eq!(cron_status.active_schedules, 0);
    }

    #[test]
    fn test_is_armed_empty() {
        let mgr = TriggerManager::default();
        assert!(!mgr.is_armed("test-path"));
        assert!(!mgr.is_cron_armed("some-trigger"));
    }

    #[test]
    fn test_manual_route_management() {
        let mgr = TriggerManager::default();

        // Manually insert a route (bypassing arm_webhook which needs Tauri)
        {
            let mut routes = mgr.routes.lock().unwrap();
            routes.insert("test-path".to_string(), WebhookRoute {
                trigger_id: "t1".into(),
                workflow_id: "wf1".into(),
                auth_mode: auth::AuthMode::None,
                response_mode: server::ResponseMode::Immediate,
                timeout_secs: 30,
                methods: vec!["POST".into()],
                max_per_minute: None,
            });
        }
        assert!(mgr.is_armed("test-path"));
        assert!(!mgr.is_armed("other-path"));
        assert_eq!(mgr.status().active_hooks, 1);

        // Disarm
        mgr.disarm_webhook("test-path").unwrap();
        assert!(!mgr.is_armed("test-path"));
        assert_eq!(mgr.status().active_hooks, 0);
    }

    #[test]
    fn test_stop_all_clears_routes() {
        let mgr = TriggerManager::default();
        {
            let mut routes = mgr.routes.lock().unwrap();
            routes.insert("a".into(), WebhookRoute {
                trigger_id: "t1".into(),
                workflow_id: "wf1".into(),
                auth_mode: auth::AuthMode::None,
                response_mode: server::ResponseMode::Immediate,
                timeout_secs: 30,
                methods: vec![],
                max_per_minute: None,
            });
            routes.insert("b".into(), WebhookRoute {
                trigger_id: "t2".into(),
                workflow_id: "wf2".into(),
                auth_mode: auth::AuthMode::None,
                response_mode: server::ResponseMode::Immediate,
                timeout_secs: 30,
                methods: vec![],
                max_per_minute: None,
            });
        }
        assert_eq!(mgr.status().active_hooks, 2);
        mgr.stop_all();
        assert_eq!(mgr.status().active_hooks, 0);
    }

    // --- Cron scheduler unit tests ---

    #[test]
    fn test_cron_schedule_parse_valid() {
        use std::str::FromStr;
        let schedule = cron::Schedule::from_str("*/5 * * * * *");
        assert!(schedule.is_ok(), "*/5 * * * * * should parse");

        let schedule = cron::Schedule::from_str("0 9 * * * *");
        assert!(schedule.is_ok(), "0 9 * * * * should parse");

        let schedule = cron::Schedule::from_str("0 0 1 * * *");
        assert!(schedule.is_ok(), "0 0 1 * * * should parse");
    }

    #[test]
    fn test_cron_schedule_parse_invalid() {
        use std::str::FromStr;
        // Too few fields
        let schedule = cron::Schedule::from_str("* * *");
        assert!(schedule.is_err(), "3 fields should be invalid");
    }

    #[test]
    fn test_cron_next_occurrence() {
        use std::str::FromStr;
        use chrono::{TimeZone, Timelike};
        let schedule = cron::Schedule::from_str("0 30 9 * * * *").unwrap();
        let base = chrono::Utc.with_ymd_and_hms(2026, 2, 26, 8, 0, 0).unwrap();
        let next = schedule.after(&base).next().unwrap();
        assert_eq!(next.hour(), 9);
        assert_eq!(next.minute(), 30);
    }

    #[test]
    fn test_cron_timezone_conversion() {
        use std::str::FromStr;
        let tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
        let utc_now = chrono::Utc::now();
        let local = utc_now.with_timezone(&tz);
        // The timezone should differ from UTC by several hours
        assert!(tz != chrono_tz::Tz::UTC);
        // Basic sanity: the converted time should have the same timestamp
        assert_eq!(local.timestamp(), utc_now.timestamp());
    }

    #[test]
    fn test_cron_max_concurrent_skip() {
        let entry = CronScheduleEntry {
            trigger_id: "t1".into(),
            workflow_id: "wf1".into(),
            expression: "0 * * * * *".into(),
            timezone: "UTC".into(),
            static_input: serde_json::json!({}),
            max_concurrent: 2,
            active_runs: Arc::new(AtomicU32::new(2)), // already at max
            fire_count: Arc::new(AtomicI64::new(0)),
            last_fired_minute: Arc::new(Mutex::new(None)),
        };

        let active = entry.active_runs.load(Ordering::Relaxed);
        assert!(active >= entry.max_concurrent, "should skip when at max concurrent");
    }

    #[test]
    fn test_cron_fire_count_increment() {
        let fire_count = Arc::new(AtomicI64::new(0));
        assert_eq!(fire_count.fetch_add(1, Ordering::Relaxed), 0);
        assert_eq!(fire_count.fetch_add(1, Ordering::Relaxed), 1);
        assert_eq!(fire_count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_cron_manual_schedule_management() {
        let mgr = TriggerManager::default();

        // Manually insert a schedule (bypassing arm_cron which needs Tauri)
        {
            let mut schedules = mgr.cron_schedules.lock().unwrap();
            schedules.insert("t1".into(), CronScheduleEntry {
                trigger_id: "t1".into(),
                workflow_id: "wf1".into(),
                expression: "0 * * * * *".into(),
                timezone: "UTC".into(),
                static_input: serde_json::json!({}),
                max_concurrent: 1,
                active_runs: Arc::new(AtomicU32::new(0)),
                fire_count: Arc::new(AtomicI64::new(0)),
                last_fired_minute: Arc::new(Mutex::new(None)),
            });
        }
        assert!(mgr.is_cron_armed("t1"));
        assert!(!mgr.is_cron_armed("t2"));
        assert_eq!(mgr.cron_status().active_schedules, 1);

        // Disarm
        mgr.disarm_cron("t1").unwrap();
        assert!(!mgr.is_cron_armed("t1"));
        assert_eq!(mgr.cron_status().active_schedules, 0);
    }
}

pub mod auth;
pub mod rate_limit;
pub mod server;

use crate::db::Database;
use crate::sidecar::SidecarManager;
use rate_limit::RateLimiter;
use server::{WebhookRoute, WebhookState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// Manages webhook trigger lifecycle — arming, disarming, and the Axum server.
/// Follows the same pattern as LiveWorkflowManager.
#[derive(Clone)]
pub struct TriggerManager {
    routes: Arc<Mutex<HashMap<String, WebhookRoute>>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    port: Arc<Mutex<u16>>,
    rate_limiter: RateLimiter,
}

impl Default for TriggerManager {
    fn default() -> Self {
        Self {
            routes: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(9876)),
            rate_limiter: RateLimiter::new(60),
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
    }

    /// Check if a specific path is armed.
    pub fn is_armed(&self, path: &str) -> bool {
        self.routes
            .lock()
            .map(|r| r.contains_key(path))
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
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookServerStatus {
    pub running: bool,
    pub port: u16,
    pub active_hooks: usize,
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
    }

    #[test]
    fn test_is_armed_empty() {
        let mgr = TriggerManager::default();
        assert!(!mgr.is_armed("test-path"));
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
}

use super::auth::{AuthMode, validate_auth};
use super::rate_limit::RateLimiter;
use crate::db::{Database, now_iso};
use crate::sidecar::SidecarManager;
use crate::workflow::engine::execute_workflow_ephemeral;
use crate::workflow::validation::validate_graph_json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::any;
use axum::Router;
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use uuid::Uuid;

/// Routing entry for a single webhook endpoint.
#[derive(Clone, Debug)]
pub struct WebhookRoute {
    pub trigger_id: String,
    pub workflow_id: String,
    pub auth_mode: AuthMode,
    pub response_mode: ResponseMode,
    pub timeout_secs: u64,
    pub methods: Vec<String>,
    pub max_per_minute: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResponseMode {
    Immediate,
    Wait,
}

impl ResponseMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "wait" => ResponseMode::Wait,
            _ => ResponseMode::Immediate,
        }
    }
}

/// Shared state for the Axum webhook server.
#[derive(Clone)]
pub struct WebhookState {
    pub routes: Arc<Mutex<HashMap<String, WebhookRoute>>>,
    pub rate_limiter: RateLimiter,
    pub db: Database,
    pub sidecar: SidecarManager,
    pub app_handle: tauri::AppHandle,
}

#[derive(Serialize)]
struct WebhookResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    run_id: Option<String>,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Build the Axum router with a catch-all handler.
pub fn build_router(state: WebhookState) -> Router {
    Router::new()
        .route("/hook/{*path}", any(webhook_handler))
        .route("/health", axum::routing::get(health_handler))
        .with_state(state)
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

async fn webhook_handler(
    State(state): State<WebhookState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    eprintln!("[webhook] {} /hook/{}", method, path);

    // 1. Lookup route
    let route = {
        let routes = state.routes.lock().unwrap_or_else(|e| e.into_inner());
        routes.get(&path).cloned()
    };
    let route = match route {
        Some(r) => r,
        None => {
            return (StatusCode::NOT_FOUND, Json(WebhookResponse {
                run_id: None,
                status: "error".into(),
                output: None,
                error: Some(format!("No webhook registered for path: {}", path)),
            }));
        }
    };

    // 2. Validate method
    if !route.methods.is_empty() {
        let method_str = method.as_str().to_uppercase();
        if !route.methods.iter().any(|m| m.to_uppercase() == method_str) {
            return (StatusCode::METHOD_NOT_ALLOWED, Json(WebhookResponse {
                run_id: None,
                status: "error".into(),
                output: None,
                error: Some(format!("Method {} not allowed", method)),
            }));
        }
    }

    // 3. Rate limit
    if !state.rate_limiter.check(&path, route.max_per_minute) {
        return (StatusCode::TOO_MANY_REQUESTS, Json(WebhookResponse {
            run_id: None,
            status: "error".into(),
            output: None,
            error: Some("Rate limit exceeded".into()),
        }));
    }

    // 4. Auth
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    let sig_header = headers.get("x-signature").and_then(|v| v.to_str().ok());
    if let Err(e) = validate_auth(&route.auth_mode, auth_header, sig_header, &body) {
        return (StatusCode::UNAUTHORIZED, Json(WebhookResponse {
            run_id: None,
            status: "error".into(),
            output: None,
            error: Some(e),
        }));
    }

    // 5. Parse body + build workflow inputs
    let body_value: serde_json::Value = if body.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&body).unwrap_or_else(|_| {
            serde_json::Value::String(String::from_utf8_lossy(&body).to_string())
        })
    };

    let headers_value: serde_json::Value = {
        let map: HashMap<String, String> = headers.iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|vs| (k.as_str().to_string(), vs.to_string())))
            .collect();
        serde_json::to_value(map).unwrap_or_default()
    };

    let query_value = serde_json::Value::Object(serde_json::Map::new());

    let mut inputs = HashMap::new();
    inputs.insert("__webhook_body".to_string(), body_value.clone());
    inputs.insert("__webhook_headers".to_string(), headers_value);
    inputs.insert("__webhook_query".to_string(), query_value);
    inputs.insert("__webhook_method".to_string(), serde_json::Value::String(method.to_string()));
    // Also inject body as "input" for standard Input nodes
    inputs.insert("input".to_string(), body_value);

    // 6. Load workflow + settings
    let (graph_json, all_settings, workflow_name, agent_id) = {
        let conn = match state.db.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(WebhookResponse {
                    run_id: None, status: "error".into(), output: None,
                    error: Some(format!("DB lock error: {e}")),
                }));
            }
        };

        let wf = conn.query_row(
            "SELECT name, graph_json, agent_id FROM workflows WHERE id = ?1 AND is_archived = 0",
            params![route.workflow_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?)),
        );
        let (name, graph, wf_agent_id) = match wf {
            Ok(r) => r,
            Err(e) => {
                return (StatusCode::NOT_FOUND, Json(WebhookResponse {
                    run_id: None, status: "error".into(), output: None,
                    error: Some(format!("Workflow not found: {e}")),
                }));
            }
        };

        let agent = wf_agent_id.filter(|id| !id.is_empty()).unwrap_or_else(|| {
            conn.query_row(
                "SELECT id FROM agents WHERE is_archived = 0 ORDER BY created_at LIMIT 1",
                [], |row| row.get::<_, String>(0),
            ).unwrap_or_default()
        });

        let mut stmt = conn.prepare("SELECT key, value FROM settings").unwrap();
        let mut settings = HashMap::<String, String>::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).unwrap();
        for row in rows.flatten() {
            settings.insert(row.0, row.1);
        }

        (graph, settings, name, agent)
    };

    // 7. Validate workflow
    match validate_graph_json(&graph_json) {
        Ok(v) if !v.valid => {
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(WebhookResponse {
                run_id: None, status: "error".into(), output: None,
                error: Some(format!("Invalid workflow: {}", v.errors.join("; "))),
            }));
        }
        Err(e) => {
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(WebhookResponse {
                run_id: None, status: "error".into(), output: None,
                error: Some(e),
            }));
        }
        _ => {}
    }

    // 8. Create session
    let session_id = Uuid::new_v4().to_string();
    let now = now_iso();
    {
        let conn = match state.db.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(WebhookResponse {
                    run_id: None, status: "error".into(), output: None,
                    error: Some(format!("DB lock error: {e}")),
                }));
            }
        };
        if let Err(e) = conn.execute(
            "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
            params![session_id, agent_id, format!("Webhook: {}", workflow_name), now, now],
        ) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(WebhookResponse {
                run_id: None, status: "error".into(), output: None,
                error: Some(format!("Failed to create session: {e}")),
            }));
        }
    }

    // 9. Log trigger fire
    let log_id = Uuid::new_v4().to_string();
    {
        let conn = match state.db.conn.lock() {
            Ok(c) => c,
            Err(_) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(WebhookResponse {
                    run_id: None, status: "error".into(), output: None,
                    error: Some("DB lock error".into()),
                }));
            }
        };
        let _ = conn.execute(
            "INSERT INTO trigger_log (id, trigger_id, run_id, fired_at, status) VALUES (?1, ?2, ?3, ?4, 'fired')",
            params![log_id, route.trigger_id, session_id, now],
        );
        let _ = conn.execute(
            "UPDATE triggers SET last_fired = ?1, fire_count = fire_count + 1, updated_at = ?1 WHERE id = ?2",
            params![now, route.trigger_id],
        );
    }

    // 10. Execute workflow
    if route.response_mode == ResponseMode::Immediate {
        // Fire-and-forget: spawn execution, return 202 immediately
        let db = state.db.clone();
        let sidecar = state.sidecar.clone();
        let app = state.app_handle.clone();
        let sid = session_id.clone();
        let log_id_clone = log_id.clone();

        tauri::async_runtime::spawn(async move {
            let result = execute_workflow_ephemeral(
                &db, &sidecar, &app, &sid, &graph_json, &inputs, &all_settings, false,
            ).await;

            // Update trigger log with result
            if let Ok(conn) = db.conn.lock() {
                let status = match &result {
                    Ok(_) => "completed",
                    Err(_) => "error",
                };
                let _ = conn.execute(
                    "UPDATE trigger_log SET status = ?1 WHERE id = ?2",
                    params![status, log_id_clone],
                );
            }
        });

        (StatusCode::ACCEPTED, Json(WebhookResponse {
            run_id: Some(session_id),
            status: "accepted".into(),
            output: None,
            error: None,
        }))
    } else {
        // Wait mode: execute and return the result
        let result = execute_workflow_ephemeral(
            &state.db, &state.sidecar, &state.app_handle,
            &session_id, &graph_json, &inputs, &all_settings, false,
        ).await;

        match result {
            Ok(run_result) => {
                // Update log
                if let Ok(conn) = state.db.conn.lock() {
                    let _ = conn.execute(
                        "UPDATE trigger_log SET status = 'completed' WHERE id = ?1",
                        params![log_id],
                    );
                }
                let output = run_result.outputs.values().next().cloned();
                (StatusCode::OK, Json(WebhookResponse {
                    run_id: Some(session_id),
                    status: "completed".into(),
                    output,
                    error: run_result.error,
                }))
            }
            Err(e) => {
                if let Ok(conn) = state.db.conn.lock() {
                    let _ = conn.execute(
                        "UPDATE trigger_log SET status = 'error', metadata = ?1 WHERE id = ?2",
                        params![serde_json::json!({"error": e}).to_string(), log_id],
                    );
                }
                (StatusCode::INTERNAL_SERVER_ERROR, Json(WebhookResponse {
                    run_id: Some(session_id),
                    status: "error".into(),
                    output: None,
                    error: Some(e),
                }))
            }
        }
    }
}

/// Start the webhook server on the given port. Returns a shutdown sender.
pub async fn start_server(
    state: WebhookState,
    port: u16,
) -> Result<oneshot::Sender<()>, String> {
    let router = build_router(state);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| format!("Failed to bind webhook server on port {}: {e}", port))?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    eprintln!("[webhook] Server starting on http://{}", addr);

    tauri::async_runtime::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
                eprintln!("[webhook] Server shutting down");
            })
            .await
            .unwrap_or_else(|e| eprintln!("[webhook] Server error: {e}"));
    });

    Ok(shutdown_tx)
}

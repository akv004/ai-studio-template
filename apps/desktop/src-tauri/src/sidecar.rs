use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State, Window};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8765;
const TOOL_APPROVAL_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize)]
pub struct SidecarStatus {
    running: bool,
    host: String,
    port: u16,
}

#[derive(Debug, Serialize)]
pub struct SidecarProxyResponse {
    status: u16,
    json: Option<serde_json::Value>,
    text: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ToolApprovalRequest {
    id: String,
    method: String,
    path: String,
    body: Option<serde_json::Value>,
}

/// SSE chunk from /chat/stream endpoint
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum StreamChunk {
    #[serde(rename = "token")]
    Token { content: String, index: i64 },
    #[serde(rename = "done")]
    Done { content: String, usage: serde_json::Value },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Default)]
struct SidecarInner {
    child: Option<Child>,
    token: Option<String>,
    host: String,
    port: u16,
}

impl SidecarInner {
    fn new() -> Self {
        Self {
            child: None,
            token: None,
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

#[derive(Clone)]
pub struct SidecarManager {
    inner: Arc<Mutex<SidecarInner>>,
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SidecarInner::new())),
        }
    }
}

impl SidecarManager {
    async fn is_healthy(&self) -> bool {
        let inner = self.inner.lock().await;
        let url = format!("{}/health", inner.base_url());
        drop(inner);

        let client = reqwest::Client::new();
        match client.get(url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    fn repo_root_from_manifest() -> PathBuf {
        // CARGO_MANIFEST_DIR points to `apps/desktop/src-tauri` in this repo.
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."))
    }

    fn sidecar_script_path() -> PathBuf {
        Self::repo_root_from_manifest().join("apps/sidecar/server.py")
    }

    fn spawn_sidecar(token: &str, host: &str, port: u16) -> Result<Child, String> {
        let script = Self::sidecar_script_path();
        if !script.exists() {
            return Err(format!(
                "Sidecar script not found at {}",
                script.to_string_lossy()
            ));
        }

        let cwd = Self::repo_root_from_manifest();
        // Prefer the project-local venv (apps/sidecar/.venv/bin/python)
        let venv_python = cwd.join("apps/sidecar/.venv/bin/python");
        let python_override = std::env::var("AI_STUDIO_PYTHON").ok();
        let python_candidates = python_override
            .into_iter()
            .chain(
                venv_python
                    .exists()
                    .then(|| venv_python.to_string_lossy().to_string()),
            )
            .chain(["python3".to_string(), "python".to_string()]);

        let mut last_err: Option<String> = None;
        for python in python_candidates {
            let mut cmd = Command::new(&python);
            cmd.arg(script.as_os_str())
                .current_dir(&cwd)
                .env("HOST", host)
                .env("PORT", port.to_string())
                .env("AI_STUDIO_TOKEN", token)
                .env("TOOLS_MODE", "sandboxed")
                .env("PYTHONUNBUFFERED", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            match cmd.spawn() {
                Ok(child) => return Ok(child),
                Err(e) => last_err = Some(format!("Failed to spawn `{python}`: {e}")),
            }
        }

        Err(last_err.unwrap_or_else(|| "Failed to spawn sidecar".to_string()))
    }

    pub(crate) async fn start(&self, _app: &AppHandle) -> Result<SidecarStatus, String> {
        // If we already own a running sidecar (child + token), verify health and return.
        {
            let inner = self.inner.lock().await;
            if inner.child.is_some() && inner.token.is_some() {
                drop(inner);
                if self.is_healthy().await {
                    let inner = self.inner.lock().await;
                    return Ok(SidecarStatus {
                        running: true,
                        host: inner.host.clone(),
                        port: inner.port,
                    });
                }
                // Our child died — fall through to kill + respawn.
            }
        }

        // Kill any existing child we own.
        {
            let mut inner = self.inner.lock().await;
            if let Some(mut child) = inner.child.take() {
                eprintln!("[sidecar] Killing previous sidecar process");
                let _ = child.kill();
                let _ = child.wait();
            }
            inner.token = None;
        }

        // Kill any orphaned sidecar on our port (from a previous Tauri hot-reload).
        // This is the key fix: after hot-reload, our SidecarManager is fresh (no child/token)
        // but a stale sidecar may still be listening. It passes /health but rejects our new token.
        #[cfg(unix)]
        {
            let port = { self.inner.lock().await.port };
            eprintln!("[sidecar] Killing orphaned processes on port {}", port);
            let _ = std::process::Command::new("fuser")
                .args(["-k", &format!("{}/tcp", port)])
                .output();
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let token = Uuid::new_v4().to_string();
        let (host, port) = {
            let inner = self.inner.lock().await;
            (inner.host.clone(), inner.port)
        };

        let child = Self::spawn_sidecar(&token, &host, port)?;

        {
            let mut inner = self.inner.lock().await;
            inner.child = Some(child);
            inner.token = Some(token);
        }

        // Wait for health.
        for _ in 0..100 {
            if self.is_healthy().await {
                let inner = self.inner.lock().await;
                return Ok(SidecarStatus {
                    running: true,
                    host: inner.host.clone(),
                    port: inner.port,
                });
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let _ = self.stop().await;
        Err(format!(
            "Sidecar did not become healthy in time. Ensure Python deps are installed (pip install -r apps/sidecar/requirements.txt) and port {port} is free."
        ))
    }

    pub(crate) async fn stop(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        if let Some(mut child) = inner.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        inner.token = None;
        Ok(())
    }

    pub(crate) async fn status(&self) -> SidecarStatus {
        let running = self.is_healthy().await;
        let inner = self.inner.lock().await;
        SidecarStatus {
            running,
            host: inner.host.clone(),
            port: inner.port,
        }
    }

    pub(crate) async fn token(&self) -> Option<String> {
        let inner = self.inner.lock().await;
        inner.token.clone()
    }

    /// Direct HTTP request to sidecar — used by send_message (bypasses tool approval gate).
    pub(crate) async fn proxy_request(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let base_url = {
            let inner = self.inner.lock().await;
            inner.base_url()
        };
        let url = format!("{base_url}{path}");
        let token = self.token().await;

        let client = reqwest::Client::new();
        let http_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| "Invalid HTTP method".to_string())?;
        let mut builder = client.request(http_method, &url);
        if let Some(t) = token {
            builder = builder.header("x-ai-studio-token", t);
        }
        if let Some(b) = body {
            builder = builder.json(&b);
        }

        let resp = builder
            .send()
            .await
            .map_err(|e| format!("Sidecar request failed: {e}"))?;

        let status = resp.status();
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

        if !status.is_success() {
            let text = String::from_utf8_lossy(&bytes);
            return Err(format!("Sidecar returned {status}: {text}"));
        }

        serde_json::from_slice(&bytes)
            .map_err(|e| format!("Failed to parse sidecar response: {e}"))
    }

    /// Streaming HTTP request to sidecar — consumes SSE line by line.
    /// Calls `on_token` for each token chunk, returns (full_content, usage) on done.
    pub(crate) async fn proxy_request_stream<F>(
        &self,
        path: &str,
        body: serde_json::Value,
        mut on_token: F,
    ) -> Result<(String, serde_json::Value), String>
    where
        F: FnMut(&str, i64),
    {
        let base_url = {
            let inner = self.inner.lock().await;
            inner.base_url()
        };
        let url = format!("{base_url}{path}");
        let token = self.token().await;

        let client = reqwest::Client::new();
        let mut builder = client.post(&url).json(&body);
        if let Some(t) = token {
            builder = builder.header("x-ai-studio-token", t);
        }

        let resp = builder
            .send()
            .await
            .map_err(|e| format!("Stream request failed: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Sidecar stream returned {status}: {text}"));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| format!("Stream read error: {e}"))?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            // Process complete SSE lines
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    match serde_json::from_str::<StreamChunk>(data) {
                        Ok(StreamChunk::Token { content, index }) => {
                            on_token(&content, index);
                        }
                        Ok(StreamChunk::Done { content, usage }) => {
                            return Ok((content, usage));
                        }
                        Ok(StreamChunk::Error { message }) => {
                            return Err(format!("Stream error from sidecar: {message}"));
                        }
                        Err(_) => {
                            eprintln!("[sidecar-stream] Unparseable SSE data: {}", data);
                        }
                    }
                }
            }
        }

        Err("Stream ended without done event".to_string())
    }
}

/// Connect to sidecar's WebSocket `/events` endpoint and bridge events to the UI.
/// Persists each event to SQLite and emits `agent_event` to the frontend.
pub fn spawn_event_bridge(app: &AppHandle, sidecar: &SidecarManager, db: &crate::db::Database) {
    let app_handle = app.clone();
    let sidecar_inner = sidecar.inner.clone();
    let db_conn = db.conn.clone();

    tauri::async_runtime::spawn(async move {
        // Wait a bit for sidecar to be fully ready
        tokio::time::sleep(Duration::from_secs(2)).await;

        loop {
            let (ws_url, token) = {
                let inner = sidecar_inner.lock().await;
                let url = format!("ws://{}:{}/events", inner.host, inner.port);
                let token = inner.token.clone();
                (url, token)
            };

            // If no token, sidecar isn't running yet — wait and retry
            if token.is_none() {
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
            let token = token.unwrap();

            let connect_result = tokio_tungstenite::connect_async(&ws_url).await;
            match connect_result {
                Ok((ws_stream, _)) => {
                    println!("[event-bridge] Connected to {}", ws_url);
                    let (mut write, mut read) = ws_stream.split();

                    // Send auth message
                    use futures_util::SinkExt;
                    let auth_msg = serde_json::json!({"type": "auth", "token": token});
                    if write
                        .send(tokio_tungstenite::tungstenite::Message::Text(auth_msg.to_string().into()))
                        .await
                        .is_err()
                    {
                        println!("[event-bridge] Auth send failed, reconnecting...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }

                    // Read events
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                                    let _ = persist_ws_event(&db_conn, &event);
                                    let _ = app_handle.emit("agent_event", &event);
                                }
                            }
                            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
                            Err(_) => break,
                            _ => {}
                        }
                    }

                    println!("[event-bridge] Disconnected, reconnecting...");
                }
                Err(e) => {
                    println!("[event-bridge] Connection failed: {e}, retrying...");
                }
            }

            // Reconnect with backoff
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

/// Calculate cost for an LLM response event based on model pricing.
fn calculate_cost(model: &str, input_tokens: i64, output_tokens: i64) -> f64 {
    // Pricing: (input_per_1m, output_per_1m)
    let (input_rate, output_rate) = if model.contains("opus") {
        (15.0, 75.0)
    } else if model.contains("sonnet") {
        (3.0, 15.0)
    } else if model.contains("haiku") {
        (0.80, 4.0)
    } else if model.contains("gpt-4o-mini") {
        (0.15, 0.60)
    } else if model.contains("gpt-4o") {
        (2.50, 10.0)
    } else if model.contains("gemini-2.0-flash") {
        (0.10, 0.40)
    } else if model.contains("gemini-1.5-pro") {
        (1.25, 5.0)
    } else if model.contains("gemini") {
        (0.10, 0.40)
    } else if model.contains("ollama") || model.contains("llama") || model.contains("qwen") {
        (0.0, 0.0)
    } else {
        (1.0, 3.0) // conservative default
    };

    (input_tokens as f64 / 1_000_000.0 * input_rate) + (output_tokens as f64 / 1_000_000.0 * output_rate)
}

/// Persist a WebSocket event to the events table in SQLite.
/// For `llm.response.completed` events, calculates and stores cost_usd.
fn persist_ws_event(
    conn: &std::sync::Mutex<rusqlite::Connection>,
    event: &serde_json::Value,
) -> Result<(), String> {
    let event_id = event["event_id"].as_str().unwrap_or_default();
    let event_type = event["type"].as_str().unwrap_or_default();
    let ts = event["ts"].as_str().unwrap_or_default();
    let session_id = event["session_id"].as_str().unwrap_or_default();
    let source = event["source"].as_str().unwrap_or_default();
    let seq = event["seq"].as_i64().unwrap_or(0);
    let payload = event.get("payload").map(|p| p.to_string()).unwrap_or_else(|| "{}".to_string());

    // Calculate cost for LLM response events
    let cost_usd = if event_type == "llm.response.completed" {
        let p = event.get("payload").unwrap_or(&serde_json::Value::Null);
        let model = p["model"].as_str().unwrap_or("");
        let input_tokens = p["input_tokens"].as_i64().unwrap_or(0);
        let output_tokens = p["output_tokens"].as_i64().unwrap_or(0);
        let cost = calculate_cost(model, input_tokens, output_tokens);
        if cost > 0.0 { Some(cost) } else { None }
    } else {
        event["cost_usd"].as_f64()
    };

    let conn = conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO events (event_id, type, ts, session_id, source, seq, payload, cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![event_id, event_type, ts, session_id, source, seq, payload, cost_usd],
    )
    .map_err(|e| format!("Failed to persist event: {e}"))?;

    // Update session cost totals
    if let Some(cost) = cost_usd {
        let p = event.get("payload").unwrap_or(&serde_json::Value::Null);
        let input_tokens = p["input_tokens"].as_i64().unwrap_or(0);
        let output_tokens = p["output_tokens"].as_i64().unwrap_or(0);
        let _ = conn.execute(
            "UPDATE sessions SET
                total_input_tokens = total_input_tokens + ?1,
                total_output_tokens = total_output_tokens + ?2,
                total_cost_usd = total_cost_usd + ?3,
                updated_at = ?4
             WHERE id = ?5",
            rusqlite::params![input_tokens, output_tokens, cost, ts, session_id],
        );
    }

    Ok(())
}

pub struct ApprovalManager {
    pending: Mutex<HashMap<String, oneshot::Sender<bool>>>,
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }
}

impl ApprovalManager {
    /// Register a pending approval and return its receiver.
    pub async fn register(&self, id: String, tx: oneshot::Sender<bool>) {
        self.pending.lock().await.insert(id, tx);
    }

    /// Remove a pending approval (cleanup after timeout/response).
    pub async fn remove(&self, id: &str) {
        self.pending.lock().await.remove(id);
    }
}

#[tauri::command]
pub async fn sidecar_start(app: AppHandle, sidecar: State<'_, SidecarManager>) -> Result<SidecarStatus, String> {
    sidecar.start(&app).await
}

#[tauri::command]
pub async fn sidecar_stop(sidecar: State<'_, SidecarManager>) -> Result<(), String> {
    sidecar.stop().await
}

#[tauri::command]
pub async fn sidecar_status(sidecar: State<'_, SidecarManager>) -> Result<SidecarStatus, String> {
    Ok(sidecar.status().await)
}

#[tauri::command]
pub async fn approve_tool_request(
    approvals: State<'_, ApprovalManager>,
    id: String,
    approve: bool,
) -> Result<(), String> {
    let mut pending = approvals.pending.lock().await;
    if let Some(tx) = pending.remove(&id) {
        let _ = tx.send(approve);
        Ok(())
    } else {
        Err("Unknown approval request id".to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct SidecarRequestBody {
    method: String,
    path: String,
    body: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn sidecar_request(
    window: Window,
    app: AppHandle,
    sidecar: State<'_, SidecarManager>,
    approvals: State<'_, ApprovalManager>,
    request: SidecarRequestBody,
) -> Result<SidecarProxyResponse, String> {
    // Ensure the sidecar is running.
    let _ = sidecar.start(&app).await?;

    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|_| "Invalid HTTP method".to_string())?;
    let mut path = request.path;
    if !path.starts_with('/') {
        path = format!("/{}", path);
    }

    // Gate tool calls behind an explicit user approval.
    if path.starts_with("/tools/") {
        let id = Uuid::new_v4().to_string();
        let payload = ToolApprovalRequest {
            id: id.clone(),
            method: method.to_string(),
            path: path.clone(),
            body: request.body.clone(),
        };

        let (tx, rx) = oneshot::channel::<bool>();
        approvals.pending.lock().await.insert(id.clone(), tx);
        window
            .emit("tool_approval_requested", payload)
            .map_err(|e| e.to_string())?;

        let approved = match tokio::time::timeout(TOOL_APPROVAL_TIMEOUT, rx).await {
            Ok(Ok(v)) => v,
            Ok(Err(_)) => false,
            Err(_) => false,
        };
        approvals.pending.lock().await.remove(&id);

        if !approved {
            return Err("Tool request denied (or timed out)".to_string());
        }
    }

    let base_url = {
        let inner = sidecar.inner.lock().await;
        inner.base_url()
    };
    let url = format!("{base_url}{path}");
    let token = sidecar.token().await;

    let client = reqwest::Client::new();
    let mut builder = client.request(method, url);
    if let Some(token) = token {
        builder = builder.header("x-ai-studio-token", token);
    }
    if let Some(body) = request.body {
        builder = builder.json(&body);
    }

    let resp = builder.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

    let (json, text) = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(v) => (Some(v), None),
        Err(_) => (
            None,
            Some(String::from_utf8_lossy(&bytes).to_string()),
        ),
    };

    Ok(SidecarProxyResponse { status, json, text })
}

#[cfg(test)]
mod tests {
    use super::StreamChunk;

    #[test]
    fn test_stream_chunk_token() {
        let data = r#"{"type":"token","content":"Hello","index":0}"#;
        let chunk: StreamChunk = serde_json::from_str(data).unwrap();
        match chunk {
            StreamChunk::Token { content, index } => {
                assert_eq!(content, "Hello");
                assert_eq!(index, 0);
            }
            _ => panic!("Expected Token variant"),
        }
    }

    #[test]
    fn test_stream_chunk_done() {
        let data = r#"{"type":"done","content":"Hello world","usage":{"prompt_tokens":10,"completion_tokens":5}}"#;
        let chunk: StreamChunk = serde_json::from_str(data).unwrap();
        match chunk {
            StreamChunk::Done { content, usage } => {
                assert_eq!(content, "Hello world");
                assert_eq!(usage["prompt_tokens"].as_i64().unwrap(), 10);
                assert_eq!(usage["completion_tokens"].as_i64().unwrap(), 5);
            }
            _ => panic!("Expected Done variant"),
        }
    }

    #[test]
    fn test_stream_chunk_error() {
        let data = r#"{"type":"error","message":"Provider timeout"}"#;
        let chunk: StreamChunk = serde_json::from_str(data).unwrap();
        match chunk {
            StreamChunk::Error { message } => {
                assert_eq!(message, "Provider timeout");
            }
            _ => panic!("Expected Error variant"),
        }
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State, Window};
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

#[derive(Debug, Serialize)]
pub struct ToolApprovalRequest {
    id: String,
    method: String,
    path: String,
    body: Option<serde_json::Value>,
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
        let python_override = std::env::var("AI_STUDIO_PYTHON").ok();
        let python_candidates = python_override
            .into_iter()
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
        // If already healthy, do nothing.
        if self.is_healthy().await {
            let inner = self.inner.lock().await;
            return Ok(SidecarStatus {
                running: true,
                host: inner.host.clone(),
                port: inner.port,
            });
        }

        // If a previous sidecar process exists but is unhealthy, stop it before spawning a new one.
        {
            let mut inner = self.inner.lock().await;
            if let Some(mut child) = inner.child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            inner.token = None;
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

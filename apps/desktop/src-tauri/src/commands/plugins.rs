use crate::db::{Database, now_iso};
use crate::error::AppError;
use crate::sidecar::SidecarManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: String,
    pub license: String,
    pub runtime: String,
    pub entry_point: String,
    pub transport: String,
    pub permissions: Vec<String>,
    pub provides_tools: bool,
    pub provides_node_types: Vec<String>,
    pub directory: String,
    pub enabled: bool,
    pub installed_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub installed: usize,
    pub updated: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConnectResult {
    pub tools: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStartupResult {
    pub connected: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    homepage: String,
    #[serde(default)]
    license: String,
    #[serde(default = "default_runtime")]
    runtime: String,
    entry_point: String,
    #[serde(default = "default_transport")]
    transport: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    provides: PluginProvides,
}

#[derive(Debug, Deserialize, Default)]
struct PluginProvides {
    #[serde(default)]
    tools: bool,
    #[serde(default)]
    node_types: Vec<String>,
}

fn default_runtime() -> String { "python".to_string() }
fn default_transport() -> String { "stdio".to_string() }

#[tauri::command]
pub fn list_plugins(db: tauri::State<'_, Database>) -> Result<Vec<Plugin>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, version, description, author, homepage, license,
                runtime, entry_point, transport, permissions,
                provides_tools, provides_node_types, directory, enabled,
                installed_at, updated_at
         FROM plugins ORDER BY name ASC"
    )?;

    let plugins = stmt.query_map([], |row| {
        let permissions_json: String = row.get(10)?;
        let node_types_json: String = row.get(12)?;
        Ok(Plugin {
            id: row.get(0)?,
            name: row.get(1)?,
            version: row.get(2)?,
            description: row.get(3)?,
            author: row.get(4)?,
            homepage: row.get(5)?,
            license: row.get(6)?,
            runtime: row.get(7)?,
            entry_point: row.get(8)?,
            transport: row.get(9)?,
            permissions: serde_json::from_str(&permissions_json).unwrap_or_default(),
            provides_tools: row.get::<_, i64>(11)? != 0,
            provides_node_types: serde_json::from_str(&node_types_json).unwrap_or_default(),
            directory: row.get(13)?,
            enabled: row.get::<_, i64>(14)? != 0,
            installed_at: row.get(15)?,
            updated_at: row.get(16)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(plugins)
}

#[tauri::command]
pub fn scan_plugins(db: tauri::State<'_, Database>) -> Result<ScanResult, AppError> {
    let plugin_dir = plugin_directory()?;
    let mut installed = 0;
    let mut updated = 0;
    let mut errors = Vec::new();

    // Create plugin directory if it doesn't exist
    if !plugin_dir.exists() {
        std::fs::create_dir_all(&plugin_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create plugin directory: {e}")))?;
        return Ok(ScanResult { installed, updated, errors });
    }

    // Scan each subdirectory for plugin.json
    let entries = std::fs::read_dir(&plugin_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read plugin directory: {e}")))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("Failed to read directory entry: {e}"));
                continue;
            }
        };

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("plugin.json");
        if !manifest_path.exists() {
            continue;
        }

        match read_and_install_plugin(&db, &path, &manifest_path) {
            Ok(is_new) => {
                if is_new { installed += 1; } else { updated += 1; }
            }
            Err(e) => {
                let dir_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                errors.push(format!("{}: {}", dir_name, e));
            }
        }
    }

    Ok(ScanResult { installed, updated, errors })
}

fn read_and_install_plugin(
    db: &Database,
    dir: &std::path::Path,
    manifest_path: &std::path::Path,
) -> Result<bool, String> {
    let content = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read plugin.json: {e}"))?;

    let manifest: PluginManifest = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid plugin.json: {e}"))?;

    // Validate required fields
    if manifest.id.is_empty() { return Err("Missing 'id' field".into()); }
    if manifest.name.is_empty() { return Err("Missing 'name' field".into()); }
    if manifest.entry_point.is_empty() { return Err("Missing 'entry_point' field".into()); }

    // Check entry point exists
    let entry_path = dir.join(&manifest.entry_point);
    if !entry_path.exists() {
        return Err(format!("Entry point '{}' not found", manifest.entry_point));
    }

    let now = now_iso();
    let dir_str = dir.to_string_lossy().to_string();
    let permissions_json = serde_json::to_string(&manifest.permissions)
        .unwrap_or_else(|_| "[]".to_string());
    let node_types_json = serde_json::to_string(&manifest.provides.node_types)
        .unwrap_or_else(|_| "[]".to_string());

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if plugin already exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM plugins WHERE id = ?1",
            params![manifest.id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        // Update existing plugin metadata (keep enabled state)
        conn.execute(
            "UPDATE plugins SET
                name = ?1, version = ?2, description = ?3, author = ?4,
                homepage = ?5, license = ?6, runtime = ?7, entry_point = ?8,
                transport = ?9, permissions = ?10, provides_tools = ?11,
                provides_node_types = ?12, directory = ?13, updated_at = ?14
             WHERE id = ?15",
            params![
                manifest.name, manifest.version, manifest.description,
                manifest.author, manifest.homepage, manifest.license,
                manifest.runtime, manifest.entry_point, manifest.transport,
                permissions_json, manifest.provides.tools as i64,
                node_types_json, dir_str, now, manifest.id,
            ],
        ).map_err(|e| format!("Failed to update plugin: {e}"))?;
        Ok(false)
    } else {
        // Insert new plugin (disabled by default)
        conn.execute(
            "INSERT INTO plugins (id, name, version, description, author, homepage, license,
                runtime, entry_point, transport, permissions, provides_tools,
                provides_node_types, directory, enabled, installed_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 0, ?15, ?16)",
            params![
                manifest.id, manifest.name, manifest.version, manifest.description,
                manifest.author, manifest.homepage, manifest.license,
                manifest.runtime, manifest.entry_point, manifest.transport,
                permissions_json, manifest.provides.tools as i64,
                node_types_json, dir_str, now, now,
            ],
        ).map_err(|e| format!("Failed to install plugin: {e}"))?;
        Ok(true)
    }
}

/// Build the shell command + args for a plugin based on its runtime and entry point.
fn build_plugin_command(runtime: &str, directory: &str, entry_point: &str) -> (String, Vec<String>) {
    let full_path = std::path::Path::new(directory).join(entry_point);
    let full_path_str = full_path.to_string_lossy().to_string();

    match runtime {
        "python" => ("python3".to_string(), vec![full_path_str]),
        "node" => ("node".to_string(), vec![full_path_str]),
        "binary" => (full_path_str, vec![]),
        _ => ("python3".to_string(), vec![full_path_str]),
    }
}

/// Connect a single plugin to the sidecar as an MCP server.
async fn connect_plugin_to_sidecar(
    sidecar: &SidecarManager,
    id: &str,
    runtime: &str,
    directory: &str,
    entry_point: &str,
) -> Result<Vec<String>, String> {
    let (command, args) = build_plugin_command(runtime, directory, entry_point);

    let body = serde_json::json!({
        "name": format!("plugin:{}", id),
        "transport": "stdio",
        "command": command,
        "args": args,
        "env": {},
    });

    let resp = sidecar.proxy_request("POST", "/mcp/connect", Some(body)).await?;

    // Extract discovered tool names from response
    let tools = resp.get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(tools)
}

/// Disconnect a plugin from the sidecar.
async fn disconnect_plugin_from_sidecar(
    sidecar: &SidecarManager,
    id: &str,
) -> Result<(), String> {
    let body = serde_json::json!({
        "name": format!("plugin:{}", id),
    });
    sidecar.proxy_request("POST", "/mcp/disconnect", Some(body)).await?;
    Ok(())
}

#[tauri::command]
pub async fn enable_plugin(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, SidecarManager>,
    id: String,
) -> Result<PluginConnectResult, AppError> {
    // 1. Read plugin metadata from DB
    let (runtime, directory, entry_point) = {
        let conn = db.conn.lock()?;
        conn.query_row(
            "SELECT runtime, directory, entry_point FROM plugins WHERE id = ?1",
            params![id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            )),
        ).map_err(|_| AppError::NotFound(format!("Plugin not found: {id}")))?
    };

    // 2. Connect to sidecar as MCP server
    let tools = connect_plugin_to_sidecar(&sidecar, &id, &runtime, &directory, &entry_point)
        .await
        .map_err(|e| AppError::Sidecar(format!("Failed to connect plugin: {e}")))?;

    // 3. Set enabled in DB (only after successful connect)
    {
        let conn = db.conn.lock()?;
        conn.execute(
            "UPDATE plugins SET enabled = 1, updated_at = ?1 WHERE id = ?2",
            params![now_iso(), id],
        )?;
    }

    Ok(PluginConnectResult { tools })
}

#[tauri::command]
pub async fn disable_plugin(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, SidecarManager>,
    id: String,
) -> Result<(), AppError> {
    // 1. Disconnect from sidecar (best-effort — don't fail if sidecar is down)
    let _ = disconnect_plugin_from_sidecar(&sidecar, &id).await;

    // 2. Set disabled in DB
    let conn = db.conn.lock()?;
    let rows = conn.execute(
        "UPDATE plugins SET enabled = 0, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), id],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Plugin not found: {id}")));
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_plugin(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, SidecarManager>,
    id: String,
) -> Result<(), AppError> {
    // 1. Check if plugin was enabled — if so, disconnect first
    let was_enabled = {
        let conn = db.conn.lock()?;
        conn.query_row(
            "SELECT enabled FROM plugins WHERE id = ?1",
            params![id],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) != 0
    };

    if was_enabled {
        let _ = disconnect_plugin_from_sidecar(&sidecar, &id).await;
    }

    // 2. Delete from DB
    let conn = db.conn.lock()?;
    let rows = conn.execute("DELETE FROM plugins WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Plugin not found: {id}")));
    }
    Ok(())
}

/// Connect all enabled plugins to the sidecar on app startup.
#[tauri::command]
pub async fn connect_enabled_plugins(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, SidecarManager>,
) -> Result<PluginStartupResult, AppError> {
    // Read all enabled plugins from DB
    let plugins = {
        let conn = db.conn.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, runtime, directory, entry_point FROM plugins WHERE enabled = 1"
        )?;
        let result = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        result
    };

    let mut connected = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for (id, runtime, directory, entry_point) in &plugins {
        match connect_plugin_to_sidecar(&sidecar, id, runtime, directory, entry_point).await {
            Ok(tools) => {
                println!("[plugins] Connected '{}' — {} tools", id, tools.len());
                connected += 1;
            }
            Err(e) => {
                eprintln!("[plugins] Failed to connect '{}': {}", id, e);
                errors.push(format!("{}: {}", id, e));
                failed += 1;
            }
        }
    }

    Ok(PluginStartupResult { connected, failed, errors })
}

/// Returns the plugin directory path (~/.ai-studio/plugins/)
fn plugin_directory() -> Result<std::path::PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("Cannot determine home directory".into()))?;
    Ok(home.join(".ai-studio").join("plugins"))
}

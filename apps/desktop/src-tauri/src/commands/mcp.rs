use crate::db::{Database, now_iso};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub env: serde_json::Value,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMcpServerRequest {
    pub name: String,
    #[serde(default = "default_transport")]
    pub transport: String,
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    pub url: Option<String>,
    #[serde(default = "default_env")]
    pub env: serde_json::Value,
}

fn default_transport() -> String { "stdio".to_string() }
fn default_env() -> serde_json::Value { serde_json::json!({}) }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMcpServerRequest {
    pub name: Option<String>,
    pub transport: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub env: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[tauri::command]
pub fn list_mcp_servers(db: tauri::State<'_, Database>) -> Result<Vec<McpServer>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, transport, command, args, url, env, enabled, created_at, updated_at
             FROM mcp_servers ORDER BY name ASC",
        )?;

    let servers = stmt
        .query_map([], |row| {
            let args_json: String = row.get(4)?;
            let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
            let env_json: String = row.get(6)?;
            let env: serde_json::Value = serde_json::from_str(&env_json)
                .unwrap_or(serde_json::json!({}));
            Ok(McpServer {
                id: row.get(0)?,
                name: row.get(1)?,
                transport: row.get(2)?,
                command: row.get(3)?,
                args,
                url: row.get(5)?,
                env,
                enabled: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(servers)
}

#[tauri::command]
pub fn add_mcp_server(
    db: tauri::State<'_, Database>,
    config: CreateMcpServerRequest,
) -> Result<McpServer, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let args_json = serde_json::to_string(&config.args).unwrap_or_else(|_| "[]".to_string());
    let env_json = serde_json::to_string(&config.env).unwrap_or_else(|_| "{}".to_string());

    let conn = db.conn.lock()?;
    conn.execute(
        "INSERT INTO mcp_servers (id, name, transport, command, args, url, env, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9)",
        params![
            id, config.name, config.transport, config.command,
            args_json, config.url, env_json, now, now,
        ],
    )
    .map_err(|e| AppError::Db(format!("Failed to add MCP server: {e}")))?;

    Ok(McpServer {
        id,
        name: config.name,
        transport: config.transport,
        command: config.command,
        args: config.args,
        url: config.url,
        env: config.env,
        enabled: true,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_mcp_server(
    db: tauri::State<'_, Database>,
    id: String,
    updates: UpdateMcpServerRequest,
) -> Result<McpServer, AppError> {
    let conn = db.conn.lock()?;
    let now = now_iso();

    let mut sets = vec!["updated_at = ?1".to_string()];
    let mut param_index = 2u32;
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now.clone())];

    if let Some(ref name) = updates.name {
        sets.push(format!("name = ?{param_index}"));
        values.push(Box::new(name.clone()));
        param_index += 1;
    }
    if let Some(ref transport) = updates.transport {
        sets.push(format!("transport = ?{param_index}"));
        values.push(Box::new(transport.clone()));
        param_index += 1;
    }
    if let Some(ref command) = updates.command {
        sets.push(format!("command = ?{param_index}"));
        values.push(Box::new(command.clone()));
        param_index += 1;
    }
    if let Some(ref args) = updates.args {
        let args_json = serde_json::to_string(args).unwrap_or_else(|_| "[]".to_string());
        sets.push(format!("args = ?{param_index}"));
        values.push(Box::new(args_json));
        param_index += 1;
    }
    if let Some(ref url) = updates.url {
        sets.push(format!("url = ?{param_index}"));
        values.push(Box::new(url.clone()));
        param_index += 1;
    }
    if let Some(ref env) = updates.env {
        let env_json = serde_json::to_string(env).unwrap_or_else(|_| "{}".to_string());
        sets.push(format!("env = ?{param_index}"));
        values.push(Box::new(env_json));
        param_index += 1;
    }
    if let Some(enabled) = updates.enabled {
        sets.push(format!("enabled = ?{param_index}"));
        values.push(Box::new(if enabled { 1i32 } else { 0 }));
        param_index += 1;
    }

    let sql = format!(
        "UPDATE mcp_servers SET {} WHERE id = ?{param_index}",
        sets.join(", ")
    );
    values.push(Box::new(id.clone()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let rows = conn
        .execute(&sql, param_refs.as_slice())
        .map_err(|e| AppError::Db(format!("Failed to update MCP server: {e}")))?;

    if rows == 0 {
        return Err(AppError::NotFound("MCP server not found".to_string()));
    }

    conn.query_row(
        "SELECT id, name, transport, command, args, url, env, enabled, created_at, updated_at
         FROM mcp_servers WHERE id = ?1",
        params![id],
        |row| {
            let args_json: String = row.get(4)?;
            let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
            let env_json: String = row.get(6)?;
            let env: serde_json::Value = serde_json::from_str(&env_json)
                .unwrap_or(serde_json::json!({}));
            Ok(McpServer {
                id: row.get(0)?,
                name: row.get(1)?,
                transport: row.get(2)?,
                command: row.get(3)?,
                args,
                url: row.get(5)?,
                env,
                enabled: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| AppError::NotFound(format!("MCP server not found: {e}")))
}

#[tauri::command]
pub fn remove_mcp_server(db: tauri::State<'_, Database>, id: String) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    let rows = conn
        .execute("DELETE FROM mcp_servers WHERE id = ?1", params![id])
        .map_err(|e| AppError::Db(format!("Failed to remove MCP server: {e}")))?;
    if rows == 0 {
        return Err(AppError::NotFound("MCP server not found".to_string()));
    }
    Ok(())
}

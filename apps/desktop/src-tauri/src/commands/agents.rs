use crate::db::{Database, now_iso};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================
// AGENT TYPES
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: String,
    pub model: String,
    pub system_prompt: String,
    pub temperature: f64,
    pub max_tokens: i64,
    pub tools: Vec<String>,
    pub tools_mode: String,
    pub mcp_servers: Vec<String>,
    pub approval_rules: Vec<serde_json::Value>,
    pub routing_mode: String,
    pub routing_rules: Vec<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    pub is_archived: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentRequest {
    pub name: String,
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i64,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default = "default_tools_mode")]
    pub tools_mode: String,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default = "default_routing_mode")]
    pub routing_mode: String,
    #[serde(default)]
    pub routing_rules: Vec<serde_json::Value>,
}

fn default_temperature() -> f64 { 0.7 }
fn default_max_tokens() -> i64 { 4096 }
fn default_tools_mode() -> String { "restricted".to_string() }
fn default_routing_mode() -> String { "single".to_string() }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i64>,
    pub tools: Option<Vec<String>>,
    pub tools_mode: Option<String>,
    pub mcp_servers: Option<Vec<String>>,
    pub approval_rules: Option<Vec<serde_json::Value>>,
    pub routing_mode: Option<String>,
    pub routing_rules: Option<Vec<serde_json::Value>>,
}

// ============================================
// AGENT COMMANDS
// ============================================

#[tauri::command]
pub fn list_agents(db: tauri::State<'_, Database>) -> Result<Vec<Agent>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, provider, model, system_prompt,
                    temperature, max_tokens, tools, tools_mode, mcp_servers,
                    approval_rules, created_at, updated_at, is_archived,
                    routing_mode, routing_rules
             FROM agents WHERE is_archived = 0
             ORDER BY updated_at DESC",
        )?;

    let agents = stmt
        .query_map([], |row| {
            let tools_json: String = row.get(8)?;
            let tools: Vec<String> =
                serde_json::from_str(&tools_json).unwrap_or_default();
            let mcp_json: String = row.get(10)?;
            let mcp_servers: Vec<String> =
                serde_json::from_str(&mcp_json).unwrap_or_default();
            let ar_json: String = row.get(11)?;
            let approval_rules: Vec<serde_json::Value> =
                serde_json::from_str(&ar_json).unwrap_or_default();
            let rr_json: String = row.get(16)?;
            let routing_rules: Vec<serde_json::Value> =
                serde_json::from_str(&rr_json).unwrap_or_default();
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                provider: row.get(3)?,
                model: row.get(4)?,
                system_prompt: row.get(5)?,
                temperature: row.get(6)?,
                max_tokens: row.get(7)?,
                tools,
                tools_mode: row.get(9)?,
                mcp_servers,
                approval_rules,
                routing_mode: row.get(15)?,
                routing_rules,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                is_archived: row.get::<_, i32>(14)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(agents)
}

#[tauri::command]
pub fn get_agent(db: tauri::State<'_, Database>, id: String) -> Result<Agent, AppError> {
    let conn = db.conn.lock()?;
    conn.query_row(
        "SELECT id, name, description, provider, model, system_prompt,
                temperature, max_tokens, tools, tools_mode, mcp_servers,
                approval_rules, created_at, updated_at, is_archived,
                routing_mode, routing_rules
         FROM agents WHERE id = ?1",
        params![id],
        |row| {
            let tools_json: String = row.get(8)?;
            let tools: Vec<String> =
                serde_json::from_str(&tools_json).unwrap_or_default();
            let mcp_json: String = row.get(10)?;
            let mcp_servers: Vec<String> =
                serde_json::from_str(&mcp_json).unwrap_or_default();
            let ar_json: String = row.get(11)?;
            let approval_rules: Vec<serde_json::Value> =
                serde_json::from_str(&ar_json).unwrap_or_default();
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                provider: row.get(3)?,
                model: row.get(4)?,
                system_prompt: row.get(5)?,
                temperature: row.get(6)?,
                max_tokens: row.get(7)?,
                tools,
                tools_mode: row.get(9)?,
                mcp_servers,
                approval_rules,
                routing_mode: row.get(15)?,
                routing_rules: {
                    let rr_json: String = row.get(16)?;
                    serde_json::from_str(&rr_json).unwrap_or_default()
                },
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                is_archived: row.get::<_, i32>(14)? != 0,
            })
        },
    )
    .map_err(|_| AppError::NotFound("Agent not found".into()))
}

#[tauri::command]
pub fn create_agent(
    db: tauri::State<'_, Database>,
    agent: CreateAgentRequest,
) -> Result<Agent, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let tools_json = serde_json::to_string(&agent.tools).unwrap_or_else(|_| "[]".to_string());
    let mcp_json = serde_json::to_string(&agent.mcp_servers).unwrap_or_else(|_| "[]".to_string());
    let rr_json = serde_json::to_string(&agent.routing_rules).unwrap_or_else(|_| "[]".to_string());

    let conn = db.conn.lock()?;
    conn.execute(
        "INSERT INTO agents (id, name, description, provider, model, system_prompt,
                             temperature, max_tokens, tools, tools_mode, mcp_servers,
                             routing_mode, routing_rules,
                             created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            id,
            agent.name,
            agent.description,
            agent.provider,
            agent.model,
            agent.system_prompt,
            agent.temperature,
            agent.max_tokens,
            tools_json,
            agent.tools_mode,
            mcp_json,
            agent.routing_mode,
            rr_json,
            now,
            now,
        ],
    )
    .map_err(|e| AppError::Db(format!("Failed to create agent: {e}")))?;

    Ok(Agent {
        id,
        name: agent.name,
        description: agent.description,
        provider: agent.provider,
        model: agent.model,
        system_prompt: agent.system_prompt,
        temperature: agent.temperature,
        max_tokens: agent.max_tokens,
        tools: agent.tools,
        tools_mode: agent.tools_mode,
        mcp_servers: agent.mcp_servers,
        approval_rules: vec![],
        routing_mode: agent.routing_mode,
        routing_rules: agent.routing_rules,
        created_at: now.clone(),
        updated_at: now,
        is_archived: false,
    })
}

#[tauri::command]
pub fn update_agent(
    db: tauri::State<'_, Database>,
    id: String,
    updates: UpdateAgentRequest,
) -> Result<Agent, AppError> {
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
    if let Some(ref desc) = updates.description {
        sets.push(format!("description = ?{param_index}"));
        values.push(Box::new(desc.clone()));
        param_index += 1;
    }
    if let Some(ref provider) = updates.provider {
        sets.push(format!("provider = ?{param_index}"));
        values.push(Box::new(provider.clone()));
        param_index += 1;
    }
    if let Some(ref model) = updates.model {
        sets.push(format!("model = ?{param_index}"));
        values.push(Box::new(model.clone()));
        param_index += 1;
    }
    if let Some(ref prompt) = updates.system_prompt {
        sets.push(format!("system_prompt = ?{param_index}"));
        values.push(Box::new(prompt.clone()));
        param_index += 1;
    }
    if let Some(temp) = updates.temperature {
        sets.push(format!("temperature = ?{param_index}"));
        values.push(Box::new(temp));
        param_index += 1;
    }
    if let Some(max_t) = updates.max_tokens {
        sets.push(format!("max_tokens = ?{param_index}"));
        values.push(Box::new(max_t));
        param_index += 1;
    }
    if let Some(ref tools) = updates.tools {
        let tools_json = serde_json::to_string(tools).unwrap_or_else(|_| "[]".to_string());
        sets.push(format!("tools = ?{param_index}"));
        values.push(Box::new(tools_json));
        param_index += 1;
    }
    if let Some(ref tools_mode) = updates.tools_mode {
        sets.push(format!("tools_mode = ?{param_index}"));
        values.push(Box::new(tools_mode.clone()));
        param_index += 1;
    }
    if let Some(ref mcp_servers) = updates.mcp_servers {
        let mcp_json = serde_json::to_string(mcp_servers).unwrap_or_else(|_| "[]".to_string());
        sets.push(format!("mcp_servers = ?{param_index}"));
        values.push(Box::new(mcp_json));
        param_index += 1;
    }
    if let Some(ref approval_rules) = updates.approval_rules {
        let ar_json = serde_json::to_string(approval_rules).unwrap_or_else(|_| "[]".to_string());
        sets.push(format!("approval_rules = ?{param_index}"));
        values.push(Box::new(ar_json));
        param_index += 1;
    }
    if let Some(ref routing_mode) = updates.routing_mode {
        sets.push(format!("routing_mode = ?{param_index}"));
        values.push(Box::new(routing_mode.clone()));
        param_index += 1;
    }
    if let Some(ref routing_rules) = updates.routing_rules {
        let rr_json = serde_json::to_string(routing_rules).unwrap_or_else(|_| "[]".to_string());
        sets.push(format!("routing_rules = ?{param_index}"));
        values.push(Box::new(rr_json));
        param_index += 1;
    }

    let sql = format!(
        "UPDATE agents SET {} WHERE id = ?{param_index}",
        sets.join(", ")
    );
    values.push(Box::new(id.clone()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let rows = conn
        .execute(&sql, param_refs.as_slice())
        .map_err(|e| AppError::Db(format!("Failed to update agent: {e}")))?;

    if rows == 0 {
        return Err(AppError::NotFound("Agent not found".into()));
    }

    drop(conn);
    get_agent(db, id)
}

#[tauri::command]
pub fn delete_agent(db: tauri::State<'_, Database>, id: String) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    let now = now_iso();
    let rows = conn
        .execute(
            "UPDATE agents SET is_archived = 1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| AppError::Db(format!("Failed to archive agent: {e}")))?;

    if rows == 0 {
        return Err(AppError::NotFound("Agent not found".into()));
    }
    Ok(())
}

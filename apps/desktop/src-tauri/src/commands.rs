// ============================================
// TAURI COMMANDS — Agent, Session, Run, Settings CRUD
// All backed by real SQLite persistence
// ============================================

use crate::db::{Database, now_iso};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
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
}

fn default_temperature() -> f64 { 0.7 }
fn default_max_tokens() -> i64 { 4096 }
fn default_tools_mode() -> String { "restricted".to_string() }

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
}

// ============================================
// AGENT COMMANDS
// ============================================

#[tauri::command]
pub fn list_agents(db: tauri::State<'_, Database>) -> Result<Vec<Agent>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, provider, model, system_prompt,
                    temperature, max_tokens, tools, tools_mode, mcp_servers,
                    approval_rules, created_at, updated_at, is_archived
             FROM agents WHERE is_archived = 0
             ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

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
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                is_archived: row.get::<_, i32>(14)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(agents)
}

#[tauri::command]
pub fn get_agent(db: tauri::State<'_, Database>, id: String) -> Result<Agent, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, name, description, provider, model, system_prompt,
                temperature, max_tokens, tools, tools_mode, mcp_servers,
                approval_rules, created_at, updated_at, is_archived
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
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                is_archived: row.get::<_, i32>(14)? != 0,
            })
        },
    )
    .map_err(|e| format!("Agent not found: {e}"))
}

#[tauri::command]
pub fn create_agent(
    db: tauri::State<'_, Database>,
    agent: CreateAgentRequest,
) -> Result<Agent, String> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let tools_json = serde_json::to_string(&agent.tools).unwrap_or_else(|_| "[]".to_string());
    let mcp_json = serde_json::to_string(&agent.mcp_servers).unwrap_or_else(|_| "[]".to_string());

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO agents (id, name, description, provider, model, system_prompt,
                             temperature, max_tokens, tools, tools_mode, mcp_servers,
                             created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
            now,
            now,
        ],
    )
    .map_err(|e| format!("Failed to create agent: {e}"))?;

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
) -> Result<Agent, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
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

    let sql = format!(
        "UPDATE agents SET {} WHERE id = ?{param_index}",
        sets.join(", ")
    );
    values.push(Box::new(id.clone()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let rows = conn
        .execute(&sql, param_refs.as_slice())
        .map_err(|e| format!("Failed to update agent: {e}"))?;

    if rows == 0 {
        return Err("Agent not found".to_string());
    }

    drop(conn);
    get_agent(db, id)
}

#[tauri::command]
pub fn delete_agent(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = now_iso();
    let rows = conn
        .execute(
            "UPDATE agents SET is_archived = 1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| format!("Failed to archive agent: {e}"))?;

    if rows == 0 {
        return Err("Agent not found".to_string());
    }
    Ok(())
}

// ============================================
// SESSION TYPES
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub status: String,
    pub message_count: i64,
    pub event_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
    pub created_at: String,
    pub updated_at: String,
    pub ended_at: Option<String>,
    pub agent_name: Option<String>,
    pub agent_model: Option<String>,
    pub parent_session_id: Option<String>,
    pub branch_from_seq: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub seq: i64,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_usd: Option<f64>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

// ============================================
// SESSION COMMANDS
// ============================================

#[tauri::command]
pub fn list_sessions(db: tauri::State<'_, Database>) -> Result<Vec<Session>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.agent_id, s.title, s.status, s.message_count,
                    s.event_count, s.total_input_tokens, s.total_output_tokens,
                    s.total_cost_usd, s.created_at, s.updated_at, s.ended_at,
                    a.name, a.model,
                    s.parent_session_id, s.branch_from_seq
             FROM sessions s
             LEFT JOIN agents a ON a.id = s.agent_id
             WHERE s.status != 'archived'
             ORDER BY s.updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                title: row.get(2)?,
                status: row.get(3)?,
                message_count: row.get(4)?,
                event_count: row.get(5)?,
                total_input_tokens: row.get(6)?,
                total_output_tokens: row.get(7)?,
                total_cost_usd: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                ended_at: row.get(11)?,
                agent_name: row.get(12)?,
                agent_model: row.get(13)?,
                parent_session_id: row.get(14)?,
                branch_from_seq: row.get(15)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

#[tauri::command]
pub fn create_session(
    db: tauri::State<'_, Database>,
    agent_id: String,
    title: Option<String>,
) -> Result<Session, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_iso();

    let (agent_name, agent_model): (String, String) = conn
        .query_row(
            "SELECT name, model FROM agents WHERE id = ?1 AND is_archived = 0",
            params![agent_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Agent not found".to_string())?;

    let session_title = title.unwrap_or_else(|| format!("Chat with {agent_name}"));

    conn.execute(
        "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
        params![id, agent_id, session_title, now, now],
    )
    .map_err(|e| format!("Failed to create session: {e}"))?;

    Ok(Session {
        id,
        agent_id,
        title: session_title,
        status: "active".to_string(),
        message_count: 0,
        event_count: 0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cost_usd: 0.0,
        created_at: now.clone(),
        updated_at: now,
        ended_at: None,
        agent_name: Some(agent_name),
        agent_model: Some(agent_model),
        parent_session_id: None,
        branch_from_seq: None,
    })
}

#[tauri::command]
pub fn get_session_messages(
    db: tauri::State<'_, Database>,
    session_id: String,
) -> Result<Vec<Message>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, seq, role, content, model, provider,
                    input_tokens, output_tokens, cost_usd, duration_ms, created_at
             FROM messages WHERE session_id = ?1
             ORDER BY seq ASC",
        )
        .map_err(|e| e.to_string())?;

    let messages = stmt
        .query_map(params![session_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                session_id: row.get(1)?,
                seq: row.get(2)?,
                role: row.get(3)?,
                content: row.get(4)?,
                model: row.get(5)?,
                provider: row.get(6)?,
                input_tokens: row.get(7)?,
                output_tokens: row.get(8)?,
                cost_usd: row.get(9)?,
                duration_ms: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(messages)
}

#[tauri::command]
pub fn branch_session(
    db: tauri::State<'_, Database>,
    session_id: String,
    seq: i64,
) -> Result<Session, String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| format!("Failed to start transaction: {e}"))?;

    // 1. Look up parent session
    let (agent_id, parent_title): (String, String) = tx
        .query_row(
            "SELECT agent_id, title FROM sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Parent session not found".to_string())?;

    // 2. Look up agent
    let (agent_name, agent_model): (String, String) = tx
        .query_row(
            "SELECT name, model FROM agents WHERE id = ?1",
            params![agent_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Agent not found".to_string())?;

    // 3. Create new session (strip nested "Branch of " prefix)
    let new_id = Uuid::new_v4().to_string();
    let now = now_iso();
    let base_title = parent_title.strip_prefix("Branch of ").unwrap_or(&parent_title);
    let branch_title = format!("Branch of {base_title}");

    tx.execute(
        "INSERT INTO sessions (id, agent_id, title, status, parent_session_id, branch_from_seq, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6, ?7)",
        params![new_id, agent_id, branch_title, session_id, seq, now, now],
    )
    .map_err(|e| format!("Failed to create branch session: {e}"))?;

    // 4. Copy messages where seq <= branch point
    let mut stmt = tx
        .prepare(
            "SELECT seq, role, content, model, provider, input_tokens, output_tokens,
                    cost_usd, duration_ms, created_at
             FROM messages WHERE session_id = ?1 AND seq <= ?2
             ORDER BY seq ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(i64, String, String, Option<String>, Option<String>,
                    Option<i64>, Option<i64>, Option<f64>, Option<i64>, String)> = stmt
        .query_map(params![session_id, seq], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);

    let msg_count = rows.len() as i64;
    let mut total_in: i64 = 0;
    let mut total_out: i64 = 0;
    let mut total_cost: f64 = 0.0;
    for (m_seq, role, content, model, provider, in_tok, out_tok, cost, dur, created) in &rows {
        let msg_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO messages (id, session_id, seq, role, content, model, provider,
                                   input_tokens, output_tokens, cost_usd, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![msg_id, new_id, m_seq, role, content, model, provider,
                    in_tok, out_tok, cost, dur, created],
        )
        .map_err(|e| format!("Failed to copy message: {e}"))?;
        total_in += in_tok.unwrap_or(0);
        total_out += out_tok.unwrap_or(0);
        total_cost += cost.unwrap_or(0.0);
    }

    // 5. Update counters on new session
    tx.execute(
        "UPDATE sessions SET message_count = ?1, total_input_tokens = ?2,
                total_output_tokens = ?3, total_cost_usd = ?4 WHERE id = ?5",
        params![msg_count, total_in, total_out, total_cost, new_id],
    )
    .map_err(|e| format!("Failed to update session counters: {e}"))?;

    tx.commit().map_err(|e| format!("Failed to commit branch: {e}"))?;

    Ok(Session {
        id: new_id,
        agent_id,
        title: branch_title,
        status: "active".to_string(),
        message_count: msg_count,
        event_count: 0,
        total_input_tokens: total_in,
        total_output_tokens: total_out,
        total_cost_usd: total_cost,
        created_at: now.clone(),
        updated_at: now,
        ended_at: None,
        agent_name: Some(agent_name),
        agent_model: Some(agent_model),
        parent_session_id: Some(session_id),
        branch_from_seq: Some(seq),
    })
}

#[tauri::command]
pub fn delete_session(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute("DELETE FROM sessions WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete session: {e}"))?;
    if rows == 0 {
        return Err("Session not found".to_string());
    }
    Ok(())
}

// ============================================
// EVENT COMMANDS (Inspector reads these)
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub event_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub ts: String,
    pub session_id: String,
    pub source: String,
    pub seq: i64,
    pub payload: serde_json::Value,
    pub cost_usd: Option<f64>,
}

#[tauri::command]
pub fn get_session_events(
    db: tauri::State<'_, Database>,
    session_id: String,
) -> Result<Vec<Event>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT event_id, type, ts, session_id, source, seq, payload, cost_usd
             FROM events WHERE session_id = ?1
             ORDER BY seq ASC",
        )
        .map_err(|e| e.to_string())?;

    let events = stmt
        .query_map(params![session_id], |row| {
            let payload_str: String = row.get(6)?;
            let payload: serde_json::Value =
                serde_json::from_str(&payload_str)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
            Ok(Event {
                event_id: row.get(0)?,
                event_type: row.get(1)?,
                ts: row.get(2)?,
                session_id: row.get(3)?,
                source: row.get(4)?,
                seq: row.get(5)?,
                payload,
                cost_usd: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(events)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub total_events: i64,
    pub total_messages: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
    pub models_used: Vec<String>,
}

#[tauri::command]
pub fn get_session_stats(
    db: tauri::State<'_, Database>,
    session_id: String,
) -> Result<SessionStats, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (total_events, total_messages, total_input, total_output, total_cost): (
        i64, i64, i64, i64, f64,
    ) = conn
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM events WHERE session_id = ?1),
                (SELECT COUNT(*) FROM messages WHERE session_id = ?1),
                COALESCE((SELECT SUM(input_tokens) FROM messages WHERE session_id = ?1), 0),
                COALESCE((SELECT SUM(output_tokens) FROM messages WHERE session_id = ?1), 0),
                COALESCE((SELECT SUM(cost_usd) FROM messages WHERE session_id = ?1), 0.0)",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT DISTINCT model FROM messages WHERE session_id = ?1 AND model IS NOT NULL")
        .map_err(|e| e.to_string())?;
    let models: Vec<String> = stmt
        .query_map(params![session_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(SessionStats {
        total_events,
        total_messages,
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_cost_usd: total_cost,
        models_used: models,
    })
}

// ============================================
// RUN COMMANDS
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: String,
    pub agent_id: String,
    pub session_id: Option<String>,
    pub name: String,
    pub input: String,
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub total_events: i64,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
    pub duration_ms: Option<i64>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub agent_name: Option<String>,
}

#[tauri::command]
pub fn list_runs(db: tauri::State<'_, Database>) -> Result<Vec<Run>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.agent_id, r.session_id, r.name, r.input, r.status,
                    r.output, r.error, r.total_events, r.total_tokens,
                    r.total_cost_usd, r.duration_ms, r.created_at,
                    r.started_at, r.completed_at, a.name
             FROM runs r
             LEFT JOIN agents a ON a.id = r.agent_id
             ORDER BY r.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let runs = stmt
        .query_map([], |row| {
            Ok(Run {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                session_id: row.get(2)?,
                name: row.get(3)?,
                input: row.get(4)?,
                status: row.get(5)?,
                output: row.get(6)?,
                error: row.get(7)?,
                total_events: row.get(8)?,
                total_tokens: row.get(9)?,
                total_cost_usd: row.get(10)?,
                duration_ms: row.get(11)?,
                created_at: row.get(12)?,
                started_at: row.get(13)?,
                completed_at: row.get(14)?,
                agent_name: row.get(15)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(runs)
}

// ============================================
// RUN CREATE / CANCEL / GET
// ============================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRunRequest {
    pub agent_id: String,
    pub input: String,
    pub name: Option<String>,
}

/// Create a new run record and fire off execution asynchronously.
/// Returns immediately with a `pending` run; the background task will
/// update status → running → completed/failed.
#[tauri::command]
pub async fn create_run(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    app: tauri::AppHandle,
    request: CreateRunRequest,
) -> Result<Run, String> {
    let run_id = Uuid::new_v4().to_string();
    let now = now_iso();

    // Look up agent
    let (agent_name, provider, model, system_prompt) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT name, provider, model, system_prompt FROM agents WHERE id = ?1 AND is_archived = 0",
            params![request.agent_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            )),
        )
        .map_err(|_| "Agent not found".to_string())?
    };

    // Create a dedicated session for this run
    let session_id = Uuid::new_v4().to_string();
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let session_title = format!("Run: {}", request.name.as_deref().unwrap_or(&request.input[..request.input.len().min(50)]));
        conn.execute(
            "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
            params![session_id, request.agent_id, session_title, now, now],
        )
        .map_err(|e| format!("Failed to create run session: {e}"))?;
    }

    let run_name = request.name.unwrap_or_else(|| {
        let preview = if request.input.len() > 60 {
            format!("{}...", &request.input[..57])
        } else {
            request.input.clone()
        };
        preview
    });

    // Insert run record
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO runs (id, agent_id, session_id, name, input, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
            params![run_id, request.agent_id, session_id, run_name, request.input, now],
        )
        .map_err(|e| format!("Failed to create run: {e}"))?;
    }

    let run = Run {
        id: run_id.clone(),
        agent_id: request.agent_id.clone(),
        session_id: Some(session_id.clone()),
        name: run_name.clone(),
        input: request.input.clone(),
        status: "pending".to_string(),
        output: None,
        error: None,
        total_events: 0,
        total_tokens: 0,
        total_cost_usd: 0.0,
        duration_ms: None,
        created_at: now.clone(),
        started_at: None,
        completed_at: None,
        agent_name: Some(agent_name),
    };

    // Load provider config from settings (same as send_message)
    let provider_config = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let prefix = format!("provider.{}.", provider);
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings WHERE key LIKE ?1")
            .map_err(|e| e.to_string())?;
        let mut config = serde_json::Map::new();
        let rows = stmt
            .query_map(params![format!("{}%", prefix)], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, value))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (key, value) = row.map_err(|e| e.to_string())?;
            let field = key.strip_prefix(&prefix).unwrap_or(&key);
            let clean_value = value.trim_matches('"').to_string();
            config.insert(field.to_string(), serde_json::Value::String(clean_value));
        }
        config
    };

    // Spawn background execution
    let db_clone = db.inner().clone();
    let sidecar_clone = sidecar.inner().clone();
    let run_id_bg = run_id.clone();
    let session_id_bg = session_id;
    let input_bg = request.input.clone();
    let agent_id_bg = request.agent_id;

    tauri::async_runtime::spawn(async move {
        execute_run(
            &db_clone, &sidecar_clone, &app,
            &run_id_bg, &session_id_bg, &agent_id_bg,
            &input_bg, &provider, &model, &system_prompt,
            &provider_config,
        ).await;
    });

    Ok(run)
}

/// Background run execution — calls sidecar /chat and updates run record.
async fn execute_run(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    app: &tauri::AppHandle,
    run_id: &str,
    session_id: &str,
    _agent_id: &str,
    input: &str,
    provider: &str,
    model: &str,
    system_prompt: &str,
    provider_config: &serde_json::Map<String, serde_json::Value>,
) {
    let started_at = now_iso();

    // Update status → running
    {
        if let Ok(conn) = db.conn.lock() {
            let _ = conn.execute(
                "UPDATE runs SET status = 'running', started_at = ?1 WHERE id = ?2",
                params![started_at, run_id],
            );
        }
    }

    // Emit status update to UI
    let _ = app.emit("run_status_changed", serde_json::json!({
        "runId": run_id, "status": "running",
    }));

    let start_time = std::time::Instant::now();

    // Build sidecar request (same as send_message)
    let api_key = provider_config.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let base_url = provider_config.get("base_url")
        .or_else(|| provider_config.get("endpoint"))
        .and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut extra_config = serde_json::Map::new();
    for (k, v) in provider_config {
        if k != "api_key" && k != "base_url" && k != "endpoint" {
            extra_config.insert(k.clone(), v.clone());
        }
    }

    let mut chat_body = serde_json::json!({
        "message": input,
        "conversation_id": session_id,
        "provider": provider,
        "model": model,
        "system_prompt": system_prompt,
        "tools_enabled": true,
    });
    if !api_key.is_empty() {
        chat_body["api_key"] = serde_json::Value::String(api_key);
    }
    if !base_url.is_empty() {
        chat_body["base_url"] = serde_json::Value::String(base_url);
    }
    if !extra_config.is_empty() {
        chat_body["extra_config"] = serde_json::Value::Object(extra_config);
    }

    // Call sidecar
    let result = sidecar.proxy_request("POST", "/chat", Some(chat_body)).await;
    let duration_ms = start_time.elapsed().as_millis() as i64;
    let completed_at = now_iso();

    match result {
        Ok(resp) => {
            let content = resp.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let usage = resp.get("usage");
            let input_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
            let output_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
            let total_tokens = input_tokens + output_tokens;

            if let Ok(conn) = db.conn.lock() {
                let _ = conn.execute(
                    "UPDATE runs SET status = 'completed', output = ?1, total_tokens = ?2,
                     duration_ms = ?3, completed_at = ?4 WHERE id = ?5",
                    params![content, total_tokens, duration_ms, completed_at, run_id],
                );
            }

            let _ = app.emit("run_status_changed", serde_json::json!({
                "runId": run_id, "status": "completed",
            }));
        }
        Err(e) => {
            if let Ok(conn) = db.conn.lock() {
                let _ = conn.execute(
                    "UPDATE runs SET status = 'failed', error = ?1, duration_ms = ?2,
                     completed_at = ?3 WHERE id = ?4",
                    params![e.to_string(), duration_ms, completed_at, run_id],
                );
            }

            let _ = app.emit("run_status_changed", serde_json::json!({
                "runId": run_id, "status": "failed", "error": e.to_string(),
            }));
        }
    }
}

#[tauri::command]
pub fn cancel_run(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = now_iso();
    let rows = conn
        .execute(
            "UPDATE runs SET status = 'cancelled', completed_at = ?1 WHERE id = ?2 AND status IN ('pending', 'running')",
            params![now, id],
        )
        .map_err(|e| format!("Failed to cancel run: {e}"))?;
    if rows == 0 {
        return Err("Run not found or already completed".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_run(db: tauri::State<'_, Database>, id: String) -> Result<Run, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT r.id, r.agent_id, r.session_id, r.name, r.input, r.status,
                r.output, r.error, r.total_events, r.total_tokens,
                r.total_cost_usd, r.duration_ms, r.created_at,
                r.started_at, r.completed_at, a.name
         FROM runs r
         LEFT JOIN agents a ON a.id = r.agent_id
         WHERE r.id = ?1",
        params![id],
        |row| {
            Ok(Run {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                session_id: row.get(2)?,
                name: row.get(3)?,
                input: row.get(4)?,
                status: row.get(5)?,
                output: row.get(6)?,
                error: row.get(7)?,
                total_events: row.get(8)?,
                total_tokens: row.get(9)?,
                total_cost_usd: row.get(10)?,
                duration_ms: row.get(11)?,
                created_at: row.get(12)?,
                started_at: row.get(13)?,
                completed_at: row.get(14)?,
                agent_name: row.get(15)?,
            })
        },
    )
    .map_err(|e| format!("Run not found: {e}"))
}

// ============================================
// DB WIPE COMMAND — dev utility
// ============================================

#[tauri::command]
pub fn wipe_database(db: tauri::State<'_, Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute_batch(
        "DELETE FROM events;
         DELETE FROM messages;
         DELETE FROM runs;
         DELETE FROM sessions;
         DELETE FROM workflows;
         DELETE FROM agents;
         DELETE FROM mcp_servers;
         DELETE FROM approval_rules;
         DELETE FROM settings;
         DELETE FROM provider_keys;"
    )
    .map_err(|e| format!("Failed to wipe database: {e}"))?;
    println!("[db] Database wiped — all data deleted");
    Ok(())
}

// ============================================
// SETTINGS COMMANDS
// ============================================

#[tauri::command]
pub fn get_all_settings(db: tauri::State<'_, Database>) -> Result<serde_json::Value, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let mut map = serde_json::Map::new();
    let rows = stmt
        .query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        let parsed: serde_json::Value =
            serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
        map.insert(key, parsed);
    }

    Ok(serde_json::Value::Object(map))
}

#[tauri::command]
pub fn set_setting(
    db: tauri::State<'_, Database>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let value_str = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value_str],
    )
    .map_err(|e| format!("Failed to save setting: {e}"))?;
    Ok(())
}

// ============================================
// PROVIDER KEY COMMANDS
// ============================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderKeyInfo {
    pub provider: String,
    pub has_key: bool,
    pub base_url: Option<String>,
    pub updated_at: String,
}

#[tauri::command]
pub fn list_provider_keys(db: tauri::State<'_, Database>) -> Result<Vec<ProviderKeyInfo>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT provider, base_url, updated_at FROM provider_keys")
        .map_err(|e| e.to_string())?;

    let keys = stmt
        .query_map([], |row| {
            Ok(ProviderKeyInfo {
                provider: row.get(0)?,
                has_key: true,
                base_url: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(keys)
}

#[tauri::command]
pub fn set_provider_key(
    db: tauri::State<'_, Database>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = now_iso();
    conn.execute(
        "INSERT OR REPLACE INTO provider_keys (provider, api_key, base_url, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![provider, api_key, base_url, now],
    )
    .map_err(|e| format!("Failed to save provider key: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_provider_key(db: tauri::State<'_, Database>, provider: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM provider_keys WHERE provider = ?1",
        params![provider],
    )
    .map_err(|e| format!("Failed to delete provider key: {e}"))?;
    Ok(())
}

// ============================================
// SEND MESSAGE — the core chat loop
// User → persist → sidecar LLM → persist response → record events
// ============================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub session_id: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    pub user_message: Message,
    pub assistant_message: Message,
}

#[tauri::command]
pub async fn send_message(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    request: SendMessageRequest,
) -> Result<SendMessageResponse, String> {
    let now = now_iso();

    // 1. Load session + agent info + provider config from settings
    let (provider, model, system_prompt, tools_mode, provider_config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let agent_id: String = conn
            .query_row(
                "SELECT agent_id FROM sessions WHERE id = ?1",
                params![request.session_id],
                |row| row.get(0),
            )
            .map_err(|_| "Session not found".to_string())?;

        let (provider, model, system_prompt, tools_mode) = conn.query_row(
            "SELECT provider, model, system_prompt, tools_mode FROM agents WHERE id = ?1",
            params![agent_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
        )
        .map_err(|_| "Agent not found".to_string())?;

        // Load provider settings (e.g. provider.google.api_key, provider.azure_openai.endpoint)
        let prefix = format!("provider.{}.", provider);
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings WHERE key LIKE ?1")
            .map_err(|e| e.to_string())?;
        let mut config = serde_json::Map::new();
        let rows = stmt
            .query_map(params![format!("{}%", prefix)], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, value))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (key, value) = row.map_err(|e| e.to_string())?;
            // Strip prefix: "provider.google.api_key" → "api_key"
            let field = key.strip_prefix(&prefix).unwrap_or(&key);
            // Strip JSON quotes if the value was stored as a JSON string
            let clean_value = value.trim_matches('"').to_string();
            config.insert(field.to_string(), serde_json::Value::String(clean_value));
        }

        (provider, model, system_prompt, tools_mode, config)
    };

    // 2. Get next sequence number
    let user_seq = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let max_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) FROM messages WHERE session_id = ?1",
                params![request.session_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        max_seq + 1
    };

    // 3. Persist user message
    let user_msg_id = Uuid::new_v4().to_string();
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (id, session_id, seq, role, content, created_at)
             VALUES (?1, ?2, ?3, 'user', ?4, ?5)",
            params![user_msg_id, request.session_id, user_seq, request.content, now],
        )
        .map_err(|e| format!("Failed to save user message: {e}"))?;
    }

    let user_message = Message {
        id: user_msg_id,
        session_id: request.session_id.clone(),
        seq: user_seq,
        role: "user".to_string(),
        content: request.content.clone(),
        model: None,
        provider: None,
        input_tokens: None,
        output_tokens: None,
        cost_usd: None,
        duration_ms: None,
        created_at: now.clone(),
    };

    // 4. Load full message history from SQLite (source of truth for sidecar)
    let history: Vec<serde_json::Value> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY seq ASC",
            )
            .map_err(|e| e.to_string())?;
        let result = stmt.query_map(params![request.session_id], |row| {
            Ok(serde_json::json!({
                "role": row.get::<_, String>(0)?,
                "content": row.get::<_, String>(1)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
        result
    };

    // 5. Record events
    record_event(&db, &request.session_id, "message.user", "ui.user",
        serde_json::json!({ "content": request.content }))?;
    record_event(&db, &request.session_id, "llm.request.started", "desktop.chat",
        serde_json::json!({ "model": model, "provider": provider }))?;

    // 6. Call sidecar for real LLM response
    let llm_start = std::time::Instant::now();
    let api_key = provider_config.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let base_url = provider_config.get("base_url")
        .or_else(|| provider_config.get("endpoint"))
        .and_then(|v| v.as_str()).unwrap_or("").to_string();

    // Build extra_config from remaining provider settings (deployment, api_version, model_name, etc.)
    let mut extra_config = serde_json::Map::new();
    for (k, v) in &provider_config {
        if k != "api_key" && k != "base_url" && k != "endpoint" {
            extra_config.insert(k.clone(), v.clone());
        }
    }

    let tools_enabled = tools_mode != "sandboxed";
    let mut chat_body = serde_json::json!({
        "message": request.content,
        "conversation_id": request.session_id,
        "provider": provider,
        "model": model,
        "system_prompt": system_prompt,
        "tools_enabled": tools_enabled,
        "history": history,
    });
    // Only include credentials if we have an API key or base_url
    if !api_key.is_empty() {
        chat_body["api_key"] = serde_json::Value::String(api_key);
    }
    if !base_url.is_empty() {
        chat_body["base_url"] = serde_json::Value::String(base_url);
    }
    if !extra_config.is_empty() {
        chat_body["extra_config"] = serde_json::Value::Object(extra_config);
    }

    let resp = sidecar.proxy_request("POST", "/chat", Some(chat_body)).await
        .map_err(|e| {
            let _ = record_event(&db, &request.session_id, "agent.error", "desktop.chat",
                serde_json::json!({ "error": format!("{e}"), "error_code": "SidecarRequestFailed", "severity": "error" }));
            format!("LLM call failed: {e}")
        })?;

    let duration_ms = llm_start.elapsed().as_millis() as i64;
    let content = resp.get("content").and_then(|v| v.as_str()).unwrap_or("(no response)").to_string();
    let usage = resp.get("usage");
    let input_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let output_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let response_model = resp.get("model").and_then(|v| v.as_str()).unwrap_or(&model).to_string();

    // 6b. Record tool call events (if any tools were used)
    if let Some(tool_calls) = resp.get("tool_calls").and_then(|v| v.as_array()) {
        for tc in tool_calls {
            let tool_name = tc.get("tool_name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let tool_input = tc.get("tool_input").cloned().unwrap_or(serde_json::json!({}));
            let tool_output = tc.get("tool_output").and_then(|v| v.as_str()).unwrap_or("");
            let tool_duration = tc.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tool_error = tc.get("error").and_then(|v| v.as_str());
            let tool_call_id = tc.get("tool_call_id").and_then(|v| v.as_str()).unwrap_or("");

            record_event(&db, &request.session_id, "tool.requested", "sidecar.chat",
                serde_json::json!({
                    "tool_call_id": tool_call_id,
                    "tool_name": tool_name,
                    "tool_input": tool_input,
                }))?;

            if let Some(err) = tool_error {
                record_event(&db, &request.session_id, "tool.error", "sidecar.chat",
                    serde_json::json!({
                        "tool_call_id": tool_call_id,
                        "tool_name": tool_name,
                        "error": err,
                        "duration_ms": tool_duration,
                    }))?;
            } else {
                record_event(&db, &request.session_id, "tool.completed", "sidecar.chat",
                    serde_json::json!({
                        "tool_call_id": tool_call_id,
                        "tool_name": tool_name,
                        "tool_output": tool_output,
                        "duration_ms": tool_duration,
                    }))?;
            }
        }
    }

    // 7. Persist assistant message
    let assistant_seq = user_seq + 1;
    let assistant_msg_id = Uuid::new_v4().to_string();
    let resp_now = now_iso();
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (id, session_id, seq, role, content, model, provider,
                                   input_tokens, output_tokens, duration_ms, created_at)
             VALUES (?1, ?2, ?3, 'assistant', ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                assistant_msg_id, request.session_id, assistant_seq,
                content, response_model, provider,
                input_tokens, output_tokens, duration_ms, resp_now,
            ],
        )
        .map_err(|e| format!("Failed to save assistant message: {e}"))?;

        conn.execute(
            "UPDATE sessions SET
                message_count = message_count + 2,
                total_input_tokens = total_input_tokens + ?1,
                total_output_tokens = total_output_tokens + ?2,
                updated_at = ?3
             WHERE id = ?4",
            params![input_tokens, output_tokens, resp_now, request.session_id],
        )
        .map_err(|e| format!("Failed to update session: {e}"))?;
    }

    let assistant_message = Message {
        id: assistant_msg_id,
        session_id: request.session_id.clone(),
        seq: assistant_seq,
        role: "assistant".to_string(),
        content: content.clone(),
        model: Some(response_model.clone()),
        provider: Some(provider.clone()),
        input_tokens: Some(input_tokens),
        output_tokens: Some(output_tokens),
        cost_usd: None,
        duration_ms: Some(duration_ms),
        created_at: resp_now,
    };

    // 8. Record completion events
    record_event(&db, &request.session_id, "llm.response.completed", "desktop.chat",
        serde_json::json!({
            "model": response_model, "provider": provider,
            "input_tokens": input_tokens, "output_tokens": output_tokens,
            "duration_ms": duration_ms,
        }))?;
    record_event(&db, &request.session_id, "message.assistant", "desktop.chat",
        serde_json::json!({ "content": content, "model": response_model }))?;

    Ok(SendMessageResponse { user_message, assistant_message })
}

// ============================================
// EVENT HELPER
// ============================================

fn record_event(
    db: &tauri::State<'_, Database>,
    session_id: &str,
    event_type: &str,
    source: &str,
    payload: serde_json::Value,
) -> Result<Event, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let event_id = Uuid::new_v4().to_string();
    let ts = now_iso();

    let next_seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(seq), 0) + 1 FROM events WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .unwrap_or(1);

    let cost_usd = payload.get("cost_usd").and_then(|v| v.as_f64());
    let payload_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());

    conn.execute(
        "INSERT INTO events (event_id, type, ts, session_id, source, seq, payload, cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![event_id, event_type, ts, session_id, source, next_seq, payload_str, cost_usd],
    )
    .map_err(|e| format!("Failed to record event: {e}"))?;

    conn.execute(
        "UPDATE sessions SET event_count = event_count + 1 WHERE id = ?1",
        params![session_id],
    ).ok();

    Ok(Event {
        event_id,
        event_type: event_type.to_string(),
        ts,
        session_id: session_id.to_string(),
        source: source.to_string(),
        seq: next_seq,
        payload,
        cost_usd,
    })
}

// ============================================
// MCP SERVER COMMANDS
// ============================================

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
pub fn list_mcp_servers(db: tauri::State<'_, Database>) -> Result<Vec<McpServer>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, transport, command, args, url, env, enabled, created_at, updated_at
             FROM mcp_servers ORDER BY name ASC",
        )
        .map_err(|e| e.to_string())?;

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
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(servers)
}

#[tauri::command]
pub fn add_mcp_server(
    db: tauri::State<'_, Database>,
    config: CreateMcpServerRequest,
) -> Result<McpServer, String> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let args_json = serde_json::to_string(&config.args).unwrap_or_else(|_| "[]".to_string());
    let env_json = serde_json::to_string(&config.env).unwrap_or_else(|_| "{}".to_string());

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO mcp_servers (id, name, transport, command, args, url, env, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9)",
        params![
            id, config.name, config.transport, config.command,
            args_json, config.url, env_json, now, now,
        ],
    )
    .map_err(|e| format!("Failed to add MCP server: {e}"))?;

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
) -> Result<McpServer, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
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
        .map_err(|e| format!("Failed to update MCP server: {e}"))?;

    if rows == 0 {
        return Err("MCP server not found".to_string());
    }

    // Re-read the updated record
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
    .map_err(|e| format!("MCP server not found: {e}"))
}

#[tauri::command]
pub fn remove_mcp_server(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute("DELETE FROM mcp_servers WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to remove MCP server: {e}"))?;
    if rows == 0 {
        return Err("MCP server not found".to_string());
    }
    Ok(())
}

// ============================================
// GLOBAL APPROVAL RULES CRUD
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRule {
    pub id: String,
    pub name: String,
    pub tool_pattern: String,
    pub action: String,
    pub priority: i64,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApprovalRuleRequest {
    pub name: String,
    pub tool_pattern: String,
    pub action: String,
    #[serde(default)]
    pub priority: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApprovalRuleRequest {
    pub name: Option<String>,
    pub tool_pattern: Option<String>,
    pub action: Option<String>,
    pub priority: Option<i64>,
    pub enabled: Option<bool>,
}

#[tauri::command]
pub fn list_approval_rules(db: tauri::State<'_, Database>) -> Result<Vec<ApprovalRule>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, tool_pattern, action, priority, enabled, created_at
             FROM approval_rules ORDER BY priority DESC, name ASC",
        )
        .map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map([], |row| {
            Ok(ApprovalRule {
                id: row.get(0)?,
                name: row.get(1)?,
                tool_pattern: row.get(2)?,
                action: row.get(3)?,
                priority: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rules)
}

#[tauri::command]
pub fn create_approval_rule(
    db: tauri::State<'_, Database>,
    rule: CreateApprovalRuleRequest,
) -> Result<ApprovalRule, String> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO approval_rules (id, name, tool_pattern, action, priority, enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        params![id, rule.name, rule.tool_pattern, rule.action, rule.priority, now],
    )
    .map_err(|e| format!("Failed to create approval rule: {e}"))?;

    Ok(ApprovalRule {
        id,
        name: rule.name,
        tool_pattern: rule.tool_pattern,
        action: rule.action,
        priority: rule.priority,
        enabled: true,
        created_at: now,
    })
}

#[tauri::command]
pub fn update_approval_rule(
    db: tauri::State<'_, Database>,
    id: String,
    updates: UpdateApprovalRuleRequest,
) -> Result<ApprovalRule, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut sets = Vec::new();
    let mut param_index = 1u32;
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref name) = updates.name {
        sets.push(format!("name = ?{param_index}"));
        values.push(Box::new(name.clone()));
        param_index += 1;
    }
    if let Some(ref tool_pattern) = updates.tool_pattern {
        sets.push(format!("tool_pattern = ?{param_index}"));
        values.push(Box::new(tool_pattern.clone()));
        param_index += 1;
    }
    if let Some(ref action) = updates.action {
        sets.push(format!("action = ?{param_index}"));
        values.push(Box::new(action.clone()));
        param_index += 1;
    }
    if let Some(priority) = updates.priority {
        sets.push(format!("priority = ?{param_index}"));
        values.push(Box::new(priority));
        param_index += 1;
    }
    if let Some(enabled) = updates.enabled {
        sets.push(format!("enabled = ?{param_index}"));
        values.push(Box::new(if enabled { 1i32 } else { 0 }));
        param_index += 1;
    }

    if sets.is_empty() {
        return Err("No fields to update".to_string());
    }

    let sql = format!(
        "UPDATE approval_rules SET {} WHERE id = ?{param_index}",
        sets.join(", ")
    );
    values.push(Box::new(id.clone()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let rows = conn
        .execute(&sql, param_refs.as_slice())
        .map_err(|e| format!("Failed to update approval rule: {e}"))?;

    if rows == 0 {
        return Err("Approval rule not found".to_string());
    }

    // Re-read the updated record
    conn.query_row(
        "SELECT id, name, tool_pattern, action, priority, enabled, created_at
         FROM approval_rules WHERE id = ?1",
        params![id],
        |row| {
            Ok(ApprovalRule {
                id: row.get(0)?,
                name: row.get(1)?,
                tool_pattern: row.get(2)?,
                action: row.get(3)?,
                priority: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("Approval rule not found: {e}"))
}

#[tauri::command]
pub fn delete_approval_rule(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute("DELETE FROM approval_rules WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete approval rule: {e}"))?;
    if rows == 0 {
        return Err("Approval rule not found".to_string());
    }
    Ok(())
}

// ============================================
// WORKFLOW TYPES (Node Editor)
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub graph_json: String,
    pub variables_json: String,
    pub agent_id: Option<String>,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agent_id: Option<String>,
    pub node_count: i64,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkflowRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_graph_json")]
    pub graph_json: String,
    #[serde(default = "default_variables_json")]
    pub variables_json: String,
    pub agent_id: Option<String>,
}

fn default_graph_json() -> String {
    r#"{"nodes":[],"edges":[],"viewport":{"x":0,"y":0,"zoom":1}}"#.to_string()
}
fn default_variables_json() -> String { "[]".to_string() }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub graph_json: Option<String>,
    pub variables_json: Option<String>,
    pub agent_id: Option<Option<String>>,
}

// ============================================
// WORKFLOW COMMANDS (Node Editor CRUD)
// ============================================

#[tauri::command]
pub fn list_workflows(db: tauri::State<'_, Database>) -> Result<Vec<WorkflowSummary>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, agent_id, graph_json, is_archived, created_at, updated_at
             FROM workflows WHERE is_archived = 0
             ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let workflows = stmt
        .query_map([], |row| {
            let graph: String = row.get(4)?;
            let node_count = serde_json::from_str::<serde_json::Value>(&graph)
                .ok()
                .and_then(|v| v.get("nodes")?.as_array().map(|a| a.len() as i64))
                .unwrap_or(0);
            Ok(WorkflowSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                agent_id: row.get(3)?,
                node_count,
                is_archived: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(workflows)
}

#[tauri::command]
pub fn get_workflow(db: tauri::State<'_, Database>, id: String) -> Result<Workflow, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, name, description, graph_json, variables_json, agent_id, is_archived, created_at, updated_at
         FROM workflows WHERE id = ?1",
        params![id],
        |row| {
            Ok(Workflow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                graph_json: row.get(3)?,
                variables_json: row.get(4)?,
                agent_id: row.get(5)?,
                is_archived: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .map_err(|e| format!("Workflow not found: {e}"))
}

#[tauri::command]
pub fn create_workflow(
    db: tauri::State<'_, Database>,
    workflow: CreateWorkflowRequest,
) -> Result<Workflow, String> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO workflows (id, name, description, graph_json, variables_json, agent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id, workflow.name, workflow.description, workflow.graph_json,
            workflow.variables_json, workflow.agent_id, now, now,
        ],
    )
    .map_err(|e| format!("Failed to create workflow: {e}"))?;

    Ok(Workflow {
        id,
        name: workflow.name,
        description: workflow.description,
        graph_json: workflow.graph_json,
        variables_json: workflow.variables_json,
        agent_id: workflow.agent_id,
        is_archived: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_workflow(
    db: tauri::State<'_, Database>,
    id: String,
    updates: UpdateWorkflowRequest,
) -> Result<Workflow, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
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
    if let Some(ref graph) = updates.graph_json {
        sets.push(format!("graph_json = ?{param_index}"));
        values.push(Box::new(graph.clone()));
        param_index += 1;
    }
    if let Some(ref vars) = updates.variables_json {
        sets.push(format!("variables_json = ?{param_index}"));
        values.push(Box::new(vars.clone()));
        param_index += 1;
    }
    if let Some(ref agent_id_opt) = updates.agent_id {
        sets.push(format!("agent_id = ?{param_index}"));
        values.push(Box::new(agent_id_opt.clone()));
        param_index += 1;
    }

    let sql = format!(
        "UPDATE workflows SET {} WHERE id = ?{param_index}",
        sets.join(", ")
    );
    values.push(Box::new(id.clone()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let rows = conn
        .execute(&sql, param_refs.as_slice())
        .map_err(|e| format!("Failed to update workflow: {e}"))?;

    if rows == 0 {
        return Err("Workflow not found".to_string());
    }

    drop(conn);
    get_workflow(db, id)
}

#[tauri::command]
pub fn delete_workflow(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = now_iso();
    let rows = conn
        .execute(
            "UPDATE workflows SET is_archived = 1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| format!("Failed to archive workflow: {e}"))?;

    if rows == 0 {
        return Err("Workflow not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn duplicate_workflow(db: tauri::State<'_, Database>, id: String) -> Result<Workflow, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let source = conn.query_row(
        "SELECT name, description, graph_json, variables_json, agent_id
         FROM workflows WHERE id = ?1",
        params![id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
        )),
    )
    .map_err(|e| format!("Workflow not found: {e}"))?;

    let new_id = Uuid::new_v4().to_string();
    let now = now_iso();
    let new_name = format!("{} (copy)", source.0);

    conn.execute(
        "INSERT INTO workflows (id, name, description, graph_json, variables_json, agent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![new_id, new_name, source.1, source.2, source.3, source.4, now, now],
    )
    .map_err(|e| format!("Failed to duplicate workflow: {e}"))?;

    Ok(Workflow {
        id: new_id,
        name: new_name,
        description: source.1,
        graph_json: source.2,
        variables_json: source.3,
        agent_id: source.4,
        is_archived: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

// ============================================
// SIMPLE COMMANDS (kept for testing)
// ============================================

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AI Studio.", name)
}

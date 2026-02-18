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
                     duration_ms = ?3, completed_at = ?4
                     WHERE id = ?5 AND status = 'running'",
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
                     completed_at = ?3
                     WHERE id = ?4 AND status = 'running'",
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

/// Record an event using a direct Database reference (for background tasks / tokio::spawn).
pub fn record_event_db(
    db: &Database,
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
// WORKFLOW VALIDATION (Phase 3B)
// ============================================

#[tauri::command]
pub fn validate_workflow(db: tauri::State<'_, Database>, id: String) -> Result<ValidationResult, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let graph_json: String = conn
        .query_row(
            "SELECT graph_json FROM workflows WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Workflow not found: {e}"))?;

    validate_graph_json(&graph_json)
}

/// Validate a workflow graph. Pure function — no DB needed.
pub fn validate_graph_json(graph_json: &str) -> Result<ValidationResult, String> {
    let graph: serde_json::Value = serde_json::from_str(graph_json)
        .map_err(|e| format!("Invalid graph JSON: {e}"))?;

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let nodes = graph.get("nodes").and_then(|v| v.as_array());
    let edges = graph.get("edges").and_then(|v| v.as_array());

    let nodes = match nodes {
        Some(n) => n,
        None => {
            errors.push("Graph has no nodes array".to_string());
            return Ok(ValidationResult { valid: false, errors, warnings });
        }
    };

    if nodes.is_empty() {
        errors.push("Workflow has no nodes".to_string());
        return Ok(ValidationResult { valid: false, errors, warnings });
    }

    let edges = edges.cloned().unwrap_or_default();

    // Collect node IDs and types
    let mut node_ids: Vec<String> = Vec::new();
    let mut node_types: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut has_input = false;
    let mut has_output = false;

    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let ntype = node.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if ntype == "input" { has_input = true; }
        if ntype == "output" { has_output = true; }
        node_ids.push(id.clone());
        node_types.insert(id, ntype);
    }

    if !has_input {
        errors.push("Workflow must have at least one Input node".to_string());
    }
    if !has_output {
        errors.push("Workflow must have at least one Output node".to_string());
    }

    // Build adjacency list for cycle detection
    let mut adj: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut in_degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut connected_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();

    for id in &node_ids {
        adj.entry(id.clone()).or_default();
        in_degree.entry(id.clone()).or_insert(0);
    }

    for edge in &edges {
        let source = edge.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let target = edge.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if !source.is_empty() && !target.is_empty() {
            adj.entry(source.clone()).or_default().push(target.clone());
            *in_degree.entry(target.clone()).or_insert(0) += 1;
            connected_nodes.insert(source);
            connected_nodes.insert(target);
        }
    }

    // Kahn's algorithm for cycle detection (also gives topological order)
    let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    for (id, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(id.clone());
        }
    }

    let mut visited_count = 0usize;
    while let Some(node) = queue.pop_front() {
        visited_count += 1;
        if let Some(neighbors) = adj.get(&node) {
            for n in neighbors {
                if let Some(d) = in_degree.get_mut(n) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(n.clone());
                    }
                }
            }
        }
    }

    if visited_count < node_ids.len() {
        errors.push("Workflow contains a cycle — execution would loop forever".to_string());
    }

    // Check for orphan nodes (not connected by any edge)
    for id in &node_ids {
        let ntype = node_types.get(id).map(|s| s.as_str()).unwrap_or("");
        // Input nodes with no outgoing edges and Output nodes with no incoming edges are errors
        // Nodes with no connections at all are warnings
        if !connected_nodes.contains(id) && nodes.len() > 1 {
            if ntype == "input" || ntype == "output" {
                warnings.push(format!("Node '{}' ({}) has no connections", id, ntype));
            } else {
                warnings.push(format!("Orphan node '{}' ({}) — not connected to any edge", id, ntype));
            }
        }
    }

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}

// ============================================
// WORKFLOW EXECUTION ENGINE (Phase 3B)
// ============================================

#[tauri::command]
pub async fn run_workflow(
    db: tauri::State<'_, Database>,
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    app: tauri::AppHandle,
    request: RunWorkflowRequest,
) -> Result<WorkflowRunResult, String> {
    eprintln!("[workflow] === RUN START === workflow_id={}, input_keys={:?}",
        request.workflow_id, request.inputs.keys().collect::<Vec<_>>());

    // 1. Load workflow
    let (workflow_name, graph_json, workflow_agent_id) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT name, graph_json, agent_id FROM workflows WHERE id = ?1 AND is_archived = 0",
            params![request.workflow_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?)),
        )
        .map_err(|e| {
            eprintln!("[workflow] ERROR: Workflow not found: {e}");
            format!("Workflow not found: {e}")
        })?
    };
    eprintln!("[workflow] Loaded '{}', agent_id={:?}", workflow_name, workflow_agent_id);

    // 2. Validate
    let validation = validate_graph_json(&graph_json)?;
    if !validation.valid {
        eprintln!("[workflow] ERROR: Validation failed: {}", validation.errors.join("; "));
        return Err(format!("Invalid workflow: {}", validation.errors.join("; ")));
    }

    // 3. Create a session for this workflow run
    // Sessions require a valid agent_id (FK constraint). Use workflow's agent or first available.
    let agent_id = match workflow_agent_id {
        Some(ref id) if !id.is_empty() => {
            eprintln!("[workflow] Using workflow agent_id: {}", id);
            id.clone()
        }
        _ => {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let id = conn.query_row(
                "SELECT id FROM agents WHERE is_archived = 0 ORDER BY created_at LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            ).map_err(|_| {
                eprintln!("[workflow] ERROR: No agents in database");
                "No agents available. Create an agent first before running workflows.".to_string()
            })?;
            eprintln!("[workflow] Using fallback agent_id: {}", id);
            id
        }
    };
    let session_id = Uuid::new_v4().to_string();
    let now = now_iso();
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
            params![session_id, agent_id, format!("Workflow: {}", workflow_name), now, now],
        )
        .map_err(|e| {
            eprintln!("[workflow] ERROR: Session creation failed: {e} (agent_id={})", agent_id);
            format!("Failed to create workflow session: {e}")
        })?;
    }
    eprintln!("[workflow] Created session {}", session_id);

    // 4. Load provider config (for LLM nodes)
    let all_settings = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings")
            .map_err(|e| e.to_string())?;
        let mut settings = std::collections::HashMap::<String, String>::new();
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            let (k, v) = row.map_err(|e| e.to_string())?;
            settings.insert(k, v);
        }
        settings
    };
    eprintln!("[workflow] Loaded {} settings", all_settings.len());

    // 5. Spawn background execution
    let db_clone = db.inner().clone();
    let sidecar_clone = sidecar.inner().clone();
    let session_id_clone = session_id.clone();
    let inputs = request.inputs.clone();

    let result_handle = tauri::async_runtime::spawn(async move {
        execute_workflow(
            &db_clone, &sidecar_clone, &app,
            &session_id_clone, &graph_json, &inputs, &all_settings,
        ).await
    });

    match result_handle.await {
        Ok(result) => {
            match &result {
                Ok(r) => eprintln!("[workflow] === RUN DONE === status={}, tokens={}, cost=${:.4}, duration={}ms",
                    r.status, r.total_tokens, r.total_cost_usd, r.duration_ms),
                Err(e) => eprintln!("[workflow] === RUN FAILED === {}", e),
            }
            result
        }
        Err(e) => {
            eprintln!(
                "[workflow.run] panic workflow_id={} session_id={} error={}",
                request.workflow_id, session_id, e
            );
            Err(format!("Workflow execution panicked: {e}"))
        }
    }
}

/// Template variable resolution: replaces `{{node_id.handle}}` and `{{input.name}}` patterns.
fn resolve_template(
    template: &str,
    node_outputs: &std::collections::HashMap<String, serde_json::Value>,
    inputs: &std::collections::HashMap<String, serde_json::Value>,
) -> String {
    let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        let key = caps[1].trim();
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        if parts.len() == 2 {
            let (source, field) = (parts[0], parts[1]);
            if source == "input" || source == "inputs" {
                if let Some(val) = inputs.get(field) {
                    return match val.as_str() {
                        Some(s) => s.to_string(),
                        None => val.to_string(),
                    };
                }
            }
            // Look up node output
            if let Some(val) = node_outputs.get(source) {
                if field == "output" || field == "result" {
                    return match val.as_str() {
                        Some(s) => s.to_string(),
                        None => val.to_string(),
                    };
                }
                // Try as JSON object field
                if let Some(obj) = val.as_object() {
                    if let Some(field_val) = obj.get(field) {
                        return match field_val.as_str() {
                            Some(s) => s.to_string(),
                            None => field_val.to_string(),
                        };
                    }
                }
                return match val.as_str() {
                    Some(s) => s.to_string(),
                    None => val.to_string(),
                };
            }
        }
        // Single-part reference (no dot): e.g. {{input}}, {{node_id}}
        if parts.len() == 1 {
            // Check node outputs by ID
            if let Some(val) = node_outputs.get(key) {
                return match val.as_str() {
                    Some(s) => s.to_string(),
                    None => val.to_string(),
                };
            }
            // "input" as shorthand: return first input value
            if (key == "input" || key == "inputs") && !inputs.is_empty() {
                let val = inputs.values().next().unwrap();
                return match val.as_str() {
                    Some(s) => s.to_string(),
                    None => val.to_string(),
                };
            }
        }
        // Unresolved — return placeholder as-is
        eprintln!("[workflow] WARN: Unresolved template var '{}' (node_outputs={:?}, inputs={:?})",
            key, node_outputs.keys().collect::<Vec<_>>(), inputs.keys().collect::<Vec<_>>());
        caps[0].to_string()
    }).to_string()
}

/// Emit a workflow event with full canonical envelope fields.
fn emit_workflow_event(
    app: &tauri::AppHandle,
    session_id: &str,
    event_type: &str,
    payload: serde_json::Value,
    seq: &std::sync::atomic::AtomicI64,
) {
    let _ = app.emit("agent_event", serde_json::json!({
        "event_id": Uuid::new_v4().to_string(),
        "type": event_type,
        "ts": chrono::Utc::now().to_rfc3339(),
        "session_id": session_id,
        "source": "desktop.workflow",
        "seq": seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        "payload": payload,
        "cost_usd": null,
    }));
}

/// Core workflow execution — DAG walker with sequential node execution.
async fn execute_workflow(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    app: &tauri::AppHandle,
    session_id: &str,
    graph_json: &str,
    inputs: &std::collections::HashMap<String, serde_json::Value>,
    all_settings: &std::collections::HashMap<String, String>,
) -> Result<WorkflowRunResult, String> {
    let start_time = std::time::Instant::now();
    let seq_counter = std::sync::atomic::AtomicI64::new(1);
    let graph: serde_json::Value = serde_json::from_str(graph_json)
        .map_err(|e| format!("Invalid graph JSON: {e}"))?;

    let nodes = graph.get("nodes").and_then(|v| v.as_array())
        .ok_or("No nodes in graph")?;
    let edges = graph.get("edges").and_then(|v| v.as_array())
        .cloned().unwrap_or_default();

    // Emit workflow.started
    let _ = record_event_db(db, session_id, "workflow.started", "desktop.workflow",
        serde_json::json!({ "node_count": nodes.len(), "edge_count": edges.len() }));
    emit_workflow_event(app, session_id, "workflow.started",
        serde_json::json!({ "node_count": nodes.len(), "edge_count": edges.len() }),
        &seq_counter);

    // Build adjacency + in-degree for topological sort
    let mut node_map: std::collections::HashMap<String, &serde_json::Value> = std::collections::HashMap::new();
    let mut adj: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut in_degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    // Map: target_node_id → vec of (source_node_id, source_handle, target_handle)
    let mut incoming_edges: std::collections::HashMap<String, Vec<(String, String, String)>> = std::collections::HashMap::new();
    // Map: (source_node_id, source_handle) → vec of target_node_ids (for router branch skipping)
    let mut outgoing_by_handle: std::collections::HashMap<(String, String), Vec<String>> = std::collections::HashMap::new();

    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        node_map.insert(id.clone(), node);
        adj.entry(id.clone()).or_default();
        in_degree.entry(id.clone()).or_insert(0);
    }

    for edge in &edges {
        let source = edge.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let target = edge.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let source_handle = edge.get("sourceHandle").and_then(|v| v.as_str()).unwrap_or("output").to_string();
        let target_handle = edge.get("targetHandle").and_then(|v| v.as_str()).unwrap_or("input").to_string();
        if !source.is_empty() && !target.is_empty() {
            adj.entry(source.clone()).or_default().push(target.clone());
            *in_degree.entry(target.clone()).or_insert(0) += 1;
            incoming_edges.entry(target.clone()).or_default().push((source.clone(), source_handle.clone(), target_handle));
            outgoing_by_handle.entry((source, source_handle)).or_default().push(target);
        }
    }

    // Kahn's topological sort
    let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    for (id, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(id.clone());
        }
    }
    let mut topo_order: Vec<String> = Vec::new();
    let mut temp_degree = in_degree.clone();
    while let Some(node_id) = queue.pop_front() {
        topo_order.push(node_id.clone());
        if let Some(neighbors) = adj.get(&node_id) {
            for n in neighbors {
                if let Some(d) = temp_degree.get_mut(n) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(n.clone());
                    }
                }
            }
        }
    }

    // Execute nodes in topological order
    eprintln!("[workflow] Topological order: {:?}", topo_order);
    let mut node_outputs: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    let mut workflow_outputs: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    let mut total_tokens: i64 = 0;
    let mut total_cost: f64 = 0.0;
    let mut skipped_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();

    for node_id in &topo_order {
        // Transitive skip: if ALL predecessors are skipped, skip this node too
        if !skipped_nodes.contains(node_id) {
            if let Some(preds) = incoming_edges.get(node_id) {
                if !preds.is_empty() && preds.iter().all(|(src, _, _)| skipped_nodes.contains(src)) {
                    skipped_nodes.insert(node_id.clone());
                }
            }
        }

        // Skip nodes downstream of non-selected router branches
        if skipped_nodes.contains(node_id) {
            let _ = record_event_db(db, session_id, "workflow.node.skipped", "desktop.workflow",
                serde_json::json!({ "node_id": node_id, "reason": "downstream of skipped branch" }));
            emit_workflow_event(app, session_id, "workflow.node.skipped",
                serde_json::json!({ "node_id": node_id }),
                &seq_counter);
            continue;
        }

        let node = match node_map.get(node_id) {
            Some(n) => *n,
            None => continue,
        };
        let node_type = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let node_data = node.get("data").unwrap_or(&serde_json::Value::Null);

        // Emit node.started
        let _ = record_event_db(db, session_id, "workflow.node.started", "desktop.workflow",
            serde_json::json!({ "node_id": node_id, "node_type": node_type }));
        emit_workflow_event(app, session_id, "workflow.node.started",
            serde_json::json!({ "node_id": node_id, "node_type": node_type }),
            &seq_counter);

        // Resolve input from incoming edges
        let incoming_value = if let Some(inc) = incoming_edges.get(node_id) {
            if inc.len() == 1 {
                node_outputs.get(&inc[0].0).cloned()
            } else {
                // Multiple inputs — build object keyed by target handle
                let mut obj = serde_json::Map::new();
                for (src_id, _src_handle, tgt_handle) in inc {
                    if let Some(val) = node_outputs.get(src_id) {
                        obj.insert(tgt_handle.clone(), val.clone());
                    }
                }
                if obj.is_empty() { None } else { Some(serde_json::Value::Object(obj)) }
            }
        } else {
            None
        };

        let node_start = std::time::Instant::now();
        let result = match node_type {
            "input" => execute_input_node(node_data, node_id, inputs),
            "output" => execute_output_node(node_data, node_id, &incoming_value, &mut workflow_outputs),
            "llm" => execute_llm_node(
                db, sidecar, session_id, node_data, node_id,
                &incoming_value, &node_outputs, inputs, all_settings,
            ).await,
            "transform" => execute_transform_node(node_data, node_id, &incoming_value, &node_outputs, inputs),
            "router" => execute_router_node(
                db, sidecar, session_id, node_data, node_id,
                &incoming_value, &node_outputs, inputs, all_settings,
                &adj, &outgoing_by_handle, &mut skipped_nodes,
            ).await,
            "tool" => execute_tool_node(
                db, app, sidecar, session_id, node_data, node_id, &incoming_value,
            ).await,
            "approval" => {
                execute_approval_node(
                    db, app, session_id, node_data, node_id, &incoming_value, &seq_counter,
                ).await
            }
            _ => {
                // Unknown/subworkflow — skip
                let _ = record_event_db(db, session_id, "workflow.node.skipped", "desktop.workflow",
                    serde_json::json!({ "node_id": node_id, "node_type": node_type, "reason": "unsupported type" }));
                emit_workflow_event(app, session_id, "workflow.node.skipped",
                    serde_json::json!({ "node_id": node_id, "node_type": node_type }),
                    &seq_counter);
                Ok(serde_json::Value::Null)
            }
        };
        let node_duration = node_start.elapsed().as_millis() as i64;

        match result {
            Ok(output) => {
                // Track tokens/cost from LLM nodes
                if let Some(usage) = output.as_object().and_then(|o| o.get("__usage")) {
                    let toks = usage.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                    let cost = usage.get("cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    total_tokens += toks;
                    total_cost += cost;
                }

                // Store output (strip __usage metadata before storing)
                let clean_output = if let Some(obj) = output.as_object() {
                    if obj.contains_key("__usage") {
                        obj.get("content").cloned()
                            .or_else(|| obj.get("result").cloned())
                            .unwrap_or(output.clone())
                    } else {
                        output.clone()
                    }
                } else {
                    output.clone()
                };
                node_outputs.insert(node_id.clone(), clean_output.clone());

                // Emit node.completed
                let output_preview = match clean_output.as_str() {
                    Some(s) => s[..s.len().min(200)].to_string(),
                    None => serde_json::to_string(&clean_output).unwrap_or_default()[..200.min(serde_json::to_string(&clean_output).unwrap_or_default().len())].to_string(),
                };
                let _ = record_event_db(db, session_id, "workflow.node.completed", "desktop.workflow",
                    serde_json::json!({
                        "node_id": node_id, "node_type": node_type,
                        "output_preview": output_preview, "duration_ms": node_duration,
                    }));
                emit_workflow_event(app, session_id, "workflow.node.completed",
                    serde_json::json!({
                        "node_id": node_id, "node_type": node_type,
                        "output_preview": output_preview, "duration_ms": node_duration,
                    }),
                    &seq_counter);
            }
            Err(err) => {
                eprintln!(
                    "[workflow.node.error] session_id={} node_id={} node_type={} error={}",
                    session_id, node_id, node_type, err
                );
                let _ = record_event_db(db, session_id, "workflow.node.error", "desktop.workflow",
                    serde_json::json!({
                        "node_id": node_id, "node_type": node_type,
                        "error": err, "duration_ms": node_duration,
                    }));
                emit_workflow_event(app, session_id, "workflow.node.error",
                    serde_json::json!({ "node_id": node_id, "error": &err }),
                    &seq_counter);

                // Emit workflow.failed
                let total_duration = start_time.elapsed().as_millis() as i64;
                let _ = record_event_db(db, session_id, "workflow.failed", "desktop.workflow",
                    serde_json::json!({
                        "node_id": node_id, "error": err,
                        "duration_ms": total_duration,
                    }));
                emit_workflow_event(app, session_id, "workflow.failed",
                    serde_json::json!({ "node_id": node_id, "error": &err }),
                    &seq_counter);

                return Ok(WorkflowRunResult {
                    session_id: session_id.to_string(),
                    status: "failed".to_string(),
                    outputs: workflow_outputs,
                    total_tokens,
                    total_cost_usd: total_cost,
                    duration_ms: total_duration,
                    node_count: topo_order.len(),
                    error: Some(err),
                });
            }
        }
    }

    // Workflow completed successfully
    let total_duration = start_time.elapsed().as_millis() as i64;
    let _ = record_event_db(db, session_id, "workflow.completed", "desktop.workflow",
        serde_json::json!({
            "duration_ms": total_duration, "total_tokens": total_tokens,
            "total_cost_usd": total_cost, "node_count": topo_order.len(),
        }));
    emit_workflow_event(app, session_id, "workflow.completed",
        serde_json::json!({
            "duration_ms": total_duration, "total_tokens": total_tokens,
            "total_cost_usd": total_cost,
        }),
        &seq_counter);

    Ok(WorkflowRunResult {
        session_id: session_id.to_string(),
        status: "completed".to_string(),
        outputs: workflow_outputs,
        total_tokens,
        total_cost_usd: total_cost,
        duration_ms: total_duration,
        node_count: topo_order.len(),
        error: None,
    })
}

// ============================================
// NODE EXECUTORS
// ============================================

fn execute_input_node(
    node_data: &serde_json::Value,
    node_id: &str,
    inputs: &std::collections::HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // Input node: reads from the workflow inputs map
    // Try keys in order: node_id, inputName, name, label, "input"
    let input_name = node_data
        .get("inputName")
        .and_then(|v| v.as_str())
        .or_else(|| node_data.get("name").and_then(|v| v.as_str()))
        .or_else(|| node_data.get("label").and_then(|v| v.as_str()))
        .unwrap_or(node_id);

    // Try all possible keys
    let try_keys = [node_id, input_name, "input"];
    for key in &try_keys {
        if let Some(val) = inputs.get(*key) {
            eprintln!("[workflow] Input node '{}': resolved via key '{}'", node_id, key);
            return Ok(val.clone());
        }
    }

    // Fallback: if only one input exists, use it regardless of key name
    if inputs.len() == 1 {
        let (key, val) = inputs.iter().next().unwrap();
        eprintln!("[workflow] Input node '{}': single-input fallback (key='{}')", node_id, key);
        return Ok(val.clone());
    }

    // Use default value if configured
    if let Some(default_val) = node_data.get("defaultValue").or_else(|| node_data.get("default")) {
        eprintln!("[workflow] Input node '{}': using default value", node_id);
        return Ok(default_val.clone());
    }

    let available: Vec<&String> = inputs.keys().collect();
    Err(format!(
        "No input provided for Input node '{}' (tried keys: {:?}, available: {:?})",
        node_id, try_keys, available
    ))
}

fn execute_output_node(
    _node_data: &serde_json::Value,
    node_id: &str,
    incoming: &Option<serde_json::Value>,
    workflow_outputs: &mut std::collections::HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let value = incoming.clone().unwrap_or(serde_json::Value::Null);
    workflow_outputs.insert(node_id.to_string(), value.clone());
    Ok(value)
}

async fn execute_llm_node(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    session_id: &str,
    node_data: &serde_json::Value,
    node_id: &str,
    incoming: &Option<serde_json::Value>,
    node_outputs: &std::collections::HashMap<String, serde_json::Value>,
    inputs: &std::collections::HashMap<String, serde_json::Value>,
    all_settings: &std::collections::HashMap<String, String>,
) -> Result<serde_json::Value, String> {
    let provider_name = node_data.get("provider").and_then(|v| v.as_str()).unwrap_or("ollama");
    let model = node_data.get("model").and_then(|v| v.as_str()).unwrap_or("");
    let temperature = node_data.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7);
    let system_prompt = node_data.get("systemPrompt").and_then(|v| v.as_str()).unwrap_or("");
    eprintln!("[workflow] LLM node '{}': provider={}, model={}", node_id, provider_name, model);

    // Build the user prompt from template + incoming data
    let prompt_template = node_data.get("prompt").and_then(|v| v.as_str()).unwrap_or("{{input}}");
    let prompt = if prompt_template.contains("{{") {
        resolve_template(prompt_template, node_outputs, inputs)
    } else if let Some(inc) = incoming {
        let inc_str = match inc.as_str() {
            Some(s) => s.to_string(),
            None => serde_json::to_string(inc).unwrap_or_default(),
        };
        format!("{}\n\n{}", prompt_template, inc_str)
    } else {
        prompt_template.to_string()
    };
    eprintln!("[workflow] LLM node '{}': prompt='{}' (template='{}')", node_id,
        &prompt[..prompt.len().min(200)], prompt_template);

    // Load provider config from settings
    let prefix = format!("provider.{}.", provider_name);
    let mut api_key = String::new();
    let mut base_url = String::new();
    let mut extra_config = serde_json::Map::new();
    for (k, v) in all_settings {
        if let Some(field) = k.strip_prefix(&prefix) {
            let clean_val = v.trim_matches('"').to_string();
            match field {
                "api_key" => api_key = clean_val,
                "base_url" | "endpoint" => base_url = clean_val,
                _ => { extra_config.insert(field.to_string(), serde_json::Value::String(clean_val)); }
            }
        }
    }

    eprintln!("[workflow] LLM node '{}': calling /chat/direct (api_key={}, base_url={})",
        node_id, if api_key.is_empty() { "none" } else { "set" }, if base_url.is_empty() { "none" } else { &base_url });

    // Build /chat/direct request
    let mut body = serde_json::json!({
        "messages": [{ "role": "user", "content": prompt }],
        "provider": provider_name,
        "model": model,
        "temperature": temperature,
    });
    if !system_prompt.is_empty() {
        body["system_prompt"] = serde_json::Value::String(system_prompt.to_string());
    }
    if !api_key.is_empty() {
        body["api_key"] = serde_json::Value::String(api_key);
    }
    if !base_url.is_empty() {
        body["base_url"] = serde_json::Value::String(base_url);
    }
    if !extra_config.is_empty() {
        body["extra_config"] = serde_json::Value::Object(extra_config);
    }

    // Record LLM request event
    let _ = record_event_db(db, session_id, "llm.request.started", "desktop.workflow",
        serde_json::json!({ "node_id": node_id, "model": model, "provider": provider_name }));

    let resp = sidecar.proxy_request("POST", "/chat/direct", Some(body)).await
        .map_err(|e| {
            eprintln!("[workflow] ERROR: LLM call failed for node '{}': {}", node_id, e);
            format!("LLM call failed for node '{}': {}", node_id, e)
        })?;

    let content = resp.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    eprintln!("[workflow] LLM node '{}': response OK, content_len={}", node_id, content.len());
    let usage = resp.get("usage");
    let input_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let output_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let resp_model = resp.get("model").and_then(|v| v.as_str()).unwrap_or(model).to_string();

    // Record LLM response event
    let _ = record_event_db(db, session_id, "llm.response.completed", "desktop.workflow",
        serde_json::json!({
            "node_id": node_id, "model": resp_model, "provider": provider_name,
            "input_tokens": input_tokens, "output_tokens": output_tokens,
        }));

    // Return content with usage metadata (stripped by caller)
    Ok(serde_json::json!({
        "content": content,
        "__usage": {
            "total_tokens": input_tokens + output_tokens,
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "cost_usd": 0.0,
        }
    }))
}

fn execute_transform_node(
    node_data: &serde_json::Value,
    _node_id: &str,
    incoming: &Option<serde_json::Value>,
    node_outputs: &std::collections::HashMap<String, serde_json::Value>,
    inputs: &std::collections::HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // Template mode only (first pass)
    let template = node_data.get("template").and_then(|v| v.as_str()).unwrap_or("{{input}}");

    // If template has variables, resolve them
    if template.contains("{{") {
        let result = resolve_template(template, node_outputs, inputs);
        return Ok(serde_json::Value::String(result));
    }

    // Pass-through if no template
    Ok(incoming.clone().unwrap_or(serde_json::Value::Null))
}

async fn execute_router_node(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    session_id: &str,
    node_data: &serde_json::Value,
    node_id: &str,
    incoming: &Option<serde_json::Value>,
    _node_outputs: &std::collections::HashMap<String, serde_json::Value>,
    _inputs: &std::collections::HashMap<String, serde_json::Value>,
    all_settings: &std::collections::HashMap<String, String>,
    _adj: &std::collections::HashMap<String, Vec<String>>,
    outgoing_by_handle: &std::collections::HashMap<(String, String), Vec<String>>,
    skipped_nodes: &mut std::collections::HashSet<String>,
) -> Result<serde_json::Value, String> {
    // LLM classification: ask a cheap model to classify input into one of the branches
    let branches = node_data.get("branches").and_then(|v| v.as_array());
    let branches = match branches {
        Some(b) if !b.is_empty() => b,
        _ => return Err(format!("Router node '{}' has no branches configured", node_id)),
    };

    let branch_names: Vec<String> = branches.iter()
        .filter_map(|b| {
            // Handle both string elements (UI sends string[]) and object elements
            if let Some(s) = b.as_str() {
                Some(s.to_string())
            } else {
                b.get("name").and_then(|v| v.as_str()).map(|s| s.to_string())
            }
        })
        .collect();

    if branch_names.is_empty() {
        return Err(format!("Router node '{}' has no valid branch names", node_id));
    }

    let incoming_text = incoming.as_ref().map(|v| match v.as_str() {
        Some(s) => s.to_string(),
        None => serde_json::to_string(v).unwrap_or_default(),
    }).unwrap_or_default();

    let classify_prompt = format!(
        "Classify the following input into exactly one of these categories: {}.\n\n\
         Input: {}\n\n\
         Respond with ONLY the category name, nothing else.",
        branch_names.join(", "),
        incoming_text,
    );

    // Use the router's configured provider or default
    let provider_name = node_data.get("provider").and_then(|v| v.as_str()).unwrap_or("ollama");
    let model = node_data.get("model").and_then(|v| v.as_str()).unwrap_or("");

    // Build LLM body using sidecar helper
    let mut body = serde_json::json!({
        "messages": [{ "role": "user", "content": classify_prompt }],
        "provider": provider_name,
        "model": model,
        "temperature": 0.0,
    });

    // Load provider config
    let prefix = format!("provider.{}.", provider_name);
    for (k, v) in all_settings {
        if let Some(field) = k.strip_prefix(&prefix) {
            let clean_val = v.trim_matches('"').to_string();
            match field {
                "api_key" => { body["api_key"] = serde_json::Value::String(clean_val); }
                "base_url" | "endpoint" => { body["base_url"] = serde_json::Value::String(clean_val); }
                _ => {}
            }
        }
    }

    let resp = sidecar.proxy_request("POST", "/chat/direct", Some(body)).await
        .map_err(|e| format!("Router LLM call failed: {}", e))?;

    let classification = resp.get("content").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();

    // Find matching branch (case-insensitive)
    let selected = branch_names.iter().find(|name| {
        name.eq_ignore_ascii_case(&classification)
    }).cloned().unwrap_or_else(|| branch_names[0].clone());

    // Mark downstream nodes of non-selected branches for skipping
    // Router output handles are named "branch-0", "branch-1", etc. (matching React Flow Handle ids)
    let selected_idx = branch_names.iter().position(|n| n == &selected);
    for (i, _branch_name) in branch_names.iter().enumerate() {
        if Some(i) == selected_idx {
            continue; // Don't skip the selected branch
        }
        let handle_name = format!("branch-{}", i);
        let key = (node_id.to_string(), handle_name);
        if let Some(targets) = outgoing_by_handle.get(&key) {
            for target in targets {
                eprintln!("[workflow] Router '{}': skipping downstream node '{}' (non-selected branch '{}')",
                    node_id, target, branch_names[i]);
                skipped_nodes.insert(target.clone());
            }
        }
    }

    let _ = record_event_db(db, session_id, "workflow.node.completed", "desktop.workflow",
        serde_json::json!({
            "node_id": node_id, "node_type": "router",
            "classification": &classification, "selected_branch": &selected,
        }));

    // Pass through incoming value to next node
    Ok(incoming.clone().unwrap_or(serde_json::Value::Null))
}

async fn execute_tool_node(
    db: &Database,
    app: &tauri::AppHandle,
    sidecar: &crate::sidecar::SidecarManager,
    session_id: &str,
    node_data: &serde_json::Value,
    node_id: &str,
    incoming: &Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    use tauri::Manager;

    let tool_name = node_data.get("toolName").and_then(|v| v.as_str()).unwrap_or("");
    if tool_name.is_empty() {
        return Err(format!("Tool node '{}' has no tool configured", node_id));
    }

    // Check approval setting
    let approval_mode = node_data.get("approval").and_then(|v| v.as_str()).unwrap_or("auto");
    if approval_mode == "deny" {
        return Err(format!("Tool node '{}' has approval set to 'deny' — execution blocked", node_id));
    }

    // Build tool input from node config + incoming data
    let tool_input = if let Some(configured_input) = node_data.get("toolInput") {
        configured_input.clone()
    } else if let Some(inc) = incoming {
        inc.clone()
    } else {
        serde_json::json!({})
    };

    // If approval="ask", request human approval before executing
    if approval_mode == "ask" {
        let data_preview = serde_json::to_string_pretty(&tool_input)
            .unwrap_or_default()[..500.min(serde_json::to_string_pretty(&tool_input).unwrap_or_default().len())]
            .to_string();

        let _ = record_event_db(db, session_id, "workflow.node.waiting", "desktop.workflow",
            serde_json::json!({ "node_id": node_id, "tool_name": tool_name }));

        let approval_id = Uuid::new_v4().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
        let approvals = app.state::<crate::sidecar::ApprovalManager>();
        approvals.register(approval_id.clone(), tx).await;

        let _ = app.emit("workflow_approval_requested", serde_json::json!({
            "id": approval_id,
            "nodeId": node_id,
            "sessionId": session_id,
            "message": format!("Approve tool execution: {} ?", tool_name),
            "dataPreview": data_preview,
        }));

        let approved = match tokio::time::timeout(
            std::time::Duration::from_secs(300), rx,
        ).await {
            Ok(Ok(v)) => v,
            Ok(Err(_)) => false,
            Err(_) => false,
        };
        approvals.remove(&approval_id).await;

        if !approved {
            return Err(format!("Tool execution denied by user for node '{}'", node_id));
        }
    }

    let body = serde_json::json!({
        "tool_name": tool_name,
        "tool_input": tool_input,
    });

    let resp = sidecar.proxy_request("POST", "/tools/execute", Some(body)).await
        .map_err(|e| format!("Tool execution failed for node '{}': {}", node_id, e))?;

    Ok(resp.get("result").cloned().unwrap_or(resp))
}

async fn execute_approval_node(
    db: &Database,
    app: &tauri::AppHandle,
    session_id: &str,
    node_data: &serde_json::Value,
    node_id: &str,
    incoming: &Option<serde_json::Value>,
    seq_counter: &std::sync::atomic::AtomicI64,
) -> Result<serde_json::Value, String> {
    use tauri::Manager;

    let message = node_data.get("message").and_then(|v| v.as_str())
        .unwrap_or("Approval required to continue workflow");

    let data_preview = incoming.as_ref().map(|v| match v.as_str() {
        Some(s) => s[..s.len().min(500)].to_string(),
        None => serde_json::to_string(v).unwrap_or_default()[..500.min(serde_json::to_string(v).unwrap_or_default().len())].to_string(),
    }).unwrap_or_default();

    // Emit waiting event
    let _ = record_event_db(db, session_id, "workflow.node.waiting", "desktop.workflow",
        serde_json::json!({ "node_id": node_id, "message": message }));
    emit_workflow_event(app, session_id, "workflow.node.waiting",
        serde_json::json!({ "node_id": node_id, "message": message }),
        seq_counter);

    // Create approval channel
    let approval_id = Uuid::new_v4().to_string();
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    let approvals = app.state::<crate::sidecar::ApprovalManager>();
    approvals.register(approval_id.clone(), tx).await;

    // Emit event to UI requesting approval
    let _ = app.emit("workflow_approval_requested", serde_json::json!({
        "id": approval_id,
        "nodeId": node_id,
        "sessionId": session_id,
        "message": message,
        "dataPreview": data_preview,
    }));

    // Wait with 5 minute timeout
    let approved = match tokio::time::timeout(
        std::time::Duration::from_secs(300), rx,
    ).await {
        Ok(Ok(v)) => v,
        Ok(Err(_)) => false, // channel dropped
        Err(_) => false, // timeout
    };

    // Clean up
    approvals.remove(&approval_id).await;

    if approved {
        Ok(incoming.clone().unwrap_or(serde_json::Value::Null))
    } else {
        Err(format!("Approval denied or timed out for node '{}'", node_id))
    }
}

// ============================================
// WORKFLOW EXECUTION TYPES (Phase 3B)
// ============================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunWorkflowRequest {
    pub workflow_id: String,
    pub inputs: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunResult {
    pub session_id: String,
    pub status: String,
    pub outputs: std::collections::HashMap<String, serde_json::Value>,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
    pub duration_ms: i64,
    pub node_count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// ============================================
// SIMPLE COMMANDS (kept for testing)
// ============================================

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AI Studio.", name)
}

// ============================================
// WORKFLOW TEMPLATES (Phase 3C)
// ============================================

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_count: usize,
}

const TEMPLATES: &[(&str, &str, &str, &str)] = &[
    ("code-review", "Code Review", "Analyze PR, classify by severity, output structured review",
        include_str!("../templates/code-review.json")),
    ("research", "Research Assistant", "Research a topic and produce a formatted report",
        include_str!("../templates/research.json")),
    ("data-pipeline", "Data Pipeline", "Extract structured data from raw input using LLM",
        include_str!("../templates/data-pipeline.json")),
    ("multi-model-compare", "Multi-Model Compare", "Send the same prompt to 3 models and compare outputs",
        include_str!("../templates/multi-model-compare.json")),
    ("safe-executor", "Safe Executor", "Plan a shell command with LLM, approve, then execute",
        include_str!("../templates/safe-executor.json")),
];

#[tauri::command]
pub fn list_templates() -> Vec<TemplateSummary> {
    TEMPLATES.iter().map(|(id, name, desc, json)| {
        let node_count = serde_json::from_str::<serde_json::Value>(json)
            .ok()
            .and_then(|v| v.get("nodes").and_then(|n| n.as_array()).map(|a| a.len()))
            .unwrap_or(0);
        TemplateSummary {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            node_count,
        }
    }).collect()
}

#[tauri::command]
pub fn load_template(template_id: String) -> Result<String, String> {
    TEMPLATES.iter()
        .find(|(id, _, _, _)| *id == template_id)
        .map(|(_, _, _, json)| json.to_string())
        .ok_or_else(|| format!("Template '{}' not found", template_id))
}

// ============================================
// TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // -- validate_graph_json tests --

    fn make_graph(nodes: &[(&str, &str)], edges: &[(&str, &str)]) -> String {
        let nodes_json: Vec<String> = nodes.iter().map(|(id, ntype)| {
            format!(r#"{{"id":"{}","type":"{}","position":{{"x":0,"y":0}},"data":{{}}}}"#, id, ntype)
        }).collect();
        let edges_json: Vec<String> = edges.iter().enumerate().map(|(i, (src, tgt))| {
            format!(r#"{{"id":"e{}","source":"{}","target":"{}"}}"#, i, src, tgt)
        }).collect();
        format!(r#"{{"nodes":[{}],"edges":[{}]}}"#, nodes_json.join(","), edges_json.join(","))
    }

    #[test]
    fn test_valid_simple_pipeline() {
        let graph = make_graph(
            &[("in1", "input"), ("llm1", "llm"), ("out1", "output")],
            &[("in1", "llm1"), ("llm1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid, "errors: {:?}", result.errors);
        assert!(result.warnings.is_empty(), "warnings: {:?}", result.warnings);
    }

    #[test]
    fn test_missing_input_node() {
        let graph = make_graph(
            &[("llm1", "llm"), ("out1", "output")],
            &[("llm1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Input node")));
    }

    #[test]
    fn test_missing_output_node() {
        let graph = make_graph(
            &[("in1", "input"), ("llm1", "llm")],
            &[("in1", "llm1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Output node")));
    }

    #[test]
    fn test_empty_workflow() {
        let graph = r#"{"nodes":[],"edges":[]}"#;
        let result = validate_graph_json(graph).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("no nodes")));
    }

    #[test]
    fn test_cycle_detection() {
        let graph = make_graph(
            &[("in1", "input"), ("a", "llm"), ("b", "transform"), ("out1", "output")],
            &[("in1", "a"), ("a", "b"), ("b", "a"), ("b", "out1")],
        );
        let result = validate_graph_json(graph.as_str()).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("cycle")));
    }

    #[test]
    fn test_orphan_node_warning() {
        let graph = make_graph(
            &[("in1", "input"), ("llm1", "llm"), ("orphan", "transform"), ("out1", "output")],
            &[("in1", "llm1"), ("llm1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid, "should be valid despite orphan");
        assert!(result.warnings.iter().any(|w| w.contains("Orphan") || w.contains("orphan")));
    }

    #[test]
    fn test_complex_dag_valid() {
        // Input → LLM → Router → (branch1: Tool → Output, branch2: Transform → Output)
        let graph = make_graph(
            &[
                ("in1", "input"),
                ("llm1", "llm"),
                ("router1", "router"),
                ("tool1", "tool"),
                ("transform1", "transform"),
                ("out1", "output"),
                ("out2", "output"),
            ],
            &[
                ("in1", "llm1"),
                ("llm1", "router1"),
                ("router1", "tool1"),
                ("router1", "transform1"),
                ("tool1", "out1"),
                ("transform1", "out2"),
            ],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid, "errors: {:?}", result.errors);
    }

    #[test]
    fn test_invalid_json() {
        let result = validate_graph_json("not json at all");
        assert!(result.is_err() || !result.unwrap().valid);
    }

    // -- resolve_template tests --

    #[test]
    fn test_resolve_input_variable() {
        let node_outputs = HashMap::new();
        let mut inputs = HashMap::new();
        inputs.insert("query".to_string(), serde_json::json!("What is AI?"));

        let result = resolve_template("User asks: {{input.query}}", &node_outputs, &inputs);
        assert_eq!(result, "User asks: What is AI?");
    }

    #[test]
    fn test_resolve_inputs_alias() {
        let node_outputs = HashMap::new();
        let mut inputs = HashMap::new();
        inputs.insert("text".to_string(), serde_json::json!("hello"));

        let result = resolve_template("{{inputs.text}}", &node_outputs, &inputs);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_resolve_node_output() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!("The answer is 42"));
        let inputs = HashMap::new();

        let result = resolve_template("LLM said: {{llm_1.output}}", &node_outputs, &inputs);
        assert_eq!(result, "LLM said: The answer is 42");
    }

    #[test]
    fn test_resolve_node_result_alias() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("tool_1".to_string(), serde_json::json!("file contents here"));
        let inputs = HashMap::new();

        let result = resolve_template("{{tool_1.result}}", &node_outputs, &inputs);
        assert_eq!(result, "file contents here");
    }

    #[test]
    fn test_resolve_json_field() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!({"answer": "42", "confidence": 0.95}));
        let inputs = HashMap::new();

        let result = resolve_template("Answer: {{llm_1.answer}}", &node_outputs, &inputs);
        assert_eq!(result, "Answer: 42");
    }

    #[test]
    fn test_resolve_unresolved_placeholder() {
        let node_outputs = HashMap::new();
        let inputs = HashMap::new();

        let result = resolve_template("Hello {{unknown.var}}", &node_outputs, &inputs);
        assert_eq!(result, "Hello {{unknown.var}}");
    }

    #[test]
    fn test_resolve_multiple_variables() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!("summary text"));
        let mut inputs = HashMap::new();
        inputs.insert("topic".to_string(), serde_json::json!("Rust"));

        let result = resolve_template(
            "Topic: {{input.topic}}, Summary: {{llm_1.output}}",
            &node_outputs, &inputs,
        );
        assert_eq!(result, "Topic: Rust, Summary: summary text");
    }

    #[test]
    fn test_resolve_no_templates() {
        let result = resolve_template("plain text no vars", &HashMap::new(), &HashMap::new());
        assert_eq!(result, "plain text no vars");
    }

    #[test]
    fn test_resolve_whitespace_in_braces() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), serde_json::json!("Amit"));

        let result = resolve_template("Hello {{ input.name }}", &HashMap::new(), &inputs);
        assert_eq!(result, "Hello Amit");
    }
}

use crate::db::{Database, now_iso};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
pub fn list_sessions(db: tauri::State<'_, Database>) -> Result<Vec<Session>, AppError> {
    let conn = db.conn.lock()?;
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
        )?;

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
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sessions)
}

#[tauri::command]
pub fn create_session(
    db: tauri::State<'_, Database>,
    agent_id: String,
    title: Option<String>,
) -> Result<Session, AppError> {
    let conn = db.conn.lock()?;
    let id = Uuid::new_v4().to_string();
    let now = now_iso();

    let (agent_name, agent_model): (String, String) = conn
        .query_row(
            "SELECT name, model FROM agents WHERE id = ?1 AND is_archived = 0",
            params![agent_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::NotFound("Agent not found".into()))?;

    let session_title = title.unwrap_or_else(|| format!("Chat with {agent_name}"));

    conn.execute(
        "INSERT INTO sessions (id, agent_id, title, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
        params![id, agent_id, session_title, now, now],
    )
    .map_err(|e| AppError::Db(format!("Failed to create session: {e}")))?;

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
) -> Result<Vec<Message>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, seq, role, content, model, provider,
                    input_tokens, output_tokens, cost_usd, duration_ms, created_at
             FROM messages WHERE session_id = ?1
             ORDER BY seq ASC",
        )?;

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
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

#[tauri::command]
pub fn branch_session(
    db: tauri::State<'_, Database>,
    session_id: String,
    seq: i64,
) -> Result<Session, AppError> {
    let mut conn = db.conn.lock()?;
    let tx = conn.transaction().map_err(|e| AppError::Db(format!("Failed to start transaction: {e}")))?;

    let (agent_id, parent_title): (String, String) = tx
        .query_row(
            "SELECT agent_id, title FROM sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::NotFound("Parent session not found".into()))?;

    let (agent_name, agent_model): (String, String) = tx
        .query_row(
            "SELECT name, model FROM agents WHERE id = ?1",
            params![agent_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::NotFound("Agent not found".into()))?;

    let new_id = Uuid::new_v4().to_string();
    let now = now_iso();
    let base_title = parent_title.strip_prefix("Branch of ").unwrap_or(&parent_title);
    let branch_title = format!("Branch of {base_title}");

    tx.execute(
        "INSERT INTO sessions (id, agent_id, title, status, parent_session_id, branch_from_seq, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6, ?7)",
        params![new_id, agent_id, branch_title, session_id, seq, now, now],
    )
    .map_err(|e| AppError::Db(format!("Failed to create branch session: {e}")))?;

    let mut stmt = tx
        .prepare(
            "SELECT seq, role, content, model, provider, input_tokens, output_tokens,
                    cost_usd, duration_ms, created_at
             FROM messages WHERE session_id = ?1 AND seq <= ?2
             ORDER BY seq ASC",
        )?;

    let rows: Vec<(i64, String, String, Option<String>, Option<String>,
                    Option<i64>, Option<i64>, Option<f64>, Option<i64>, String)> = stmt
        .query_map(params![session_id, seq], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
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
        .map_err(|e| AppError::Db(format!("Failed to copy message: {e}")))?;
        total_in += in_tok.unwrap_or(0);
        total_out += out_tok.unwrap_or(0);
        total_cost += cost.unwrap_or(0.0);
    }

    tx.execute(
        "UPDATE sessions SET message_count = ?1, total_input_tokens = ?2,
                total_output_tokens = ?3, total_cost_usd = ?4 WHERE id = ?5",
        params![msg_count, total_in, total_out, total_cost, new_id],
    )
    .map_err(|e| AppError::Db(format!("Failed to update session counters: {e}")))?;

    tx.commit().map_err(|e| AppError::Db(format!("Failed to commit branch: {e}")))?;

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
pub fn delete_session(db: tauri::State<'_, Database>, id: String) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    let rows = conn
        .execute("DELETE FROM sessions WHERE id = ?1", params![id])
        .map_err(|e| AppError::Db(format!("Failed to delete session: {e}")))?;
    if rows == 0 {
        return Err(AppError::NotFound("Session not found".into()));
    }
    Ok(())
}

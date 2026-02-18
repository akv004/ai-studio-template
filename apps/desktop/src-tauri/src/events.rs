use crate::db::{Database, now_iso};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Unified event recording — works with both `&Database` (background tasks)
/// and `&tauri::State<Database>` (via `.inner()`).
pub fn record_event(
    db: &Database,
    session_id: &str,
    event_type: &str,
    source: &str,
    payload: serde_json::Value,
) -> Result<Event, AppError> {
    let conn = db.conn.lock()?;
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
    .map_err(|e| AppError::Db(format!("Failed to record event: {e}")))?;

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

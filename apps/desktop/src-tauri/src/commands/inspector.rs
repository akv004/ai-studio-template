use crate::db::Database;
use crate::error::AppError;
use crate::events::Event;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub total_events: i64,
    pub total_messages: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
    pub models_used: Vec<String>,
    pub total_routing_decisions: i64,
    pub total_estimated_savings: f64,
    pub model_usage: Vec<ModelUsageStat>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsageStat {
    pub model: String,
    pub calls: i64,
    pub cost: f64,
}

#[tauri::command]
pub fn get_session_events(
    db: tauri::State<'_, Database>,
    session_id: String,
) -> Result<Vec<Event>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT event_id, type, ts, session_id, source, seq, payload, cost_usd
             FROM events WHERE session_id = ?1
             ORDER BY seq ASC",
        )?;

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
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(events)
}

#[tauri::command]
pub fn get_session_stats(
    db: tauri::State<'_, Database>,
    session_id: String,
) -> Result<SessionStats, AppError> {
    let conn = db.conn.lock()?;

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
        )?;

    let mut stmt = conn
        .prepare("SELECT DISTINCT model FROM messages WHERE session_id = ?1 AND model IS NOT NULL")?;
    let models: Vec<String> = stmt
        .query_map(params![session_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let total_routing_decisions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE session_id = ?1 AND type = 'llm.routed'",
            params![session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_estimated_savings: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(
                CAST(json_extract(payload, '$.estimated_savings') AS REAL)
            ), 0.0) FROM events WHERE session_id = ?1 AND type = 'llm.routed'",
            params![session_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let mut model_stmt = conn
        .prepare(
            "SELECT model, COUNT(*) as calls, COALESCE(SUM(cost_usd), 0.0) as cost
             FROM messages WHERE session_id = ?1 AND model IS NOT NULL
             GROUP BY model ORDER BY calls DESC"
        )?;
    let model_usage: Vec<ModelUsageStat> = model_stmt
        .query_map(params![session_id], |row| {
            Ok(ModelUsageStat {
                model: row.get(0)?,
                calls: row.get(1)?,
                cost: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SessionStats {
        total_events,
        total_messages,
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_cost_usd: total_cost,
        models_used: models,
        total_routing_decisions,
        total_estimated_savings,
        model_usage,
    })
}

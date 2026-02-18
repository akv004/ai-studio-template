use crate::db::Database;
use crate::error::AppError;
use chrono::Datelike;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetStatus {
    pub monthly_limit: Option<f64>,
    pub used: f64,
    pub remaining: f64,
    pub percentage: f64,
    pub exhausted_behavior: String,
    pub breakdown: Vec<ProviderCost>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCost {
    pub provider: String,
    pub cost: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBudgetRequest {
    pub monthly_limit: Option<f64>,
    pub exhausted_behavior: Option<String>,
}

#[tauri::command]
pub fn get_budget_status(db: tauri::State<'_, Database>) -> Result<BudgetStatus, AppError> {
    let conn = db.conn.lock()?;
    let now = chrono::Utc::now();
    let month_start = format!("{}-{:02}-01T00:00:00.000Z", now.year(), now.month());

    let monthly_limit: Option<f64> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'budget.monthly_limit'",
            [],
            |row| {
                let v: String = row.get(0)?;
                Ok(v.trim_matches('"').parse::<f64>().ok())
            },
        )
        .unwrap_or(None);

    let exhausted_behavior: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'budget.exhausted_behavior'",
            [],
            |row| {
                let v: String = row.get(0)?;
                Ok(v.trim_matches('"').to_string())
            },
        )
        .unwrap_or_else(|_| "none".to_string());

    let used: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(cost_usd), 0.0) FROM events
             WHERE type = 'llm.response.completed' AND ts >= ?1",
            params![month_start],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let mut stmt = conn
        .prepare(
            "SELECT json_extract(payload, '$.provider') as p, COALESCE(SUM(cost_usd), 0.0) as c
             FROM events
             WHERE type = 'llm.response.completed' AND ts >= ?1
             GROUP BY p
             ORDER BY c DESC",
        )
        ?;

    let breakdown: Vec<ProviderCost> = stmt
        .query_map(params![month_start], |row| {
            Ok(ProviderCost {
                provider: row.get::<_, String>(0).unwrap_or_else(|_| "unknown".to_string()),
                cost: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let limit_val = monthly_limit.unwrap_or(0.0);
    let remaining = if limit_val > 0.0 { (limit_val - used).max(0.0) } else { f64::MAX };
    let percentage = if limit_val > 0.0 { (used / limit_val * 100.0).min(100.0) } else { 0.0 };

    Ok(BudgetStatus {
        monthly_limit,
        used,
        remaining,
        percentage,
        exhausted_behavior,
        breakdown,
    })
}

#[tauri::command]
pub fn set_budget(
    db: tauri::State<'_, Database>,
    request: SetBudgetRequest,
) -> Result<(), AppError> {
    let conn = db.conn.lock()?;

    if let Some(limit) = request.monthly_limit {
        let value_str = serde_json::to_string(&limit)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('budget.monthly_limit', ?1)",
            params![value_str],
        )
        .map_err(|e| AppError::Db(format!("Failed to save budget limit: {e}")))?;
    }

    if let Some(ref behavior) = request.exhausted_behavior {
        let value_str = format!("\"{}\"", behavior);
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('budget.exhausted_behavior', ?1)",
            params![value_str],
        )
        .map_err(|e| AppError::Db(format!("Failed to save exhausted behavior: {e}")))?;
    }

    Ok(())
}

/// Get budget remaining percentage (0-100). Returns 100 if no budget set.
pub fn get_budget_remaining_pct(
    db: &Database,
    all_settings: &std::collections::HashMap<String, String>,
) -> f64 {
    let limit = all_settings
        .get("budget.monthly_limit")
        .and_then(|v| v.trim_matches('"').parse::<f64>().ok())
        .unwrap_or(0.0);

    if limit <= 0.0 {
        return 100.0;
    }

    let used = get_current_month_cost(db).unwrap_or(0.0);
    let remaining = (limit - used).max(0.0);
    (remaining / limit) * 100.0
}

pub fn get_current_month_cost(db: &Database) -> Result<f64, AppError> {
    let conn = db.conn.lock()?;
    let now = chrono::Utc::now();
    let month_start = format!("{}-{:02}-01T00:00:00.000Z", now.year(), now.month());

    let cost: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(cost_usd), 0.0) FROM events
             WHERE type = 'llm.response.completed' AND ts >= ?1",
            params![month_start],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    Ok(cost)
}

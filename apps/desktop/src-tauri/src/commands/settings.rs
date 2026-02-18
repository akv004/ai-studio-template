use crate::db::Database;
use crate::error::AppError;
use rusqlite::params;

#[tauri::command]
pub fn get_all_settings(db: tauri::State<'_, Database>) -> Result<serde_json::Value, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")?;

    let mut map = serde_json::Map::new();
    let rows = stmt
        .query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })?;

    for row in rows {
        let (key, value) = row?;
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
) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    let value_str = serde_json::to_string(&value)?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value_str],
    )
    .map_err(|e| AppError::Db(format!("Failed to save setting: {e}")))?;
    Ok(())
}

#[tauri::command]
pub fn wipe_database(db: tauri::State<'_, Database>) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
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
    .map_err(|e| AppError::Db(format!("Failed to wipe database: {e}")))?;
    println!("[db] Database wiped — all data deleted");
    Ok(())
}

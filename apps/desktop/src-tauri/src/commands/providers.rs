use crate::db::{Database, now_iso};
use crate::error::AppError;
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderKeyInfo {
    pub provider: String,
    pub has_key: bool,
    pub base_url: Option<String>,
    pub updated_at: String,
}

#[tauri::command]
pub fn list_provider_keys(db: tauri::State<'_, Database>) -> Result<Vec<ProviderKeyInfo>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn
        .prepare("SELECT provider, base_url, updated_at FROM provider_keys")?;

    let keys = stmt
        .query_map([], |row| {
            Ok(ProviderKeyInfo {
                provider: row.get(0)?,
                has_key: true,
                base_url: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(keys)
}

#[tauri::command]
pub fn set_provider_key(
    db: tauri::State<'_, Database>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    let now = now_iso();
    conn.execute(
        "INSERT OR REPLACE INTO provider_keys (provider, api_key, base_url, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![provider, api_key, base_url, now],
    )
    .map_err(|e| AppError::Db(format!("Failed to save provider key: {e}")))?;
    Ok(())
}

#[tauri::command]
pub fn delete_provider_key(db: tauri::State<'_, Database>, provider: String) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    conn.execute(
        "DELETE FROM provider_keys WHERE provider = ?1",
        params![provider],
    )
    .map_err(|e| AppError::Db(format!("Failed to delete provider key: {e}")))?;
    Ok(())
}

use crate::db::{Database, now_iso};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
pub fn list_approval_rules(db: tauri::State<'_, Database>) -> Result<Vec<ApprovalRule>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, tool_pattern, action, priority, enabled, created_at
             FROM approval_rules ORDER BY priority DESC, name ASC",
        )?;

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
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rules)
}

#[tauri::command]
pub fn create_approval_rule(
    db: tauri::State<'_, Database>,
    rule: CreateApprovalRuleRequest,
) -> Result<ApprovalRule, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();

    let conn = db.conn.lock()?;
    conn.execute(
        "INSERT INTO approval_rules (id, name, tool_pattern, action, priority, enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        params![id, rule.name, rule.tool_pattern, rule.action, rule.priority, now],
    )
    .map_err(|e| AppError::Db(format!("Failed to create approval rule: {e}")))?;

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
) -> Result<ApprovalRule, AppError> {
    let conn = db.conn.lock()?;

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
        return Err(AppError::Validation("No fields to update".to_string()));
    }

    let sql = format!(
        "UPDATE approval_rules SET {} WHERE id = ?{param_index}",
        sets.join(", ")
    );
    values.push(Box::new(id.clone()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let rows = conn
        .execute(&sql, param_refs.as_slice())
        .map_err(|e| AppError::Db(format!("Failed to update approval rule: {e}")))?;

    if rows == 0 {
        return Err(AppError::NotFound("Approval rule not found".to_string()));
    }

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
    .map_err(|e| AppError::NotFound(format!("Approval rule not found: {e}")))
}

#[tauri::command]
pub fn delete_approval_rule(db: tauri::State<'_, Database>, id: String) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    let rows = conn
        .execute("DELETE FROM approval_rules WHERE id = ?1", params![id])
        .map_err(|e| AppError::Db(format!("Failed to delete approval rule: {e}")))?;
    if rows == 0 {
        return Err(AppError::NotFound("Approval rule not found".to_string()));
    }
    Ok(())
}

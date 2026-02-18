use crate::db::{Database, now_iso};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[tauri::command]
pub fn list_workflows(db: tauri::State<'_, Database>) -> Result<Vec<WorkflowSummary>, AppError> {
    let conn = db.conn.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, agent_id, graph_json, is_archived, created_at, updated_at
             FROM workflows WHERE is_archived = 0
             ORDER BY updated_at DESC",
        )?;

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
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(workflows)
}

#[tauri::command]
pub fn get_workflow(db: tauri::State<'_, Database>, id: String) -> Result<Workflow, AppError> {
    let conn = db.conn.lock()?;
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
    .map_err(|_| AppError::NotFound(format!("Workflow '{id}' not found")))
}

#[tauri::command]
pub fn create_workflow(
    db: tauri::State<'_, Database>,
    workflow: CreateWorkflowRequest,
) -> Result<Workflow, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();

    let conn = db.conn.lock()?;
    conn.execute(
        "INSERT INTO workflows (id, name, description, graph_json, variables_json, agent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id, workflow.name, workflow.description, workflow.graph_json,
            workflow.variables_json, workflow.agent_id, now, now,
        ],
    )?;

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
) -> Result<Workflow, AppError> {
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
        .execute(&sql, param_refs.as_slice())?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Workflow '{id}' not found")));
    }

    drop(conn);
    get_workflow(db, id)
}

#[tauri::command]
pub fn delete_workflow(db: tauri::State<'_, Database>, id: String) -> Result<(), AppError> {
    let conn = db.conn.lock()?;
    let now = now_iso();
    let rows = conn
        .execute(
            "UPDATE workflows SET is_archived = 1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Workflow '{id}' not found")));
    }
    Ok(())
}

#[tauri::command]
pub fn duplicate_workflow(db: tauri::State<'_, Database>, id: String) -> Result<Workflow, AppError> {
    let conn = db.conn.lock()?;

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
    .map_err(|_| AppError::NotFound(format!("Workflow '{id}' not found")))?;

    let new_id = Uuid::new_v4().to_string();
    let now = now_iso();
    let new_name = format!("{} (copy)", source.0);

    conn.execute(
        "INSERT INTO workflows (id, name, description, graph_json, variables_json, agent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![new_id, new_name, source.1, source.2, source.3, source.4, now, now],
    )?;

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

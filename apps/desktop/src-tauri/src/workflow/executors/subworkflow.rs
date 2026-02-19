use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::events::record_event;

pub struct SubworkflowExecutor;

#[async_trait::async_trait]
impl NodeExecutor for SubworkflowExecutor {
    fn node_type(&self) -> &str { "subworkflow" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let workflow_id = node_data.get("workflowId")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if workflow_id.is_empty() {
            return Err("Subworkflow node has no workflowId configured".into());
        }

        // Circular reference detection
        if ctx.visited_workflows.contains(workflow_id) {
            return Err(format!(
                "Circular subworkflow reference detected: {} is already in the call chain ({:?})",
                workflow_id,
                ctx.visited_workflows
            ));
        }

        // Load sub-workflow from DB
        let graph_json = {
            let conn = ctx.db.conn.lock().map_err(|e| format!("DB lock: {e}"))?;
            conn.query_row(
                "SELECT graph_json FROM workflows WHERE id = ?1 AND is_archived = 0",
                rusqlite::params![workflow_id],
                |row| row.get::<_, String>(0),
            ).map_err(|e| format!("Subworkflow '{}' not found: {e}", workflow_id))?
        };

        // Build input map for sub-workflow
        let mut sub_inputs = std::collections::HashMap::new();
        if let Some(val) = incoming {
            sub_inputs.insert("input".to_string(), val.clone());
        }

        // Track visited workflows (extend the set)
        let mut visited = ctx.visited_workflows.clone();
        visited.insert(workflow_id.to_string());

        let _ = record_event(ctx.db, ctx.session_id, "workflow.node.subworkflow_start", "desktop.workflow",
            serde_json::json!({ "node_id": node_id, "sub_workflow_id": workflow_id }));

        // Execute sub-workflow recursively
        let result = crate::workflow::engine::execute_workflow_with_visited(
            ctx.db, ctx.sidecar, ctx.app,
            ctx.session_id, &graph_json, &sub_inputs, ctx.all_settings,
            &visited,
        ).await?;

        // Extract the sub-workflow output
        let output = if result.outputs.len() == 1 {
            result.outputs.into_values().next().unwrap_or(serde_json::Value::Null)
        } else if !result.outputs.is_empty() {
            serde_json::json!(result.outputs)
        } else {
            serde_json::Value::Null
        };

        Ok(NodeOutput::value(output))
    }
}

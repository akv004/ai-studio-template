use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::events::record_event;
use uuid::Uuid;
use tauri::{Emitter, Manager};

pub struct ToolExecutor;

#[async_trait::async_trait]
impl NodeExecutor for ToolExecutor {
    fn node_type(&self) -> &str { "tool" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let tool_name = node_data.get("toolName").and_then(|v| v.as_str()).unwrap_or("");
        if tool_name.is_empty() {
            return Err(format!("Tool node '{}' has no tool configured", node_id));
        }

        let approval_mode = node_data.get("approval").and_then(|v| v.as_str()).unwrap_or("auto");
        if approval_mode == "deny" {
            return Err(format!("Tool node '{}' has approval set to 'deny' — execution blocked", node_id));
        }

        let raw_input = if let Some(configured_input) = node_data.get("toolInput") {
            configured_input.clone()
        } else if let Some(inc) = incoming {
            inc.clone()
        } else {
            serde_json::json!({})
        };

        // Sidecar requires tool_input to be a JSON object.
        // If incoming is not an object, give a clear error pointing to the fix.
        let tool_input = if raw_input.is_object() {
            raw_input
        } else if raw_input.is_null() {
            serde_json::json!({})
        } else {
            return Err(format!(
                "Tool node '{}' received a non-object input (got {}). \
                 Tool nodes require JSON object input matching the tool's parameters. \
                 Add a Transform node before this Tool to reshape the data, e.g. \
                 template mode: {{\"command\": \"{{{{input}}}}\"}}",
                node_id,
                if raw_input.is_string() { "a string" }
                else if raw_input.is_number() { "a number" }
                else if raw_input.is_array() { "an array" }
                else { "a non-object value" }
            ));
        };

        if approval_mode == "ask" {
            let data_preview = serde_json::to_string_pretty(&tool_input)
                .unwrap_or_default()[..500.min(serde_json::to_string_pretty(&tool_input).unwrap_or_default().len())]
                .to_string();

            let _ = record_event(ctx.db, ctx.session_id, "workflow.node.waiting", "desktop.workflow",
                serde_json::json!({ "node_id": node_id, "tool_name": tool_name }));

            let approval_id = Uuid::new_v4().to_string();
            let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
            let approvals = ctx.app.state::<crate::sidecar::ApprovalManager>();
            approvals.register(approval_id.clone(), tx).await;

            let _ = ctx.app.emit("workflow_approval_requested", serde_json::json!({
                "id": approval_id,
                "nodeId": node_id,
                "sessionId": ctx.session_id,
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

        let resp = ctx.sidecar.proxy_request("POST", "/tools/execute", Some(body)).await
            .map_err(|e| format!("Tool execution failed for node '{}': {}", node_id, e))?;

        Ok(NodeOutput::value(resp.get("result").cloned().unwrap_or(resp)))
    }
}

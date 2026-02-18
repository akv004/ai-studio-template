use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::events::record_event;
use crate::workflow::engine::emit_workflow_event;
use uuid::Uuid;
use tauri::{Emitter, Manager};

pub struct ApprovalExecutor;

#[async_trait::async_trait]
impl NodeExecutor for ApprovalExecutor {
    fn node_type(&self) -> &str { "approval" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let message = node_data.get("message").and_then(|v| v.as_str())
            .unwrap_or("Approval required to continue workflow");

        let data_preview = incoming.as_ref().map(|v| match v.as_str() {
            Some(s) => s[..s.len().min(500)].to_string(),
            None => serde_json::to_string(v).unwrap_or_default()[..500.min(serde_json::to_string(v).unwrap_or_default().len())].to_string(),
        }).unwrap_or_default();

        let _ = record_event(ctx.db, ctx.session_id, "workflow.node.waiting", "desktop.workflow",
            serde_json::json!({ "node_id": node_id, "message": message }));
        emit_workflow_event(ctx.app, ctx.session_id, "workflow.node.waiting",
            serde_json::json!({ "node_id": node_id, "message": message }),
            ctx.seq_counter);

        let approval_id = Uuid::new_v4().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

        let approvals = ctx.app.state::<crate::sidecar::ApprovalManager>();
        approvals.register(approval_id.clone(), tx).await;

        let _ = ctx.app.emit("workflow_approval_requested", serde_json::json!({
            "id": approval_id,
            "nodeId": node_id,
            "sessionId": ctx.session_id,
            "message": message,
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

        if approved {
            Ok(NodeOutput::value(incoming.clone().unwrap_or(serde_json::Value::Null)))
        } else {
            Err(format!("Approval denied or timed out for node '{}'", node_id))
        }
    }
}

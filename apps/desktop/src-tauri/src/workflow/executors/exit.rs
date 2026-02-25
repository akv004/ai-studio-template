use super::{ExecutionContext, NodeExecutor, NodeOutput};

pub struct ExitExecutor;

/// Pass-through stub — analogous to AggregatorExecutor.
/// When paired with Loop, the Loop injects the pre-computed result via extra_outputs
/// and the Exit node is skipped. This executor only runs for standalone use.
#[async_trait::async_trait]
impl NodeExecutor for ExitExecutor {
    fn node_type(&self) -> &str { "exit" }

    async fn execute(
        &self,
        _ctx: &ExecutionContext<'_>,
        _node_id: &str,
        _node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        Ok(NodeOutput::value(
            incoming.clone().unwrap_or(serde_json::Value::Null),
        ))
    }
}

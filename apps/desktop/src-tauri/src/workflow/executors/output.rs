use super::{ExecutionContext, NodeExecutor, NodeOutput};

pub struct OutputExecutor;

#[async_trait::async_trait]
impl NodeExecutor for OutputExecutor {
    fn node_type(&self) -> &str { "output" }

    async fn execute(
        &self,
        _ctx: &ExecutionContext<'_>,
        _node_id: &str,
        _node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let value = incoming.clone().unwrap_or(serde_json::Value::Null);
        Ok(NodeOutput::value(value))
    }
}

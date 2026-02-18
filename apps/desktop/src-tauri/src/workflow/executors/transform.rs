use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;

pub struct TransformExecutor;

#[async_trait::async_trait]
impl NodeExecutor for TransformExecutor {
    fn node_type(&self) -> &str { "transform" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let template = node_data.get("template").and_then(|v| v.as_str()).unwrap_or("{{input}}");
        if template.contains("{{") {
            let result = resolve_template(template, ctx.node_outputs, ctx.inputs);
            return Ok(NodeOutput::value(serde_json::Value::String(result)));
        }
        Ok(NodeOutput::value(incoming.clone().unwrap_or(serde_json::Value::Null)))
    }
}

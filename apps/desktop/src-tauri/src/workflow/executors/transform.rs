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
        
        // Merge global inputs with incoming node inputs
        let mut local_inputs = ctx.inputs.clone();
        if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                for (k, v) in obj {
                    local_inputs.insert(k.clone(), v.clone());
                }
            } else {
                // If flattened (single 'input'), treat as 'input'
                local_inputs.insert("input".to_string(), inc.clone());
            }
        }

        if template.contains("{{") {
            let result = resolve_template(template, ctx.node_outputs, &local_inputs);
            return Ok(NodeOutput::value(serde_json::Value::String(result)));
        }
        Ok(NodeOutput::value(incoming.clone().unwrap_or(serde_json::Value::Null)))
    }
}

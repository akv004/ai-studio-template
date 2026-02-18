use super::{ExecutionContext, NodeExecutor, NodeOutput};

pub struct InputExecutor;

#[async_trait::async_trait]
impl NodeExecutor for InputExecutor {
    fn node_type(&self) -> &str { "input" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &serde_json::Value,
        _incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let input_name = node_data
            .get("inputName")
            .and_then(|v| v.as_str())
            .or_else(|| node_data.get("name").and_then(|v| v.as_str()))
            .or_else(|| node_data.get("label").and_then(|v| v.as_str()))
            .unwrap_or(node_id);

        let try_keys = [node_id, input_name, "input"];
        for key in &try_keys {
            if let Some(val) = ctx.inputs.get(*key) {
                eprintln!("[workflow] Input node '{}': resolved via key '{}'", node_id, key);
                return Ok(NodeOutput::value(val.clone()));
            }
        }

        if ctx.inputs.len() == 1 {
            let (key, val) = ctx.inputs.iter().next().unwrap();
            eprintln!("[workflow] Input node '{}': single-input fallback (key='{}')", node_id, key);
            return Ok(NodeOutput::value(val.clone()));
        }

        if let Some(default_val) = node_data.get("defaultValue").or_else(|| node_data.get("default")) {
            eprintln!("[workflow] Input node '{}': using default value", node_id);
            return Ok(NodeOutput::value(default_val.clone()));
        }

        let available: Vec<&String> = ctx.inputs.keys().collect();
        Err(format!(
            "No input provided for Input node '{}' (tried keys: {:?}, available: {:?})",
            node_id, try_keys, available
        ))
    }
}

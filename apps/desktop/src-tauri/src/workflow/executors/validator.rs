use super::{ExecutionContext, NodeExecutor, NodeOutput};

pub struct ValidatorExecutor;

#[async_trait::async_trait]
impl NodeExecutor for ValidatorExecutor {
    fn node_type(&self) -> &str { "validator" }

    async fn execute(
        &self,
        _ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        // Get data to validate from incoming edge
        let data = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("data").cloned().unwrap_or_else(|| inc.clone())
            } else {
                inc.clone()
            }
        } else {
            return Err("Validator: no data to validate".into());
        };

        let schema_str = node_data.get("schema").and_then(|v| v.as_str()).unwrap_or("{}");
        let fail_on_error = node_data.get("failOnError").and_then(|v| v.as_bool()).unwrap_or(false);

        // Parse schema
        let schema_value: serde_json::Value = serde_json::from_str(schema_str)
            .map_err(|e| format!("Invalid JSON Schema: {}", e))?;

        // Validate using iter_errors to collect all validation errors
        let validator = jsonschema::validator_for(&schema_value)
            .map_err(|e| format!("Cannot compile JSON Schema: {}", e))?;

        let error_strings: Vec<String> = validator.iter_errors(&data)
            .map(|e| e.to_string())
            .collect();

        if error_strings.is_empty() {
            Ok(NodeOutput::value(serde_json::json!({
                "valid": true,
                "data": data,
                "errors": [],
            })))
        } else if fail_on_error {
            Err(format!("Validation failed: {}", error_strings.join("; ")))
        } else {
            Ok(NodeOutput::value(serde_json::json!({
                "valid": false,
                "data": data,
                "errors": error_strings,
            })))
        }
    }
}

use super::{ExecutionContext, NodeExecutor, NodeOutput};
use std::collections::HashMap;

pub struct InputExecutor;

/// Pure logic for resolving the Input node's output value.
/// Extracted for testability — no dependencies on Tauri/DB/sidecar.
pub fn resolve_input_value(
    node_id: &str,
    node_data: &serde_json::Value,
    workflow_inputs: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let input_name = node_data
        .get("inputName")
        .and_then(|v| v.as_str())
        .or_else(|| node_data.get("name").and_then(|v| v.as_str()))
        .or_else(|| node_data.get("label").and_then(|v| v.as_str()).filter(|s| !s.is_empty()))
        .unwrap_or(node_id);

    let default_value = node_data.get("defaultValue")
        .and_then(|v| v.as_str())
        .or_else(|| node_data.get("default").and_then(|v| v.as_str()))
        .unwrap_or("");

    eprintln!("[workflow] Input node '{}': name='{}', defaultValue='{}', workflow_inputs={:?}",
        node_id, input_name, &default_value[..default_value.len().min(80)],
        workflow_inputs.keys().collect::<Vec<_>>());

    // Try resolving from workflow inputs by key
    let try_keys = [node_id, input_name, "input"];
    for key in &try_keys {
        if let Some(val) = workflow_inputs.get(*key) {
            let is_empty = val.as_str().map_or(false, |s| s.is_empty());
            if !is_empty {
                eprintln!("[workflow] Input node '{}': resolved via key '{}' → '{}'",
                    node_id, key, &val.to_string()[..val.to_string().len().min(80)]);
                return Ok(val.clone());
            }
            eprintln!("[workflow] Input node '{}': key '{}' found but EMPTY, skipping", node_id, key);
        }
    }

    // Single-input fallback: if only one workflow input exists and it's non-empty
    if workflow_inputs.len() == 1 {
        let (key, val) = workflow_inputs.iter().next().unwrap();
        let is_empty = val.as_str().map_or(false, |s| s.is_empty());
        if !is_empty {
            eprintln!("[workflow] Input node '{}': single-input fallback (key='{}') → '{}'",
                node_id, key, &val.to_string()[..val.to_string().len().min(80)]);
            return Ok(val.clone());
        }
        eprintln!("[workflow] Input node '{}': single-input fallback but value is empty", node_id);
    }

    // Fall back to defaultValue from node config
    if !default_value.is_empty() {
        eprintln!("[workflow] Input node '{}': using defaultValue → '{}'",
            node_id, &default_value[..default_value.len().min(80)]);
        return Ok(serde_json::json!(default_value));
    }

    let available: Vec<&String> = workflow_inputs.keys().collect();
    Err(format!(
        "No input provided for Input node '{}' (tried keys: {:?}, available: {:?}, defaultValue empty)",
        node_id, try_keys, available
    ))
}

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
        let val = resolve_input_value(node_id, node_data, ctx.inputs)?;
        Ok(NodeOutput::value(val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Scenario 1: User's OLD workflow — value typed in name field
    // Input node: data.name="2*8=?", no defaultValue
    // Run dialog sends: {"2*8=?": ""}
    // ============================================================
    #[test]
    fn test_old_workflow_value_in_name_field_empty_run_input() {
        let node_data = serde_json::json!({
            "name": "2*8=?",
            "dataType": "text"
        });
        let mut inputs = HashMap::new();
        inputs.insert("2*8=?".to_string(), serde_json::json!(""));

        // This should FAIL because:
        // - key "2*8=?" found but value is ""
        // - no defaultValue
        let result = resolve_input_value("input_1", &node_data, &inputs);
        assert!(result.is_err(), "Should error when value is empty and no defaultValue: {:?}", result);
    }

    // ============================================================
    // Scenario 2: User's NEW workflow — value in defaultValue field
    // Input node: data.name="query", data.defaultValue="2*8=?"
    // Run dialog sends: {"query": "2*8=?"} (pre-filled from defaultValue)
    // ============================================================
    #[test]
    fn test_new_workflow_value_in_default_value_prefilled() {
        let node_data = serde_json::json!({
            "name": "query",
            "defaultValue": "2*8=?",
            "dataType": "text"
        });
        let mut inputs = HashMap::new();
        inputs.insert("query".to_string(), serde_json::json!("2*8=?"));

        let result = resolve_input_value("input_1", &node_data, &inputs).unwrap();
        assert_eq!(result.as_str().unwrap(), "2*8=?");
    }

    // ============================================================
    // Scenario 3: User typed value in defaultValue but left run dialog empty
    // Input node: data.name="query", data.defaultValue="2*8=?"
    // Run dialog sends: {"query": ""} (user cleared it)
    // Should fall back to defaultValue
    // ============================================================
    #[test]
    fn test_empty_run_input_falls_back_to_default_value() {
        let node_data = serde_json::json!({
            "name": "query",
            "defaultValue": "2*8=?",
            "dataType": "text"
        });
        let mut inputs = HashMap::new();
        inputs.insert("query".to_string(), serde_json::json!(""));

        let result = resolve_input_value("input_1", &node_data, &inputs).unwrap();
        assert_eq!(result.as_str().unwrap(), "2*8=?");
    }

    // ============================================================
    // Scenario 4: No name set, value in defaultValue
    // Input node: data.defaultValue="hello world"
    // Run dialog sends: {"input": ""} (default key)
    // ============================================================
    #[test]
    fn test_no_name_uses_default_value() {
        let node_data = serde_json::json!({
            "defaultValue": "hello world",
            "dataType": "text"
        });
        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), serde_json::json!(""));

        let result = resolve_input_value("input_1", &node_data, &inputs).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    // ============================================================
    // Scenario 5: Run dialog provides a non-empty override
    // Input node: data.name="query", data.defaultValue="default question"
    // Run dialog sends: {"query": "override question"}
    // ============================================================
    #[test]
    fn test_run_input_overrides_default_value() {
        let node_data = serde_json::json!({
            "name": "query",
            "defaultValue": "default question",
            "dataType": "text"
        });
        let mut inputs = HashMap::new();
        inputs.insert("query".to_string(), serde_json::json!("override question"));

        let result = resolve_input_value("input_1", &node_data, &inputs).unwrap();
        assert_eq!(result.as_str().unwrap(), "override question");
    }

    // ============================================================
    // Scenario 6: Resolution by node_id
    // ============================================================
    #[test]
    fn test_resolve_by_node_id() {
        let node_data = serde_json::json!({"name": "x", "dataType": "text"});
        let mut inputs = HashMap::new();
        inputs.insert("input_1".to_string(), serde_json::json!("found by id"));

        let result = resolve_input_value("input_1", &node_data, &inputs).unwrap();
        assert_eq!(result.as_str().unwrap(), "found by id");
    }

    // ============================================================
    // Scenario 7: Single-input fallback with non-empty value
    // ============================================================
    #[test]
    fn test_single_input_fallback() {
        let node_data = serde_json::json!({"name": "x", "dataType": "text"});
        let mut inputs = HashMap::new();
        inputs.insert("some_other_key".to_string(), serde_json::json!("the value"));

        let result = resolve_input_value("input_1", &node_data, &inputs).unwrap();
        assert_eq!(result.as_str().unwrap(), "the value");
    }

    // ============================================================
    // Scenario 8: No inputs at all, no defaultValue → error
    // ============================================================
    #[test]
    fn test_no_inputs_no_default_errors() {
        let node_data = serde_json::json!({"name": "query", "dataType": "text"});
        let inputs = HashMap::new();

        let result = resolve_input_value("input_1", &node_data, &inputs);
        assert!(result.is_err());
    }
}

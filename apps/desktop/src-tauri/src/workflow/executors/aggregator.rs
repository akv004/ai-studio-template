use super::{ExecutionContext, NodeExecutor, NodeOutput};
use serde_json::Value;

pub struct AggregatorExecutor;

/// Standalone aggregation — when Aggregator is reached without being paired with Iterator.
/// When paired with Iterator, the Iterator injects the pre-computed result via extra_outputs
/// and the Aggregator is skipped. This executor only runs for standalone use.
fn aggregate_standalone(incoming: &Value, strategy: &str, separator: &str) -> Value {
    // If incoming is already an aggregated result (pre-computed), pass through
    if incoming.get("result").is_some() && incoming.get("count").is_some() {
        return incoming.clone();
    }

    // Try to interpret incoming as array
    if let Some(arr) = incoming.as_array() {
        return apply_strategy(arr, strategy, separator);
    }

    // Multiple inputs via named handles — collect into array
    if let Some(obj) = incoming.as_object() {
        let values: Vec<Value> = obj.values().cloned().collect();
        return apply_strategy(&values, strategy, separator);
    }

    // Single value — wrap in array
    apply_strategy(&[incoming.clone()], strategy, separator)
}

fn apply_strategy(items: &[Value], strategy: &str, separator: &str) -> Value {
    match strategy {
        "concat" => {
            let texts: Vec<String> = items.iter().map(|v| {
                match v.as_str() {
                    Some(s) => s.to_string(),
                    None => v.to_string(),
                }
            }).collect();
            serde_json::json!({
                "result": Value::String(texts.join(separator)),
                "count": items.len(),
            })
        }
        "merge" => {
            let mut merged = serde_json::Map::new();
            for item in items {
                if let Some(obj) = item.as_object() {
                    for (k, v) in obj {
                        merged.insert(k.clone(), v.clone());
                    }
                }
            }
            serde_json::json!({
                "result": Value::Object(merged),
                "count": items.len(),
            })
        }
        // "array" and default
        _ => {
            serde_json::json!({
                "result": items,
                "count": items.len(),
            })
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for AggregatorExecutor {
    fn node_type(&self) -> &str { "aggregator" }

    async fn execute(
        &self,
        _ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &Value,
        incoming: &Option<Value>,
    ) -> Result<NodeOutput, String> {
        let strategy = node_data.get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("array");
        let separator = node_data.get("separator")
            .and_then(|v| v.as_str())
            .unwrap_or("\n");

        let value = match incoming {
            Some(val) => aggregate_standalone(val, strategy, separator),
            None => serde_json::json!({"result": [], "count": 0}),
        };

        Ok(NodeOutput::value(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_precomputed_passthrough() {
        let incoming = serde_json::json!({"result": ["a", "b"], "count": 2});
        let output = aggregate_standalone(&incoming, "array", "\n");
        assert_eq!(output.get("count").unwrap().as_i64().unwrap(), 2);
        assert_eq!(output.get("result").unwrap().as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_aggregate_standalone_array() {
        let incoming = serde_json::json!(["item1", "item2", "item3"]);
        let output = aggregate_standalone(&incoming, "array", "\n");
        assert_eq!(output.get("count").unwrap().as_i64().unwrap(), 3);
    }

    #[test]
    fn test_aggregate_standalone_concat() {
        let incoming = serde_json::json!(["line1", "line2"]);
        let output = aggregate_standalone(&incoming, "concat", " | ");
        assert_eq!(output.get("result").unwrap().as_str().unwrap(), "line1 | line2");
    }

    #[test]
    fn test_aggregate_standalone_single_value() {
        let incoming = serde_json::json!("single");
        let output = aggregate_standalone(&incoming, "array", "\n");
        assert_eq!(output.get("count").unwrap().as_i64().unwrap(), 1);
    }

    #[test]
    fn test_aggregate_named_handles() {
        // Multiple inputs from different upstream nodes
        let incoming = serde_json::json!({"a": "hello", "b": "world"});
        let output = aggregate_standalone(&incoming, "concat", " ");
        let result = output.get("result").unwrap().as_str().unwrap();
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_aggregate_merge_strategy() {
        let incoming = serde_json::json!([
            {"a": 1, "b": 2},
            {"c": 3, "b": 99}
        ]);
        let output = aggregate_standalone(&incoming, "merge", "\n");
        let result = output.get("result").unwrap();
        assert_eq!(result.get("a").unwrap().as_i64().unwrap(), 1);
        assert_eq!(result.get("b").unwrap().as_i64().unwrap(), 99); // Last wins
        assert_eq!(result.get("c").unwrap().as_i64().unwrap(), 3);
    }

    #[test]
    fn test_aggregate_empty_input() {
        let incoming = serde_json::json!([]);
        let output = aggregate_standalone(&incoming, "array", "\n");
        assert_eq!(output.get("count").unwrap().as_i64().unwrap(), 0);
    }
}

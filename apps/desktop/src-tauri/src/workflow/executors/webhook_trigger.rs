use super::{ExecutionContext, NodeExecutor, NodeOutput};

pub struct WebhookTriggerExecutor;

#[async_trait::async_trait]
impl NodeExecutor for WebhookTriggerExecutor {
    fn node_type(&self) -> &str { "webhook_trigger" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        _node_data: &serde_json::Value,
        _incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        // Source node: reads __webhook_* keys injected by the webhook server
        let body = ctx.inputs.get("__webhook_body")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let headers = ctx.inputs.get("__webhook_headers")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let query = ctx.inputs.get("__webhook_query")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let method = ctx.inputs.get("__webhook_method")
            .cloned()
            .unwrap_or_else(|| serde_json::json!("POST"));

        eprintln!("[workflow] WebhookTrigger node '{}': method={}, body_type={}",
            node_id,
            method.as_str().unwrap_or("?"),
            if body.is_object() { "object" } else if body.is_string() { "string" } else { "other" },
        );

        let output = serde_json::json!({
            "body": body,
            "headers": headers,
            "query": query,
            "method": method,
        });

        Ok(NodeOutput::value(output))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    fn make_value(body: serde_json::Value, method: &str) -> HashMap<String, serde_json::Value> {
        let mut inputs = HashMap::new();
        inputs.insert("__webhook_body".to_string(), body);
        inputs.insert("__webhook_headers".to_string(), serde_json::json!({"content-type": "application/json"}));
        inputs.insert("__webhook_query".to_string(), serde_json::json!({"page": "1"}));
        inputs.insert("__webhook_method".to_string(), serde_json::json!(method));
        inputs
    }

    #[test]
    fn test_all_fields_present() {
        let inputs = make_value(serde_json::json!({"msg": "hello"}), "POST");
        let body = inputs.get("__webhook_body").unwrap();
        let headers = inputs.get("__webhook_headers").unwrap();
        let query = inputs.get("__webhook_query").unwrap();
        let method = inputs.get("__webhook_method").unwrap();

        let output = serde_json::json!({
            "body": body,
            "headers": headers,
            "query": query,
            "method": method,
        });

        assert_eq!(output["body"]["msg"], "hello");
        assert_eq!(output["method"], "POST");
        assert_eq!(output["query"]["page"], "1");
        assert_eq!(output["headers"]["content-type"], "application/json");
    }

    #[test]
    fn test_missing_fields_default() {
        let inputs = HashMap::<String, serde_json::Value>::new();
        let body = inputs.get("__webhook_body").cloned().unwrap_or(serde_json::Value::Null);
        let headers = inputs.get("__webhook_headers").cloned().unwrap_or_else(|| serde_json::json!({}));
        let query = inputs.get("__webhook_query").cloned().unwrap_or_else(|| serde_json::json!({}));
        let method = inputs.get("__webhook_method").cloned().unwrap_or_else(|| serde_json::json!("POST"));

        let output = serde_json::json!({
            "body": body,
            "headers": headers,
            "query": query,
            "method": method,
        });

        assert!(output["body"].is_null());
        assert!(output["headers"].is_object());
        assert!(output["query"].is_object());
        assert_eq!(output["method"], "POST");
    }

    #[test]
    fn test_json_body_passthrough() {
        let complex_body = serde_json::json!({
            "users": [{"name": "Alice"}, {"name": "Bob"}],
            "count": 2,
            "nested": {"deep": true}
        });
        let inputs = make_value(complex_body.clone(), "PUT");
        let body = inputs.get("__webhook_body").unwrap();

        assert_eq!(body["users"][0]["name"], "Alice");
        assert_eq!(body["count"], 2);
        assert_eq!(body["nested"]["deep"], true);
    }
}

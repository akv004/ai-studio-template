use super::{ExecutionContext, NodeExecutor, NodeOutput};

pub struct CronTriggerExecutor;

/// Extract cron trigger output from injected __cron_* inputs.
/// Pure function — testable without ExecutionContext.
pub fn build_cron_output(inputs: &std::collections::HashMap<String, serde_json::Value>) -> serde_json::Value {
    let timestamp = inputs.get("__cron_timestamp")
        .cloned()
        .unwrap_or_else(|| serde_json::json!(chrono::Utc::now().to_rfc3339()));
    let iteration = inputs.get("__cron_iteration")
        .cloned()
        .unwrap_or_else(|| serde_json::json!(0));
    let input = inputs.get("__cron_input")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let schedule = inputs.get("__cron_schedule")
        .cloned()
        .unwrap_or_else(|| serde_json::json!(""));

    serde_json::json!({
        "timestamp": timestamp,
        "iteration": iteration,
        "input": input,
        "schedule": schedule,
    })
}

#[async_trait::async_trait]
impl NodeExecutor for CronTriggerExecutor {
    fn node_type(&self) -> &str { "cron_trigger" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        _node_data: &serde_json::Value,
        _incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        let output = build_cron_output(ctx.inputs);

        eprintln!("[workflow] CronTrigger node '{}': schedule={}, iteration={}",
            node_id,
            output["schedule"].as_str().unwrap_or("?"),
            output["iteration"],
        );

        Ok(NodeOutput::value(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_cron_inputs(
        timestamp: &str,
        iteration: i64,
        input: serde_json::Value,
        schedule: &str,
    ) -> HashMap<String, serde_json::Value> {
        let mut inputs = HashMap::new();
        inputs.insert("__cron_timestamp".to_string(), serde_json::json!(timestamp));
        inputs.insert("__cron_iteration".to_string(), serde_json::json!(iteration));
        inputs.insert("__cron_input".to_string(), input);
        inputs.insert("__cron_schedule".to_string(), serde_json::json!(schedule));
        inputs
    }

    #[test]
    fn test_build_output_all_fields() {
        let inputs = make_cron_inputs(
            "2026-02-26T09:00:00Z", 5,
            serde_json::json!({"key": "val"}), "0 9 * * *",
        );
        let output = build_cron_output(&inputs);

        assert_eq!(output["timestamp"], "2026-02-26T09:00:00Z");
        assert_eq!(output["iteration"], 5);
        assert_eq!(output["input"]["key"], "val");
        assert_eq!(output["schedule"], "0 9 * * *");
    }

    #[test]
    fn test_build_output_missing_fields_defaults() {
        let inputs = HashMap::new();
        let output = build_cron_output(&inputs);

        assert_eq!(output["iteration"], 0);
        assert!(output["input"].is_object());
        assert!(output["input"].as_object().unwrap().is_empty());
        assert_eq!(output["schedule"], "");
        // timestamp defaults to current time (non-empty string)
        assert!(output["timestamp"].as_str().unwrap().len() > 10);
    }

    #[test]
    fn test_build_output_complex_static_input() {
        let complex = serde_json::json!({
            "reports": ["daily", "weekly"],
            "recipient": "admin@example.com",
            "options": {"format": "pdf"}
        });
        let inputs = make_cron_inputs("2026-02-26T09:00:00Z", 1, complex, "*/5 * * * *");
        let output = build_cron_output(&inputs);

        assert_eq!(output["input"]["reports"][0], "daily");
        assert_eq!(output["input"]["reports"][1], "weekly");
        assert_eq!(output["input"]["recipient"], "admin@example.com");
        assert_eq!(output["input"]["options"]["format"], "pdf");
    }

    #[test]
    fn test_build_output_iteration_sequence() {
        for i in 0..5 {
            let inputs = make_cron_inputs("2026-02-26T09:00:00Z", i, serde_json::json!({}), "0 * * * *");
            let output = build_cron_output(&inputs);
            assert_eq!(output["iteration"], i);
        }
    }

    #[test]
    fn test_build_output_partial_inputs() {
        // Only timestamp and schedule provided — iteration and input should default
        let mut inputs = HashMap::new();
        inputs.insert("__cron_timestamp".to_string(), serde_json::json!("2026-02-26T12:00:00Z"));
        inputs.insert("__cron_schedule".to_string(), serde_json::json!("0 12 * * *"));
        let output = build_cron_output(&inputs);

        assert_eq!(output["timestamp"], "2026-02-26T12:00:00Z");
        assert_eq!(output["schedule"], "0 12 * * *");
        assert_eq!(output["iteration"], 0);
        assert!(output["input"].is_object());
    }

    #[test]
    fn test_node_type() {
        let executor = CronTriggerExecutor;
        assert_eq!(executor.node_type(), "cron_trigger");
    }
}

use super::{ExecutionContext, NodeExecutor, NodeOutput};

pub struct CronTriggerExecutor;

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
        // Source node: reads __cron_* keys injected by the CronScheduler when it fires
        let timestamp = ctx.inputs.get("__cron_timestamp")
            .cloned()
            .unwrap_or_else(|| serde_json::json!(chrono::Utc::now().to_rfc3339()));
        let iteration = ctx.inputs.get("__cron_iteration")
            .cloned()
            .unwrap_or_else(|| serde_json::json!(0));
        let input = ctx.inputs.get("__cron_input")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let schedule = ctx.inputs.get("__cron_schedule")
            .cloned()
            .unwrap_or_else(|| serde_json::json!(""));

        eprintln!("[workflow] CronTrigger node '{}': schedule={}, iteration={}",
            node_id,
            schedule.as_str().unwrap_or("?"),
            iteration,
        );

        let output = serde_json::json!({
            "timestamp": timestamp,
            "iteration": iteration,
            "input": input,
            "schedule": schedule,
        });

        Ok(NodeOutput::value(output))
    }
}

#[cfg(test)]
mod tests {
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
    fn test_all_fields_present() {
        let inputs = make_cron_inputs(
            "2026-02-26T09:00:00Z",
            5,
            serde_json::json!({"key": "val"}),
            "0 9 * * *",
        );

        let output = serde_json::json!({
            "timestamp": inputs["__cron_timestamp"],
            "iteration": inputs["__cron_iteration"],
            "input": inputs["__cron_input"],
            "schedule": inputs["__cron_schedule"],
        });

        assert_eq!(output["timestamp"], "2026-02-26T09:00:00Z");
        assert_eq!(output["iteration"], 5);
        assert_eq!(output["input"]["key"], "val");
        assert_eq!(output["schedule"], "0 9 * * *");
    }

    #[test]
    fn test_missing_fields_default() {
        let inputs = HashMap::<String, serde_json::Value>::new();
        let timestamp = inputs.get("__cron_timestamp").cloned()
            .unwrap_or_else(|| serde_json::json!("default"));
        let iteration = inputs.get("__cron_iteration").cloned()
            .unwrap_or_else(|| serde_json::json!(0));
        let input = inputs.get("__cron_input").cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let schedule = inputs.get("__cron_schedule").cloned()
            .unwrap_or_else(|| serde_json::json!(""));

        let output = serde_json::json!({
            "timestamp": timestamp,
            "iteration": iteration,
            "input": input,
            "schedule": schedule,
        });

        assert_eq!(output["iteration"], 0);
        assert!(output["input"].is_object());
        assert_eq!(output["schedule"], "");
    }

    #[test]
    fn test_static_input_passthrough() {
        let complex_input = serde_json::json!({
            "reports": ["daily", "weekly"],
            "recipient": "admin@example.com",
            "options": {"format": "pdf"}
        });
        let inputs = make_cron_inputs("2026-02-26T09:00:00Z", 1, complex_input.clone(), "*/5 * * * *");

        let output_input = &inputs["__cron_input"];
        assert_eq!(output_input["reports"][0], "daily");
        assert_eq!(output_input["recipient"], "admin@example.com");
        assert_eq!(output_input["options"]["format"], "pdf");
    }

    #[test]
    fn test_iteration_counter() {
        for i in 0..5 {
            let inputs = make_cron_inputs("2026-02-26T09:00:00Z", i, serde_json::json!({}), "0 * * * *");
            assert_eq!(inputs["__cron_iteration"], i);
        }
    }
}

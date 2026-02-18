use crate::db::Database;
use crate::events::record_event;
use super::types::WorkflowRunResult;
use super::executors::{ExecutionContext, ExecutorRegistry};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicI64, Ordering};
use tauri::Emitter;
use uuid::Uuid;

/// Template variable resolution: replaces `{{node_id.handle}}` and `{{input.name}}` patterns.
pub fn resolve_template(
    template: &str,
    node_outputs: &HashMap<String, serde_json::Value>,
    inputs: &HashMap<String, serde_json::Value>,
) -> String {
    let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        let key = caps[1].trim();
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        if parts.len() == 2 {
            let (source, field) = (parts[0], parts[1]);
            if source == "input" || source == "inputs" {
                if let Some(val) = inputs.get(field) {
                    return match val.as_str() {
                        Some(s) => s.to_string(),
                        None => val.to_string(),
                    };
                }
            }
            if let Some(val) = node_outputs.get(source) {
                if field == "output" || field == "result" {
                    return match val.as_str() {
                        Some(s) => s.to_string(),
                        None => val.to_string(),
                    };
                }
                if let Some(obj) = val.as_object() {
                    if let Some(field_val) = obj.get(field) {
                        return match field_val.as_str() {
                            Some(s) => s.to_string(),
                            None => field_val.to_string(),
                        };
                    }
                }
                return match val.as_str() {
                    Some(s) => s.to_string(),
                    None => val.to_string(),
                };
            }
        }
        // Single-part reference (no dot)
        if parts.len() == 1 {
            if let Some(val) = node_outputs.get(key) {
                return match val.as_str() {
                    Some(s) => s.to_string(),
                    None => val.to_string(),
                };
            }
            // Check direct input match (e.g. {{topic}})
            if let Some(val) = inputs.get(key) {
                return match val.as_str() {
                    Some(s) => s.to_string(),
                    None => val.to_string(),
                };
            }
            if key == "input" || key == "inputs" {
                // Return entire object if it's an object, or the first value
                if let Some(val) = inputs.get("input") {
                     return match val.as_str() {
                        Some(s) => s.to_string(),
                        None => val.to_string(),
                    };
                }
                if !inputs.is_empty() {
                     let val = inputs.values().next().unwrap();
                     return match val.as_str() {
                        Some(s) => s.to_string(),
                        None => val.to_string(),
                    };
                }
            }
        }
        eprintln!("[workflow] WARN: Unresolved template var '{}' (node_outputs={:?}, inputs={:?})",
            key, node_outputs.keys().collect::<Vec<_>>(), inputs.keys().collect::<Vec<_>>());
        caps[0].to_string()
    }).to_string()
}

/// Emit a workflow event with full canonical envelope fields.
pub fn emit_workflow_event(
    app: &tauri::AppHandle,
    session_id: &str,
    event_type: &str,
    payload: serde_json::Value,
    seq: &AtomicI64,
) {
    let _ = app.emit("agent_event", serde_json::json!({
        "event_id": Uuid::new_v4().to_string(),
        "type": event_type,
        "ts": chrono::Utc::now().to_rfc3339(),
        "session_id": session_id,
        "source": "desktop.workflow",
        "seq": seq.fetch_add(1, Ordering::Relaxed),
        "payload": payload,
        "cost_usd": null,
    }));
}

/// Core workflow execution — DAG walker with sequential node execution.
pub async fn execute_workflow(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    app: &tauri::AppHandle,
    session_id: &str,
    graph_json: &str,
    inputs: &HashMap<String, serde_json::Value>,
    all_settings: &HashMap<String, String>,
) -> Result<WorkflowRunResult, String> {
    let start_time = std::time::Instant::now();
    let seq_counter = AtomicI64::new(1);
    let graph: serde_json::Value = serde_json::from_str(graph_json)
        .map_err(|e| format!("Invalid graph JSON: {e}"))?;

    let nodes = graph.get("nodes").and_then(|v| v.as_array())
        .ok_or("No nodes in graph")?;
    let edges = graph.get("edges").and_then(|v| v.as_array())
        .cloned().unwrap_or_default();

    // Emit workflow.started
    let _ = record_event(db, session_id, "workflow.started", "desktop.workflow",
        serde_json::json!({ "node_count": nodes.len(), "edge_count": edges.len() }));
    emit_workflow_event(app, session_id, "workflow.started",
        serde_json::json!({ "node_count": nodes.len(), "edge_count": edges.len() }),
        &seq_counter);

    // Build adjacency + in-degree for topological sort
    let mut node_map: HashMap<String, &serde_json::Value> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut incoming_edges: HashMap<String, Vec<(String, String, String)>> = HashMap::new();
    let mut outgoing_by_handle: HashMap<(String, String), Vec<String>> = HashMap::new();

    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        node_map.insert(id.clone(), node);
        adj.entry(id.clone()).or_default();
        in_degree.entry(id.clone()).or_insert(0);
    }

    for edge in &edges {
        let source = edge.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let target = edge.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let source_handle = edge.get("sourceHandle").and_then(|v| v.as_str()).unwrap_or("output").to_string();
        let target_handle = edge.get("targetHandle").and_then(|v| v.as_str()).unwrap_or("input").to_string();
        if !source.is_empty() && !target.is_empty() {
            adj.entry(source.clone()).or_default().push(target.clone());
            *in_degree.entry(target.clone()).or_insert(0) += 1;
            incoming_edges.entry(target.clone()).or_default().push((source.clone(), source_handle.clone(), target_handle));
            outgoing_by_handle.entry((source, source_handle)).or_default().push(target);
        }
    }

    // Kahn's topological sort
    let mut queue: VecDeque<String> = VecDeque::new();
    for (id, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(id.clone());
        }
    }
    let mut topo_order: Vec<String> = Vec::new();
    let mut temp_degree = in_degree.clone();
    while let Some(node_id) = queue.pop_front() {
        topo_order.push(node_id.clone());
        if let Some(neighbors) = adj.get(&node_id) {
            for n in neighbors {
                if let Some(d) = temp_degree.get_mut(n) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(n.clone());
                    }
                }
            }
        }
    }

    // Execute nodes in topological order
    eprintln!("[workflow] Topological order: {:?}", topo_order);
    let registry = ExecutorRegistry::new();
    let mut node_outputs: HashMap<String, serde_json::Value> = HashMap::new();
    let mut workflow_outputs: HashMap<String, serde_json::Value> = HashMap::new();
    let mut total_tokens: i64 = 0;
    let mut total_cost: f64 = 0.0;
    let mut skipped_nodes: HashSet<String> = HashSet::new();

    for node_id in &topo_order {
        // Transitive skip
        if !skipped_nodes.contains(node_id) {
            if let Some(preds) = incoming_edges.get(node_id) {
                if !preds.is_empty() && preds.iter().all(|(src, _, _)| skipped_nodes.contains(src)) {
                    skipped_nodes.insert(node_id.clone());
                }
            }
        }

        if skipped_nodes.contains(node_id) {
            let _ = record_event(db, session_id, "workflow.node.skipped", "desktop.workflow",
                serde_json::json!({ "node_id": node_id, "reason": "downstream of skipped branch" }));
            emit_workflow_event(app, session_id, "workflow.node.skipped",
                serde_json::json!({ "node_id": node_id }),
                &seq_counter);
            continue;
        }

        let node = match node_map.get(node_id) {
            Some(n) => *n,
            None => continue,
        };
        let node_type = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let node_data = node.get("data").unwrap_or(&serde_json::Value::Null);

        let _ = record_event(db, session_id, "workflow.node.started", "desktop.workflow",
            serde_json::json!({ "node_id": node_id, "node_type": node_type }));
        emit_workflow_event(app, session_id, "workflow.node.started",
            serde_json::json!({ "node_id": node_id, "node_type": node_type }),
            &seq_counter);

        // Resolve input from incoming edges
        let incoming_value = if let Some(inc) = incoming_edges.get(node_id) {
            // Optimization: If only 1 input and it's the default 'input' handle, flatten it.
            // Otherwise (multiple inputs OR 1 input with specific name), return as object.
            if inc.len() == 1 && inc[0].2 == "input" {
                node_outputs.get(&inc[0].0).cloned()
            } else {
                let mut obj = serde_json::Map::new();
                for (src_id, _src_handle, tgt_handle) in inc {
                    if let Some(val) = node_outputs.get(src_id) {
                        obj.insert(tgt_handle.clone(), val.clone());
                    }
                }
                if obj.is_empty() { None } else { Some(serde_json::Value::Object(obj)) }
            }
        } else {
            None
        };

        let node_start = std::time::Instant::now();
        let result = if let Some(executor) = registry.get(node_type) {
            let ctx = ExecutionContext {
                db, sidecar, app, session_id,
                all_settings, node_outputs: &node_outputs, inputs,
                outgoing_by_handle: &outgoing_by_handle,
                seq_counter: &seq_counter,
            };
            executor.execute(&ctx, node_id, node_data, &incoming_value).await
        } else {
            let _ = record_event(db, session_id, "workflow.node.skipped", "desktop.workflow",
                serde_json::json!({ "node_id": node_id, "node_type": node_type, "reason": "unsupported type" }));
            emit_workflow_event(app, session_id, "workflow.node.skipped",
                serde_json::json!({ "node_id": node_id, "node_type": node_type }),
                &seq_counter);
            Ok(super::executors::NodeOutput::value(serde_json::Value::Null))
        };
        let node_duration = node_start.elapsed().as_millis() as i64;

        match result {
            Ok(node_output) => {
                // Handle skip_nodes from router
                for skip_id in &node_output.skip_nodes {
                    skipped_nodes.insert(skip_id.clone());
                }

                let output = node_output.value;

                // Collect output-node values into workflow_outputs
                if node_type == "output" {
                    workflow_outputs.insert(node_id.clone(), output.clone());
                }

                if let Some(usage) = output.as_object().and_then(|o| o.get("__usage")) {
                    let toks = usage.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                    let cost = usage.get("cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    total_tokens += toks;
                    total_cost += cost;
                }

                let clean_output = if let Some(obj) = output.as_object() {
                    if obj.contains_key("__usage") {
                        obj.get("content").cloned()
                            .or_else(|| obj.get("result").cloned())
                            .unwrap_or(output.clone())
                    } else {
                        output.clone()
                    }
                } else {
                    output.clone()
                };
                node_outputs.insert(node_id.clone(), clean_output.clone());

                let output_preview = match clean_output.as_str() {
                    Some(s) => s[..s.len().min(200)].to_string(),
                    None => serde_json::to_string(&clean_output).unwrap_or_default()[..200.min(serde_json::to_string(&clean_output).unwrap_or_default().len())].to_string(),
                };
                let _ = record_event(db, session_id, "workflow.node.completed", "desktop.workflow",
                    serde_json::json!({
                        "node_id": node_id, "node_type": node_type,
                        "output_preview": output_preview, "duration_ms": node_duration,
                    }));
                emit_workflow_event(app, session_id, "workflow.node.completed",
                    serde_json::json!({
                        "node_id": node_id, "node_type": node_type,
                        "output_preview": output_preview, "duration_ms": node_duration,
                    }),
                    &seq_counter);
            }
            Err(err) => {
                eprintln!(
                    "[workflow.node.error] session_id={} node_id={} node_type={} error={}",
                    session_id, node_id, node_type, err
                );
                let _ = record_event(db, session_id, "workflow.node.error", "desktop.workflow",
                    serde_json::json!({
                        "node_id": node_id, "node_type": node_type,
                        "error": err, "duration_ms": node_duration,
                    }));
                emit_workflow_event(app, session_id, "workflow.node.error",
                    serde_json::json!({ "node_id": node_id, "error": &err }),
                    &seq_counter);

                let total_duration = start_time.elapsed().as_millis() as i64;
                let _ = record_event(db, session_id, "workflow.failed", "desktop.workflow",
                    serde_json::json!({
                        "node_id": node_id, "error": err,
                        "duration_ms": total_duration,
                    }));
                emit_workflow_event(app, session_id, "workflow.failed",
                    serde_json::json!({ "node_id": node_id, "error": &err }),
                    &seq_counter);

                return Ok(WorkflowRunResult {
                    session_id: session_id.to_string(),
                    status: "failed".to_string(),
                    outputs: workflow_outputs,
                    total_tokens,
                    total_cost_usd: total_cost,
                    duration_ms: total_duration,
                    node_count: topo_order.len(),
                    error: Some(err),
                });
            }
        }
    }

    let total_duration = start_time.elapsed().as_millis() as i64;
    let _ = record_event(db, session_id, "workflow.completed", "desktop.workflow",
        serde_json::json!({
            "duration_ms": total_duration, "total_tokens": total_tokens,
            "total_cost_usd": total_cost, "node_count": topo_order.len(),
        }));
    emit_workflow_event(app, session_id, "workflow.completed",
        serde_json::json!({
            "duration_ms": total_duration, "total_tokens": total_tokens,
            "total_cost_usd": total_cost,
        }),
        &seq_counter);

    Ok(WorkflowRunResult {
        session_id: session_id.to_string(),
        status: "completed".to_string(),
        outputs: workflow_outputs,
        total_tokens,
        total_cost_usd: total_cost,
        duration_ms: total_duration,
        node_count: topo_order.len(),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_input_variable() {
        let node_outputs = HashMap::new();
        let mut inputs = HashMap::new();
        inputs.insert("query".to_string(), serde_json::json!("What is AI?"));
        let result = resolve_template("User asks: {{input.query}}", &node_outputs, &inputs);
        assert_eq!(result, "User asks: What is AI?");
    }

    #[test]
    fn test_resolve_inputs_alias() {
        let node_outputs = HashMap::new();
        let mut inputs = HashMap::new();
        inputs.insert("text".to_string(), serde_json::json!("hello"));
        let result = resolve_template("{{inputs.text}}", &node_outputs, &inputs);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_resolve_node_output() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!("The answer is 42"));
        let inputs = HashMap::new();
        let result = resolve_template("LLM said: {{llm_1.output}}", &node_outputs, &inputs);
        assert_eq!(result, "LLM said: The answer is 42");
    }

    #[test]
    fn test_resolve_node_result_alias() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("tool_1".to_string(), serde_json::json!("file contents here"));
        let inputs = HashMap::new();
        let result = resolve_template("{{tool_1.result}}", &node_outputs, &inputs);
        assert_eq!(result, "file contents here");
    }

    #[test]
    fn test_resolve_json_field() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!({"answer": "42", "confidence": 0.95}));
        let inputs = HashMap::new();
        let result = resolve_template("Answer: {{llm_1.answer}}", &node_outputs, &inputs);
        assert_eq!(result, "Answer: 42");
    }

    #[test]
    fn test_resolve_unresolved_placeholder() {
        let node_outputs = HashMap::new();
        let inputs = HashMap::new();
        let result = resolve_template("Hello {{unknown.var}}", &node_outputs, &inputs);
        assert_eq!(result, "Hello {{unknown.var}}");
    }

    #[test]
    fn test_resolve_multiple_variables() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!("summary text"));
        let mut inputs = HashMap::new();
        inputs.insert("topic".to_string(), serde_json::json!("Rust"));
        let result = resolve_template(
            "Topic: {{input.topic}}, Summary: {{llm_1.output}}",
            &node_outputs, &inputs,
        );
        assert_eq!(result, "Topic: Rust, Summary: summary text");
    }

    #[test]
    fn test_resolve_no_templates() {
        let result = resolve_template("plain text no vars", &HashMap::new(), &HashMap::new());
        assert_eq!(result, "plain text no vars");
    }

    #[test]
    fn test_resolve_whitespace_in_braces() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), serde_json::json!("Amit"));
        let result = resolve_template("Hello {{ input.name }}", &HashMap::new(), &inputs);
        assert_eq!(result, "Hello Amit");
    }
}

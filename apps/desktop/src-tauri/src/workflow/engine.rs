use crate::db::Database;
use crate::events::record_event;
use super::types::WorkflowRunResult;
use super::executors::{ExecutionContext, ExecutorRegistry};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicI64, Ordering};
use tauri::Emitter;
use uuid::Uuid;

/// Truncate a string to at most `max_chars` characters (UTF-8 safe).
fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Select a specific output handle value from a node's stored output.
/// If the source handle is "output" (default), returns the whole value (backward compat).
/// If the output is structured (object) and contains the handle as a key, returns that field.
/// Otherwise falls back to the whole value (simple string outputs, Router passthrough, etc.).
fn resolve_source_handle(
    node_outputs: &HashMap<String, serde_json::Value>,
    src_id: &str,
    src_handle: &str,
) -> Option<serde_json::Value> {
    let val = node_outputs.get(src_id)?;
    // Default handle: return whole value
    if src_handle == "output" {
        return Some(val.clone());
    }
    // Structured output: select specific handle field
    if let Some(obj) = val.as_object() {
        if let Some(field_val) = obj.get(src_handle) {
            return Some(field_val.clone());
        }
        // Router backward compatibility: branch-* handles should unwrap the "value" field
        // from Router's {"selectedBranch": "...", "value": ...} wrapper so downstream
        // nodes receive the original incoming value, not the wrapper.
        if src_handle.starts_with("branch-") {
            if let Some(inner) = obj.get("value") {
                return Some(inner.clone());
            }
        }
    }
    // Fallback: whole value (simple strings, passthrough nodes)
    Some(val.clone())
}

/// Extract the primary text from a node output value.
/// Used for template resolution ({{node.output}}, {{node}}) and event preview strings.
/// Tries: string → object.response → object.content → object.result → JSON serialized.
pub fn extract_primary_text(val: &serde_json::Value) -> String {
    if let Some(s) = val.as_str() {
        return s.to_string();
    }
    if let Some(obj) = val.as_object() {
        for key in &["response", "content", "result", "value"] {
            if let Some(field) = obj.get(*key) {
                if let Some(s) = field.as_str() {
                    return s.to_string();
                }
                // If the field is a non-string value (object, array, number, bool),
                // serialize just that inner value — not the entire wrapper object.
                // This handles Router output where value is an object:
                // {"selectedBranch": "...", "value": {complex_object}} → serialize the inner object.
                if !field.is_null() {
                    return field.to_string();
                }
            }
        }
    }
    val.to_string()
}

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
                    return extract_primary_text(val);
                }
                if let Some(obj) = val.as_object() {
                    // Handle array index: "services[0]" → field="services", index=0
                    let (actual_field, index) = if field.contains('[') {
                        let parts: Vec<&str> = field.splitn(2, '[').collect();
                        let idx: Option<usize> = parts.get(1)
                            .and_then(|s| s.trim_end_matches(']').parse().ok());
                        (parts[0], idx)
                    } else {
                        (field, None)
                    };
                    if let Some(field_val) = obj.get(actual_field) {
                        let resolved = match (field_val, index) {
                            (serde_json::Value::Array(arr), Some(i)) => {
                                arr.get(i).cloned().unwrap_or(serde_json::Value::Null)
                            }
                            _ => field_val.clone(),
                        };
                        return match resolved.as_str() {
                            Some(s) => s.to_string(),
                            None => resolved.to_string(),
                        };
                    }
                }
                return extract_primary_text(val);
            }
        }
        // Single-part reference (no dot)
        if parts.len() == 1 {
            if let Some(val) = node_outputs.get(key) {
                return extract_primary_text(val);
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
    execute_workflow_ephemeral(db, sidecar, app, session_id, graph_json, inputs, all_settings, false).await
}

/// Core workflow execution with ephemeral flag (skips DB writes when true).
pub async fn execute_workflow_ephemeral(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    app: &tauri::AppHandle,
    session_id: &str,
    graph_json: &str,
    inputs: &HashMap<String, serde_json::Value>,
    all_settings: &HashMap<String, String>,
    ephemeral: bool,
) -> Result<WorkflowRunResult, String> {
    let visited = HashSet::new();
    let workflow_run_id = Uuid::new_v4().to_string();
    execute_workflow_with_visited(db, sidecar, app, session_id, graph_json, inputs, all_settings, &visited, &workflow_run_id, ephemeral).await
}

/// Execute workflow with circular reference tracking (for subworkflow support).
pub async fn execute_workflow_with_visited(
    db: &Database,
    sidecar: &crate::sidecar::SidecarManager,
    app: &tauri::AppHandle,
    session_id: &str,
    graph_json: &str,
    inputs: &HashMap<String, serde_json::Value>,
    all_settings: &HashMap<String, String>,
    visited_workflows: &HashSet<String>,
    workflow_run_id: &str,
    ephemeral: bool,
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
    if !ephemeral {
        let _ = record_event(db, session_id, "workflow.started", "desktop.workflow",
            serde_json::json!({ "node_count": nodes.len(), "edge_count": edges.len() }));
    }
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

    // Cycle detection: if topo sort didn't visit all nodes, a cycle exists
    if topo_order.len() < node_map.len() {
        let cyclic: Vec<_> = node_map.keys()
            .filter(|id| !topo_order.contains(id))
            .collect();
        return Err(format!("Workflow contains a cycle involving nodes: {:?}", cyclic));
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
        // Transitive skip: if ALL predecessors are skipped, skip this node too.
        // Exception: predecessors that are skipped but have pre-computed outputs
        // (via extra_outputs from Loop/Iterator) should NOT cause transitive skip.
        if !skipped_nodes.contains(node_id) {
            if let Some(preds) = incoming_edges.get(node_id) {
                if !preds.is_empty() && preds.iter().all(|(src, _, _)| {
                    skipped_nodes.contains(src) && !node_outputs.contains_key(src)
                }) {
                    skipped_nodes.insert(node_id.clone());
                }
            }
        }

        if skipped_nodes.contains(node_id) {
            if !ephemeral {
                let _ = record_event(db, session_id, "workflow.node.skipped", "desktop.workflow",
                    serde_json::json!({ "node_id": node_id, "reason": "downstream of skipped branch" }));
            }
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

        if !ephemeral {
            let _ = record_event(db, session_id, "workflow.node.started", "desktop.workflow",
                serde_json::json!({ "node_id": node_id, "node_type": node_type }));
        }
        emit_workflow_event(app, session_id, "workflow.node.started",
            serde_json::json!({ "node_id": node_id, "node_type": node_type }),
            &seq_counter);

        // Resolve input from incoming edges (using sourceHandle for handle-specific selection)
        let incoming_value = if let Some(inc) = incoming_edges.get(node_id) {
            eprintln!("[workflow] Engine: resolving incoming for node '{}' ({}) — {} edge(s): {:?}",
                node_id, node_type, inc.len(),
                inc.iter().map(|(s, sh, th)| format!("{}:{} → {}", s, sh, th)).collect::<Vec<_>>());
            // Single edge to default "input" handle: flatten to the resolved value
            if inc.len() == 1 && inc[0].2 == "input" {
                let val = resolve_source_handle(&node_outputs, &inc[0].0, &inc[0].1);
                eprintln!("[workflow] Engine: node '{}' single-edge flatten → {:?}",
                    node_id, val.as_ref().map(|v| truncate(&v.to_string(), 100).to_string()));
                val
            } else {
                // Multiple edges or named handles: build object keyed by target handle
                let mut obj = serde_json::Map::new();
                for (src_id, src_handle, tgt_handle) in inc {
                    if let Some(val) = resolve_source_handle(&node_outputs, src_id, src_handle) {
                        eprintln!("[workflow] Engine: node '{}' handle '{}' ← {}:{} = '{}'",
                            node_id, tgt_handle, src_id, src_handle,
                            truncate(&val.to_string(), 100));
                        obj.insert(tgt_handle.clone(), val);
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
                visited_workflows,
                graph_json,
                workflow_run_id,
                ephemeral,
            };
            executor.execute(&ctx, node_id, node_data, &incoming_value).await
        } else {
            if !ephemeral {
                let _ = record_event(db, session_id, "workflow.node.skipped", "desktop.workflow",
                    serde_json::json!({ "node_id": node_id, "node_type": node_type, "reason": "unsupported type" }));
            }
            emit_workflow_event(app, session_id, "workflow.node.skipped",
                serde_json::json!({ "node_id": node_id, "node_type": node_type }),
                &seq_counter);
            Ok(super::executors::NodeOutput::value(serde_json::Value::Null))
        };
        let node_duration = node_start.elapsed().as_millis() as i64;

        match result {
            Ok(node_output) => {
                // Handle skip_nodes from router/iterator
                for skip_id in &node_output.skip_nodes {
                    skipped_nodes.insert(skip_id.clone());
                }

                // Handle extra_outputs from iterator (pre-computed aggregator results)
                for (extra_id, extra_val) in node_output.extra_outputs {
                    node_outputs.insert(extra_id, extra_val);
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

                // Strip only __usage (internal stats) — preserve all handle-routable fields
                let clean_output = if let Some(obj) = output.as_object() {
                    if obj.contains_key("__usage") {
                        let mut cleaned = obj.clone();
                        cleaned.remove("__usage");
                        serde_json::Value::Object(cleaned)
                    } else {
                        output.clone()
                    }
                } else {
                    output.clone()
                };
                node_outputs.insert(node_id.clone(), clean_output.clone());

                let full_text = extract_primary_text(&clean_output);
                let output_preview = truncate(&full_text, 200).to_string();
                // DB event gets preview only (storage), UI event gets full output (display)
                if !ephemeral {
                    let _ = record_event(db, session_id, "workflow.node.completed", "desktop.workflow",
                        serde_json::json!({
                            "node_id": node_id, "node_type": node_type,
                            "output_preview": output_preview, "duration_ms": node_duration,
                        }));
                }
                emit_workflow_event(app, session_id, "workflow.node.completed",
                    serde_json::json!({
                        "node_id": node_id, "node_type": node_type,
                        "output_preview": output_preview,
                        "output_full": full_text,
                        "duration_ms": node_duration,
                    }),
                    &seq_counter);
            }
            Err(err) => {
                eprintln!(
                    "[workflow.node.error] session_id={} node_id={} node_type={} error={}",
                    session_id, node_id, node_type, err
                );
                if !ephemeral {
                    let _ = record_event(db, session_id, "workflow.node.error", "desktop.workflow",
                        serde_json::json!({
                            "node_id": node_id, "node_type": node_type,
                            "error": err, "duration_ms": node_duration,
                        }));
                }
                emit_workflow_event(app, session_id, "workflow.node.error",
                    serde_json::json!({ "node_id": node_id, "error": &err }),
                    &seq_counter);

                let total_duration = start_time.elapsed().as_millis() as i64;
                if !ephemeral {
                    let _ = record_event(db, session_id, "workflow.failed", "desktop.workflow",
                        serde_json::json!({
                            "node_id": node_id, "error": err,
                            "duration_ms": total_duration,
                        }));
                }
                emit_workflow_event(app, session_id, "workflow.failed",
                    serde_json::json!({ "node_id": node_id, "error": &err }),
                    &seq_counter);

                return Ok(WorkflowRunResult {
                    session_id: session_id.to_string(),
                    status: "failed".to_string(),
                    outputs: workflow_outputs,
                    node_outputs,
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
    if !ephemeral {
        let _ = record_event(db, session_id, "workflow.completed", "desktop.workflow",
            serde_json::json!({
                "duration_ms": total_duration, "total_tokens": total_tokens,
                "total_cost_usd": total_cost, "node_count": topo_order.len(),
            }));
    }
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
        node_outputs,
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

    // --- resolve_source_handle tests ---

    #[test]
    fn test_source_handle_default_output() {
        let mut outputs = HashMap::new();
        outputs.insert("llm_1".to_string(), serde_json::json!({"response": "hello", "usage": {"tokens": 10}}));
        let val = resolve_source_handle(&outputs, "llm_1", "output").unwrap();
        assert!(val.is_object());
        assert_eq!(val.get("response").unwrap().as_str().unwrap(), "hello");
    }

    #[test]
    fn test_source_handle_specific_field() {
        let mut outputs = HashMap::new();
        outputs.insert("llm_1".to_string(), serde_json::json!({
            "response": "The answer is 42",
            "usage": {"total_tokens": 100},
            "cost": "$0.000060"
        }));
        let val = resolve_source_handle(&outputs, "llm_1", "response").unwrap();
        assert_eq!(val.as_str().unwrap(), "The answer is 42");

        let val = resolve_source_handle(&outputs, "llm_1", "usage").unwrap();
        assert!(val.is_object());
        assert_eq!(val.get("total_tokens").unwrap().as_i64().unwrap(), 100);

        let val = resolve_source_handle(&outputs, "llm_1", "cost").unwrap();
        assert_eq!(val.as_str().unwrap(), "$0.000060");
    }

    #[test]
    fn test_source_handle_fallback_for_simple_output() {
        let mut outputs = HashMap::new();
        outputs.insert("transform_1".to_string(), serde_json::json!("plain text"));
        // Requesting a handle from a string output falls back to the string
        let val = resolve_source_handle(&outputs, "transform_1", "response").unwrap();
        assert_eq!(val.as_str().unwrap(), "plain text");
    }

    #[test]
    fn test_source_handle_missing_node() {
        let outputs = HashMap::new();
        assert!(resolve_source_handle(&outputs, "nonexistent", "output").is_none());
    }

    #[test]
    fn test_source_handle_branch_unwraps_router_value() {
        // Router output wraps value: {"selectedBranch": "done", "value": "hello"}
        // branch-0 handle should unwrap to "hello", not the whole wrapper
        let mut outputs = HashMap::new();
        outputs.insert("router_1".to_string(), serde_json::json!({
            "selectedBranch": "done",
            "value": "hello world"
        }));
        let val = resolve_source_handle(&outputs, "router_1", "branch-0").unwrap();
        assert_eq!(val.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_source_handle_branch_unwraps_router_object_value() {
        let mut outputs = HashMap::new();
        outputs.insert("router_1".to_string(), serde_json::json!({
            "selectedBranch": "analyze",
            "value": {"data": [1, 2, 3]}
        }));
        let val = resolve_source_handle(&outputs, "router_1", "branch-1").unwrap();
        assert!(val.is_object());
        assert_eq!(val.get("data").unwrap().as_array().unwrap().len(), 3);
    }

    // --- extract_primary_text tests ---

    #[test]
    fn test_primary_text_string() {
        assert_eq!(extract_primary_text(&serde_json::json!("hello")), "hello");
    }

    #[test]
    fn test_primary_text_structured_response() {
        let val = serde_json::json!({"response": "The answer", "usage": {"tokens": 10}});
        assert_eq!(extract_primary_text(&val), "The answer");
    }

    #[test]
    fn test_primary_text_structured_content() {
        let val = serde_json::json!({"content": "Some content", "metadata": {}});
        assert_eq!(extract_primary_text(&val), "Some content");
    }

    #[test]
    fn test_primary_text_no_primary_field() {
        let val = serde_json::json!({"foo": "bar", "baz": 42});
        // Falls back to JSON serialization
        let text = extract_primary_text(&val);
        assert!(text.contains("foo"));
        assert!(text.contains("bar"));
    }

    #[test]
    fn test_primary_text_router_object_value() {
        // Router output: {"selectedBranch": "done", "value": {"data": "result"}}
        // Should extract the inner object (value field), NOT the entire wrapper.
        let val = serde_json::json!({"selectedBranch": "done", "value": {"data": "result"}});
        let text = extract_primary_text(&val);
        // Should be the serialized inner object, not the whole wrapper
        assert!(text.contains("data"));
        assert!(text.contains("result"));
        assert!(!text.contains("selectedBranch"), "should not contain wrapper keys, got: {}", text);
    }

    #[test]
    fn test_primary_text_router_string_value() {
        // Router output: {"selectedBranch": "branch_a", "value": "hello world"}
        let val = serde_json::json!({"selectedBranch": "branch_a", "value": "hello world"});
        assert_eq!(extract_primary_text(&val), "hello world");
    }

    // --- resolve_template with structured LLM output ---

    #[test]
    fn test_resolve_structured_llm_output_alias() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!({
            "response": "The answer is 42",
            "content": "The answer is 42",
            "usage": {"total_tokens": 100},
            "cost": "$0.000060"
        }));
        let inputs = HashMap::new();
        // {{llm_1.output}} extracts primary text
        assert_eq!(
            resolve_template("{{llm_1.output}}", &node_outputs, &inputs),
            "The answer is 42"
        );
    }

    #[test]
    fn test_resolve_structured_llm_specific_field() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!({
            "response": "The answer is 42",
            "content": "The answer is 42",
            "usage": {"total_tokens": 100},
            "cost": "$0.000060"
        }));
        let inputs = HashMap::new();
        // {{llm_1.response}} returns the specific field
        assert_eq!(
            resolve_template("{{llm_1.response}}", &node_outputs, &inputs),
            "The answer is 42"
        );
        // {{llm_1.cost}} returns cost string
        assert_eq!(
            resolve_template("{{llm_1.cost}}", &node_outputs, &inputs),
            "$0.000060"
        );
        // {{llm_1.usage}} returns usage object as JSON
        let usage = resolve_template("{{llm_1.usage}}", &node_outputs, &inputs);
        assert!(usage.contains("total_tokens"));
    }

    #[test]
    fn test_resolve_structured_single_part_ref() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("llm_1".to_string(), serde_json::json!({
            "response": "The answer is 42",
            "usage": {"total_tokens": 100},
        }));
        let inputs = HashMap::new();
        // {{llm_1}} (no dot) extracts primary text
        assert_eq!(
            resolve_template("{{llm_1}}", &node_outputs, &inputs),
            "The answer is 42"
        );
    }

    #[test]
    fn test_resolve_array_index() {
        let mut node_outputs = HashMap::new();
        node_outputs.insert("transform_1".to_string(), serde_json::json!({
            "services": ["web-app", "auth-api", "gateway"],
            "tag": "latest",
        }));
        let inputs = HashMap::new();
        // {{transform_1.services[0]}} → first element
        assert_eq!(
            resolve_template("{{transform_1.services[0]}}", &node_outputs, &inputs),
            "web-app"
        );
        // {{transform_1.services[2]}} → third element
        assert_eq!(
            resolve_template("{{transform_1.services[2]}}", &node_outputs, &inputs),
            "gateway"
        );
        // {{transform_1.services[99]}} → out of bounds → null
        assert_eq!(
            resolve_template("{{transform_1.services[99]}}", &node_outputs, &inputs),
            "null"
        );
        // Non-array field with index → returns whole value
        assert_eq!(
            resolve_template("{{transform_1.tag}}", &node_outputs, &inputs),
            "latest"
        );
    }

    // --- Integration tests: simulate Input → LLM data flow ---

    /// Simulates the engine's incoming resolution for a named target handle.
    /// This is exactly what engine.rs lines 277-293 do.
    fn simulate_incoming_resolution(
        node_outputs: &HashMap<String, serde_json::Value>,
        edges: &[(String, String, String, String)], // (src_id, src_handle, tgt_id, tgt_handle)
        target_node: &str,
    ) -> Option<serde_json::Value> {
        let inc: Vec<_> = edges.iter()
            .filter(|(_, _, tgt, _)| tgt == target_node)
            .map(|(src, sh, _, th)| (src.clone(), sh.clone(), th.clone()))
            .collect();

        if inc.is_empty() { return None; }

        if inc.len() == 1 && inc[0].2 == "input" {
            return resolve_source_handle(node_outputs, &inc[0].0, &inc[0].1);
        }

        let mut obj = serde_json::Map::new();
        for (src_id, src_handle, tgt_handle) in &inc {
            if let Some(val) = resolve_source_handle(node_outputs, src_id, src_handle) {
                obj.insert(tgt_handle.clone(), val);
            }
        }
        if obj.is_empty() { None } else { Some(serde_json::Value::Object(obj)) }
    }

    /// Simulates LLM prompt resolution given incoming data and node config.
    fn simulate_llm_prompt(
        incoming: &Option<serde_json::Value>,
        node_data: &serde_json::Value,
        node_outputs: &HashMap<String, serde_json::Value>,
        inputs: &HashMap<String, serde_json::Value>,
    ) -> String {
        let incoming_prompt = incoming.as_ref()
            .and_then(|inc| inc.get("prompt"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let incoming_bare = incoming.as_ref()
            .and_then(|inc| inc.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let prompt_template = node_data.get("prompt").and_then(|v| v.as_str()).unwrap_or("{{input}}");

        if let Some(p) = incoming_prompt {
            return p;
        }
        if let Some(s) = incoming_bare {
            return s;
        }
        if prompt_template.contains("{{") {
            return resolve_template(prompt_template, node_outputs, inputs);
        }
        prompt_template.to_string()
    }

    #[test]
    fn test_input_to_llm_with_value_via_prompt_handle() {
        // Scenario: Input node outputs "2*8=?", edge to LLM "prompt" handle
        let mut node_outputs = HashMap::new();
        node_outputs.insert("input_1".to_string(), serde_json::json!("2*8=?"));

        // Edge: input_1:output → llm_1:prompt
        let edges = vec![
            ("input_1".to_string(), "output".to_string(), "llm_1".to_string(), "prompt".to_string()),
        ];
        let incoming = simulate_incoming_resolution(&node_outputs, &edges, "llm_1");
        assert_eq!(incoming, Some(serde_json::json!({"prompt": "2*8=?"})));

        // LLM resolves prompt from incoming "prompt" handle
        let llm_data = serde_json::json!({"provider": "google", "model": "gemini-2.0-flash"});
        let prompt = simulate_llm_prompt(&incoming, &llm_data, &node_outputs, &HashMap::new());
        assert_eq!(prompt, "2*8=?");
    }

    #[test]
    fn test_input_to_llm_empty_value_falls_to_template() {
        // Scenario: Input node outputs "" (empty), edge to LLM "prompt" handle
        let mut node_outputs = HashMap::new();
        node_outputs.insert("input_1".to_string(), serde_json::json!(""));

        let edges = vec![
            ("input_1".to_string(), "output".to_string(), "llm_1".to_string(), "prompt".to_string()),
        ];
        let incoming = simulate_incoming_resolution(&node_outputs, &edges, "llm_1");
        // incoming = {"prompt": ""} — empty string
        assert_eq!(incoming, Some(serde_json::json!({"prompt": ""})));

        // LLM skips empty "prompt" handle, falls to template
        let mut inputs = HashMap::new();
        inputs.insert("query".to_string(), serde_json::json!("What is 2+2?"));
        let llm_data = serde_json::json!({"provider": "google", "model": "gemini-2.0-flash"});
        let prompt = simulate_llm_prompt(&incoming, &llm_data, &node_outputs, &inputs);
        // Template {{input}} resolves from inputs — picks first value
        assert_eq!(prompt, "What is 2+2?");
    }

    #[test]
    fn test_input_to_llm_via_value_handle() {
        // Scenario: Input node source handle is "value" (not default "output")
        let mut node_outputs = HashMap::new();
        node_outputs.insert("input_1".to_string(), serde_json::json!("Hello world"));

        // Edge: input_1:value → llm_1:prompt
        let edges = vec![
            ("input_1".to_string(), "value".to_string(), "llm_1".to_string(), "prompt".to_string()),
        ];
        let incoming = simulate_incoming_resolution(&node_outputs, &edges, "llm_1");
        // resolve_source_handle with "value" on a string falls back to whole value
        assert_eq!(incoming, Some(serde_json::json!({"prompt": "Hello world"})));

        let llm_data = serde_json::json!({"provider": "local", "model": "qwen3"});
        let prompt = simulate_llm_prompt(&incoming, &llm_data, &node_outputs, &HashMap::new());
        assert_eq!(prompt, "Hello world");
    }

    #[test]
    fn test_input_to_llm_single_edge_to_default_handle() {
        // Scenario: single edge to default "input" handle (legacy)
        let mut node_outputs = HashMap::new();
        node_outputs.insert("input_1".to_string(), serde_json::json!("test prompt"));

        let edges = vec![
            ("input_1".to_string(), "output".to_string(), "llm_1".to_string(), "input".to_string()),
        ];
        let incoming = simulate_incoming_resolution(&node_outputs, &edges, "llm_1");
        // Single edge to "input" handle: flattened to bare value
        assert_eq!(incoming, Some(serde_json::json!("test prompt")));

        let llm_data = serde_json::json!({});
        let prompt = simulate_llm_prompt(&incoming, &llm_data, &node_outputs, &HashMap::new());
        // incoming is bare string → picked up by incoming_bare
        assert_eq!(prompt, "test prompt");
    }

    // --- Router backward compatibility integration tests ---
    // These simulate the EXACT data flow from existing templates to verify
    // the Router output change ({"selectedBranch", "value"}) doesn't break them.

    #[test]
    fn test_router_branch_to_output_content_moderator_flow() {
        // Simulates content-moderator.json:
        // LLM → Router → branch-0 (safe) → Output (value handle)
        // Router output should be unwrapped — Output node gets the moderation text, not the wrapper.
        let mut node_outputs = HashMap::new();

        // LLM produces moderation result
        node_outputs.insert("llm_1".to_string(), serde_json::json!({
            "response": "{\"verdict\": \"safe\", \"reason\": \"no issues\", \"confidence\": 0.98}",
        }));

        // Router receives LLM response and wraps with selectedBranch (this is what router.rs does)
        let router_output = serde_json::json!({
            "selectedBranch": "safe",
            "value": "{\"verdict\": \"safe\", \"reason\": \"no issues\", \"confidence\": 0.98}",
        });
        node_outputs.insert("router_1".to_string(), router_output);

        // content-moderator edges: router_1:branch-0 → output_1:value
        let edges = vec![
            ("router_1".to_string(), "branch-0".to_string(), "output_1".to_string(), "value".to_string()),
        ];

        let incoming = simulate_incoming_resolution(&node_outputs, &edges, "output_1");
        // Output node receives via "value" handle (not "input"), so it builds an object
        let incoming_val = incoming.unwrap();
        let value_field = incoming_val.get("value").unwrap();

        // The value should be the unwrapped moderation text, NOT the Router wrapper
        assert!(value_field.is_string(), "should be string, got: {}", value_field);
        assert!(value_field.as_str().unwrap().contains("verdict"),
            "should contain moderation result, got: {}", value_field);
        assert!(!value_field.to_string().contains("selectedBranch"),
            "should NOT contain Router wrapper, got: {}", value_field);
    }

    #[test]
    fn test_router_branch_to_llm_email_classifier_flow() {
        // Simulates email-classifier.json:
        // Router → branch-0 (urgent) → LLM (prompt handle)
        // LLM should receive the email text, not the Router wrapper.
        let mut node_outputs = HashMap::new();

        // Router wraps the email body with selectedBranch
        let router_output = serde_json::json!({
            "selectedBranch": "urgent",
            "value": "URGENT: Server is down, need immediate fix!",
        });
        node_outputs.insert("router_1".to_string(), router_output);

        // email-classifier edge: router_1:branch-0 → llm_2:prompt
        let edges = vec![
            ("router_1".to_string(), "branch-0".to_string(), "llm_2".to_string(), "prompt".to_string()),
        ];

        let incoming = simulate_incoming_resolution(&node_outputs, &edges, "llm_2");
        let incoming_val = incoming.unwrap();
        let prompt_field = incoming_val.get("prompt").unwrap();

        // LLM prompt handle should get the unwrapped email, not the wrapper
        assert_eq!(prompt_field.as_str().unwrap(), "URGENT: Server is down, need immediate fix!");
    }

    #[test]
    fn test_router_branch_to_default_input_handle() {
        // Edge case: branch-* edge goes to default "input" handle (single edge flatten)
        let mut node_outputs = HashMap::new();
        node_outputs.insert("router_1".to_string(), serde_json::json!({
            "selectedBranch": "done",
            "value": "final result",
        }));

        let edges = vec![
            ("router_1".to_string(), "branch-0".to_string(), "next_1".to_string(), "input".to_string()),
        ];

        let incoming = simulate_incoming_resolution(&node_outputs, &edges, "next_1");
        // Single edge to "input" handle: flattened to bare value (unwrapped)
        assert_eq!(incoming, Some(serde_json::json!("final result")));
    }

    #[test]
    fn test_router_branch_with_object_value() {
        // Router routes a JSON object (not a string) — downstream should get the object, not wrapper
        let mut node_outputs = HashMap::new();
        node_outputs.insert("router_1".to_string(), serde_json::json!({
            "selectedBranch": "analyze",
            "value": {"metrics": {"cpu": 95, "memory": 80}, "alert": true},
        }));

        let edges = vec![
            ("router_1".to_string(), "branch-0".to_string(), "transform_1".to_string(), "input".to_string()),
        ];

        let incoming = simulate_incoming_resolution(&node_outputs, &edges, "transform_1");
        let val = incoming.unwrap();

        // Should get the inner object, not the wrapper
        assert!(val.is_object());
        assert!(val.get("metrics").is_some(), "should have metrics, got: {}", val);
        assert!(val.get("selectedBranch").is_none(), "should NOT have selectedBranch, got: {}", val);
    }

    // --- Transitive skip with pre-computed outputs ---

    #[test]
    fn test_transitive_skip_respects_precomputed_outputs() {
        // Scenario: Loop marks Exit as skipped, but pre-computes Exit's output via extra_outputs.
        // Output node (downstream of Exit) should NOT be transitively skipped because
        // Exit has a pre-computed entry in node_outputs.
        let mut skipped_nodes: HashSet<String> = HashSet::new();
        skipped_nodes.insert("llm_1".to_string());  // subgraph node — truly skipped
        skipped_nodes.insert("exit_1".to_string());  // exit — skipped but has pre-computed output

        let mut node_outputs: HashMap<String, serde_json::Value> = HashMap::new();
        // Loop pre-computed the exit output via extra_outputs
        node_outputs.insert("exit_1".to_string(), serde_json::json!("final answer"));

        // Output node has one predecessor: exit_1
        let incoming_edges: Vec<(String, String, String)> = vec![
            ("exit_1".to_string(), "output".to_string(), "value".to_string()),
        ];

        // Transitive skip check (mirrors engine logic):
        // "skip if all preds are skipped AND have no pre-computed output"
        let should_skip = !incoming_edges.is_empty() && incoming_edges.iter().all(|(src, _, _)| {
            skipped_nodes.contains(src) && !node_outputs.contains_key(src)
        });

        assert!(!should_skip, "Output node should NOT be skipped when Exit has pre-computed output");

        // Contrast: if exit had NO pre-computed output, it SHOULD be skipped
        node_outputs.remove("exit_1");
        let should_skip_now = !incoming_edges.is_empty() && incoming_edges.iter().all(|(src, _, _)| {
            skipped_nodes.contains(src) && !node_outputs.contains_key(src)
        });
        assert!(should_skip_now, "Output node SHOULD be skipped when Exit is truly skipped with no output");
    }

    #[test]
    fn test_router_template_resolution_backward_compat() {
        // Verify {{router_1.output}} in templates resolves to inner value, not wrapper
        let mut node_outputs = HashMap::new();
        node_outputs.insert("router_1".to_string(), serde_json::json!({
            "selectedBranch": "approved",
            "value": "The document has been approved.",
        }));

        let inputs = HashMap::new();
        let resolved = resolve_template("Result: {{router_1.output}}", &node_outputs, &inputs);
        // extract_primary_text should extract "value" field from the wrapper
        assert_eq!(resolved, "Result: The document has been approved.");
        assert!(!resolved.contains("selectedBranch"));
    }
}

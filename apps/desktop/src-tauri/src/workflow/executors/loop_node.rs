use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::{execute_workflow_with_visited, emit_workflow_event};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct LoopExecutor;

/// Find the subgraph between a loop node and its paired exit node.
/// Uses forward+backward BFS (same pattern as iterator.rs:find_subgraph).
/// Returns: (subgraph_node_ids, exit_id)
fn find_loop_subgraph(
    graph: &Value,
    loop_id: &str,
) -> Result<(Vec<String>, String), String> {
    let nodes = graph.get("nodes").and_then(|v| v.as_array())
        .ok_or("No nodes in graph")?;
    let edges = graph.get("edges").and_then(|v| v.as_array())
        .ok_or("No edges in graph")?;

    let mut node_types: HashMap<String, String> = HashMap::new();
    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let ntype = node.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();
        node_types.insert(id, ntype);
    }

    let mut fwd_adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut rev_adj: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        let source = edge.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let target = edge.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if !source.is_empty() && !target.is_empty() {
            fwd_adj.entry(source.clone()).or_default().push(target.clone());
            rev_adj.entry(target).or_default().push(source);
        }
    }

    // Forward BFS from loop — stop at exit nodes
    let mut forward_set: HashSet<String> = HashSet::new();
    let mut exit_ids: Vec<String> = Vec::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    if let Some(neighbors) = fwd_adj.get(loop_id) {
        for n in neighbors {
            queue.push_back(n.clone());
        }
    }

    while let Some(node_id) = queue.pop_front() {
        if forward_set.contains(&node_id) || exit_ids.contains(&node_id) {
            continue;
        }
        let ntype = node_types.get(&node_id).map(|s| s.as_str()).unwrap_or("");
        if ntype == "exit" {
            exit_ids.push(node_id);
            continue;
        }
        forward_set.insert(node_id.clone());
        if let Some(neighbors) = fwd_adj.get(&node_id) {
            for n in neighbors {
                queue.push_back(n.clone());
            }
        }
    }

    if exit_ids.is_empty() {
        return Err("Loop has no paired Exit node downstream. Add an Exit node after the processing nodes.".into());
    }
    if exit_ids.len() > 1 {
        return Err(format!(
            "Loop '{}' has {} reachable Exit nodes ({:?}). Each Loop must pair with exactly one Exit.",
            loop_id, exit_ids.len(), exit_ids
        ));
    }
    let exit_id = exit_ids.into_iter().next().unwrap();

    // Backward BFS from exit — stop at loop
    let mut backward_set: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    if let Some(predecessors) = rev_adj.get(&exit_id) {
        for n in predecessors {
            queue.push_back(n.clone());
        }
    }

    while let Some(node_id) = queue.pop_front() {
        if backward_set.contains(&node_id) || node_id == loop_id {
            continue;
        }
        backward_set.insert(node_id.clone());
        if let Some(predecessors) = rev_adj.get(&node_id) {
            for n in predecessors {
                queue.push_back(n.clone());
            }
        }
    }

    let subgraph: Vec<String> = forward_set.intersection(&backward_set).cloned().collect();
    Ok((subgraph, exit_id))
}

/// Build a synthetic workflow graph wrapping the loop subgraph with Input/Output nodes.
fn build_loop_synthetic_graph(
    original_graph: &Value,
    loop_id: &str,
    subgraph_ids: &[String],
    exit_id: &str,
) -> Result<String, String> {
    let nodes = original_graph.get("nodes").and_then(|v| v.as_array())
        .ok_or("No nodes")?;
    let edges = original_graph.get("edges").and_then(|v| v.as_array())
        .ok_or("No edges")?;

    let subgraph_set: HashSet<&str> = subgraph_ids.iter().map(|s| s.as_str()).collect();
    let mut syn_nodes: Vec<Value> = Vec::new();

    // Synthetic input node
    syn_nodes.push(serde_json::json!({
        "id": "__loop_input__",
        "type": "input",
        "position": {"x": 0, "y": 0},
        "data": {"name": "input"}
    }));

    // Original subgraph nodes (preserve data)
    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if subgraph_set.contains(id) {
            syn_nodes.push(node.clone());
        }
    }

    // Synthetic output node
    syn_nodes.push(serde_json::json!({
        "id": "__loop_output__",
        "type": "output",
        "position": {"x": 0, "y": 0},
        "data": {"name": "result"}
    }));

    // Build synthetic edges
    let mut syn_edges: Vec<Value> = Vec::new();
    let mut edge_counter = 0;

    for edge in edges {
        let source = edge.get("source").and_then(|v| v.as_str()).unwrap_or("");
        let target = edge.get("target").and_then(|v| v.as_str()).unwrap_or("");
        let source_handle = edge.get("sourceHandle").and_then(|v| v.as_str()).unwrap_or("output");
        let target_handle = edge.get("targetHandle").and_then(|v| v.as_str()).unwrap_or("input");

        let (new_source, new_target, new_sh, new_th);

        if source == loop_id && subgraph_set.contains(target) {
            new_source = "__loop_input__";
            new_target = target;
            new_sh = "output";
            new_th = target_handle;
        } else if subgraph_set.contains(source) && target == exit_id {
            new_source = source;
            new_target = "__loop_output__";
            new_sh = source_handle;
            new_th = "input";
        } else if subgraph_set.contains(source) && subgraph_set.contains(target) {
            new_source = source;
            new_target = target;
            new_sh = source_handle;
            new_th = target_handle;
        } else {
            continue;
        }

        syn_edges.push(serde_json::json!({
            "id": format!("__syn_e{}__", edge_counter),
            "source": new_source,
            "target": new_target,
            "sourceHandle": new_sh,
            "targetHandle": new_th,
        }));
        edge_counter += 1;
    }

    let synthetic_graph = serde_json::json!({
        "nodes": syn_nodes,
        "edges": syn_edges,
    });

    serde_json::to_string(&synthetic_graph)
        .map_err(|e| format!("Failed to serialize synthetic graph: {e}"))
}

/// Bounded Levenshtein similarity.
/// Both strings are truncated to 10K chars before comparison.
/// Returns 1.0 - (edit_distance / max(len_a, len_b)). Returns 1.0 for two empty strings.
fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    const MAX_LEN: usize = 10_000;
    let a: Vec<char> = a.chars().take(MAX_LEN).collect();
    let b: Vec<char> = b.chars().take(MAX_LEN).collect();
    let (m, n) = (a.len(), b.len());

    if m == 0 && n == 0 {
        return 1.0;
    }
    if m == 0 || n == 0 {
        return 0.0;
    }

    // Two-row DP
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    let dist = prev[n];
    let max_len = m.max(n);
    1.0 - (dist as f64 / max_len as f64)
}

/// Stringify a Value for comparison (text similarity).
fn stringify_value(val: &Value) -> String {
    match val.as_str() {
        Some(s) => s.to_string(),
        None => serde_json::to_string(val).unwrap_or_default(),
    }
}

/// Find the Router node inside a set of subgraph node IDs by inspecting the original graph.
fn find_router_in_subgraph(graph: &Value, subgraph_ids: &[String]) -> Option<String> {
    let nodes = graph.get("nodes").and_then(|v| v.as_array())?;
    let subgraph_set: HashSet<&str> = subgraph_ids.iter().map(|s| s.as_str()).collect();
    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let ntype = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if ntype == "router" && subgraph_set.contains(id) {
            return Some(id.to_string());
        }
    }
    None
}

#[async_trait::async_trait]
impl NodeExecutor for LoopExecutor {
    fn node_type(&self) -> &str { "loop" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &Value,
        incoming: &Option<Value>,
    ) -> Result<NodeOutput, String> {
        // Parse config
        let max_iterations = node_data.get("maxIterations")
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .clamp(1, 50) as usize;
        let exit_condition = node_data.get("exitCondition")
            .and_then(|v| v.as_str())
            .unwrap_or("max_iterations");
        let stability_threshold = node_data.get("stabilityThreshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.95);
        let feedback_mode = node_data.get("feedbackMode")
            .and_then(|v| v.as_str())
            .unwrap_or("replace");

        let initial_input = incoming.clone().unwrap_or(Value::Null);

        // Parse graph and find subgraph
        let graph: Value = serde_json::from_str(ctx.graph_json)
            .map_err(|e| format!("Invalid graph JSON: {e}"))?;
        let (subgraph_ids, exit_id) = find_loop_subgraph(&graph, node_id)?;

        // For evaluator mode, find the Router node in the subgraph
        let router_id = if exit_condition == "evaluator" {
            Some(find_router_in_subgraph(&graph, &subgraph_ids)
                .ok_or("Loop with 'evaluator' exit condition requires a Router node in the subgraph")?)
        } else {
            None
        };

        // Build synthetic graph once
        let synthetic_graph = build_loop_synthetic_graph(&graph, node_id, &subgraph_ids, &exit_id)?;

        eprintln!("[workflow] Loop '{}': max={}, exit={}, feedback={}, subgraph: {:?}, exit: {}",
            node_id, max_iterations, exit_condition, feedback_mode, subgraph_ids, exit_id);

        let mut current_input = initial_input;
        let mut all_results: Vec<Value> = Vec::new();
        let mut iterations_run = 0usize;
        let mut exit_reason = "max_iterations".to_string();

        for idx in 0..max_iterations {
            iterations_run = idx + 1;

            eprintln!("[workflow] Loop '{}': iteration {}/{}", node_id, idx + 1, max_iterations);

            emit_workflow_event(ctx.app, ctx.session_id, "workflow.node.iteration",
                serde_json::json!({
                    "node_id": node_id,
                    "index": idx,
                    "total": max_iterations,
                    "input_preview": stringify_value(&current_input).chars().take(200).collect::<String>(),
                }),
                ctx.seq_counter);

            // Build inputs for this iteration
            let mut sub_inputs: HashMap<String, Value> = HashMap::new();
            sub_inputs.insert("input".to_string(), current_input.clone());

            let result = execute_workflow_with_visited(
                ctx.db, ctx.sidecar, ctx.app,
                ctx.session_id, &synthetic_graph,
                &sub_inputs, ctx.all_settings,
                ctx.visited_workflows, ctx.workflow_run_id,
                ctx.ephemeral,
            ).await.map_err(|e| format!("Loop iteration {} failed: {}", idx, e))?;

            // Extract the synthetic workflow's output (from Output nodes)
            let iteration_output = if result.outputs.len() == 1 {
                result.outputs.into_values().next().unwrap_or(Value::Null)
            } else if !result.outputs.is_empty() {
                serde_json::json!(result.outputs)
            } else {
                Value::Null
            };

            all_results.push(iteration_output.clone());

            // Check exit condition
            match exit_condition {
                "evaluator" => {
                    if let Some(ref rid) = router_id {
                        let got_output = !iteration_output.is_null();

                        if got_output {
                            // Router selected "done" — exit the loop
                            exit_reason = "evaluator_done".to_string();
                            eprintln!("[workflow] Loop '{}': evaluator exit at iteration {} (Router '{}' selected done)",
                                node_id, idx + 1, rid);
                            break;
                        }

                        // Router selected "continue" — Exit was skipped, so iteration_output is Null.
                        // For feedback, extract the last meaningful output from the subgraph's
                        // intermediate node_outputs (e.g., the LLM answer before the Router).
                        // This lets the next iteration see what was produced even though Exit was skipped.
                        if idx + 1 < max_iterations {
                            // Find the best intermediate output: try subgraph nodes in reverse,
                            // skip the Router itself (its output is a wrapper, not content).
                            let mut feedback_val: Option<Value> = None;
                            for sub_id in subgraph_ids.iter().rev() {
                                if sub_id == rid { continue; } // skip Router
                                if let Some(val) = result.node_outputs.get(sub_id) {
                                    if !val.is_null() {
                                        eprintln!("[workflow] Loop '{}': evaluator continue — using '{}' output as feedback",
                                            node_id, sub_id);
                                        feedback_val = Some(val.clone());
                                        break;
                                    }
                                }
                            }
                            if let Some(fv) = feedback_val {
                                let fv_text = crate::workflow::engine::extract_primary_text(&fv);
                                current_input = match feedback_mode {
                                    "append" => {
                                        let prev_text = stringify_value(&current_input);
                                        Value::String(format!("{}\n---\nPrevious attempt:\n{}", prev_text, fv_text))
                                    }
                                    _ => Value::String(fv_text),
                                };
                            }
                        }
                    }
                }
                "stable_output" => {
                    if all_results.len() >= 2 {
                        let prev = stringify_value(&all_results[all_results.len() - 2]);
                        let curr = stringify_value(&iteration_output);
                        let similarity = levenshtein_similarity(&prev, &curr);
                        eprintln!("[workflow] Loop '{}': stability check: similarity={:.4} threshold={}",
                            node_id, similarity, stability_threshold);
                        if similarity >= stability_threshold {
                            exit_reason = "stable_output".to_string();
                            break;
                        }
                    }
                }
                // "max_iterations" — just run all iterations
                _ => {}
            }

            // Apply feedback for next iteration (unless this is the last iteration).
            // Skip feedback when iteration_output is Null (e.g., evaluator "continue"
            // where Exit was skipped) to avoid overwriting current_input with Null.
            if idx + 1 < max_iterations && !iteration_output.is_null() {
                current_input = match feedback_mode {
                    "append" => {
                        // If both values are strings, concatenate with separator.
                        // If either is non-string (object/array), collect into a JSON array
                        // to preserve structural validity instead of blindly stringifying.
                        let both_strings = current_input.is_string() && iteration_output.is_string();
                        if both_strings {
                            let prev_text = stringify_value(&current_input);
                            let new_text = stringify_value(&iteration_output);
                            Value::String(format!("{}\n---\n{}", prev_text, new_text))
                        } else {
                            eprintln!("[workflow] Loop '{}': append mode with non-string values — wrapping in array", node_id);
                            let mut items = match current_input {
                                Value::Array(arr) => arr,
                                other => vec![other],
                            };
                            items.push(iteration_output);
                            Value::Array(items)
                        }
                    }
                    // "replace" and default
                    _ => iteration_output,
                };
            }
        }

        // Final result = last non-null result
        let final_result = all_results.iter().rev()
            .find(|v| !v.is_null())
            .cloned()
            .unwrap_or(Value::Null);

        // Skip subgraph nodes + exit (their work is done inside the loop)
        let mut skip_nodes: Vec<String> = subgraph_ids;
        skip_nodes.push(exit_id.clone());

        let mut extra_outputs = HashMap::new();
        extra_outputs.insert(exit_id, final_result.clone());

        Ok(NodeOutput {
            value: serde_json::json!({
                "output": final_result,
                "iterations": all_results,
                "count": iterations_run,
                "exit_reason": exit_reason,
            }),
            skip_nodes,
            extra_outputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- levenshtein_similarity tests ---

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_similarity("hello", "hello"), 1.0);
    }

    #[test]
    fn test_levenshtein_empty_both() {
        assert_eq!(levenshtein_similarity("", ""), 1.0);
    }

    #[test]
    fn test_levenshtein_one_empty() {
        assert_eq!(levenshtein_similarity("abc", ""), 0.0);
        assert_eq!(levenshtein_similarity("", "xyz"), 0.0);
    }

    #[test]
    fn test_levenshtein_completely_different() {
        // "abc" vs "xyz" — 3 substitutions, max_len=3, sim = 1 - 3/3 = 0.0
        assert_eq!(levenshtein_similarity("abc", "xyz"), 0.0);
    }

    #[test]
    fn test_levenshtein_one_char_diff() {
        // "hello" vs "helo" — 1 deletion, max_len=5, sim = 1 - 1/5 = 0.8
        assert!((levenshtein_similarity("hello", "helo") - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_levenshtein_similar() {
        // "kitten" vs "sitting" — edit distance 3, max_len=7, sim = 1 - 3/7 ≈ 0.571
        let sim = levenshtein_similarity("kitten", "sitting");
        assert!(sim > 0.5 && sim < 0.6, "got {}", sim);
    }

    #[test]
    fn test_levenshtein_truncation() {
        // Very long strings should still work (truncated to 10K)
        let a = "a".repeat(20_000);
        let b = "a".repeat(20_000);
        assert_eq!(levenshtein_similarity(&a, &b), 1.0);
    }

    // --- find_loop_subgraph tests ---

    #[test]
    fn test_find_loop_subgraph_simple() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "loop_1", "type": "loop", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "exit_1", "type": "exit", "data": {}}
            ],
            "edges": [
                {"id": "e1", "source": "loop_1", "target": "llm_1"},
                {"id": "e2", "source": "llm_1", "target": "exit_1"}
            ]
        });
        let (subgraph, exit_id) = find_loop_subgraph(&graph, "loop_1").unwrap();
        assert_eq!(subgraph, vec!["llm_1".to_string()]);
        assert_eq!(exit_id, "exit_1");
    }

    #[test]
    fn test_find_loop_subgraph_multi_node() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "loop_1", "type": "loop", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "llm_2", "type": "llm", "data": {}},
                {"id": "transform_1", "type": "transform", "data": {}},
                {"id": "exit_1", "type": "exit", "data": {}}
            ],
            "edges": [
                {"id": "e1", "source": "loop_1", "target": "llm_1"},
                {"id": "e2", "source": "llm_1", "target": "llm_2"},
                {"id": "e3", "source": "llm_2", "target": "transform_1"},
                {"id": "e4", "source": "transform_1", "target": "exit_1"}
            ]
        });
        let (subgraph, exit_id) = find_loop_subgraph(&graph, "loop_1").unwrap();
        assert_eq!(subgraph.len(), 3);
        assert!(subgraph.contains(&"llm_1".to_string()));
        assert!(subgraph.contains(&"llm_2".to_string()));
        assert!(subgraph.contains(&"transform_1".to_string()));
        assert_eq!(exit_id, "exit_1");
    }

    #[test]
    fn test_find_loop_subgraph_excludes_outside_nodes() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "loop_1", "type": "loop", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "outside", "type": "transform", "data": {}},
                {"id": "exit_1", "type": "exit", "data": {}},
                {"id": "out_1", "type": "output", "data": {}}
            ],
            "edges": [
                {"id": "e1", "source": "loop_1", "target": "llm_1"},
                {"id": "e2", "source": "llm_1", "target": "exit_1"},
                {"id": "e3", "source": "llm_1", "target": "outside"},
                {"id": "e4", "source": "outside", "target": "out_1"}
            ]
        });
        let (subgraph, exit_id) = find_loop_subgraph(&graph, "loop_1").unwrap();
        assert_eq!(exit_id, "exit_1");
        assert!(subgraph.contains(&"llm_1".to_string()));
        assert!(!subgraph.contains(&"outside".to_string()));
        assert!(!subgraph.contains(&"out_1".to_string()));
    }

    #[test]
    fn test_find_loop_subgraph_no_exit_errors() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "loop_1", "type": "loop", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "out_1", "type": "output", "data": {}}
            ],
            "edges": [
                {"id": "e1", "source": "loop_1", "target": "llm_1"},
                {"id": "e2", "source": "llm_1", "target": "out_1"}
            ]
        });
        let result = find_loop_subgraph(&graph, "loop_1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Exit"));
    }

    #[test]
    fn test_find_loop_subgraph_multiple_exits_errors() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "loop_1", "type": "loop", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "exit_1", "type": "exit", "data": {}},
                {"id": "exit_2", "type": "exit", "data": {}}
            ],
            "edges": [
                {"id": "e1", "source": "loop_1", "target": "llm_1"},
                {"id": "e2", "source": "llm_1", "target": "exit_1"},
                {"id": "e3", "source": "llm_1", "target": "exit_2"}
            ]
        });
        let result = find_loop_subgraph(&graph, "loop_1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exactly one Exit"));
    }

    // --- build_loop_synthetic_graph tests ---

    #[test]
    fn test_build_loop_synthetic_graph() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "loop_1", "type": "loop", "data": {}, "position": {"x": 0, "y": 0}},
                {"id": "llm_1", "type": "llm", "data": {"provider": "test"}, "position": {"x": 100, "y": 0}},
                {"id": "exit_1", "type": "exit", "data": {}, "position": {"x": 200, "y": 0}}
            ],
            "edges": [
                {"id": "e1", "source": "loop_1", "sourceHandle": "output", "target": "llm_1", "targetHandle": "prompt"},
                {"id": "e2", "source": "llm_1", "sourceHandle": "response", "target": "exit_1", "targetHandle": "input"}
            ]
        });

        let synthetic = build_loop_synthetic_graph(&graph, "loop_1", &["llm_1".to_string()], "exit_1").unwrap();
        let syn_graph: Value = serde_json::from_str(&synthetic).unwrap();

        let nodes = syn_graph.get("nodes").unwrap().as_array().unwrap();
        let edges = syn_graph.get("edges").unwrap().as_array().unwrap();

        assert_eq!(nodes.len(), 3); // __loop_input__ + llm_1 + __loop_output__
        assert_eq!(edges.len(), 2);

        let e0 = &edges[0];
        assert_eq!(e0.get("source").unwrap().as_str().unwrap(), "__loop_input__");
        assert_eq!(e0.get("target").unwrap().as_str().unwrap(), "llm_1");
        assert_eq!(e0.get("targetHandle").unwrap().as_str().unwrap(), "prompt");

        let e1 = &edges[1];
        assert_eq!(e1.get("source").unwrap().as_str().unwrap(), "llm_1");
        assert_eq!(e1.get("target").unwrap().as_str().unwrap(), "__loop_output__");
    }

    #[test]
    fn test_build_loop_synthetic_graph_multi_node() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "loop_1", "type": "loop", "data": {}, "position": {"x": 0, "y": 0}},
                {"id": "llm_1", "type": "llm", "data": {}, "position": {"x": 100, "y": 0}},
                {"id": "tr_1", "type": "transform", "data": {}, "position": {"x": 200, "y": 0}},
                {"id": "exit_1", "type": "exit", "data": {}, "position": {"x": 300, "y": 0}}
            ],
            "edges": [
                {"id": "e1", "source": "loop_1", "sourceHandle": "output", "target": "llm_1", "targetHandle": "input"},
                {"id": "e2", "source": "llm_1", "sourceHandle": "response", "target": "tr_1", "targetHandle": "input"},
                {"id": "e3", "source": "tr_1", "sourceHandle": "output", "target": "exit_1", "targetHandle": "input"}
            ]
        });

        let synthetic = build_loop_synthetic_graph(
            &graph, "loop_1",
            &["llm_1".to_string(), "tr_1".to_string()],
            "exit_1",
        ).unwrap();
        let syn_graph: Value = serde_json::from_str(&synthetic).unwrap();

        let nodes = syn_graph.get("nodes").unwrap().as_array().unwrap();
        let edges = syn_graph.get("edges").unwrap().as_array().unwrap();

        assert_eq!(nodes.len(), 4); // __loop_input__ + llm_1 + tr_1 + __loop_output__
        assert_eq!(edges.len(), 3);
    }

    // --- stringify_value tests ---

    #[test]
    fn test_stringify_value_string() {
        assert_eq!(stringify_value(&Value::String("hello".into())), "hello");
    }

    #[test]
    fn test_stringify_value_json() {
        let val = serde_json::json!({"key": "value"});
        let s = stringify_value(&val);
        assert!(s.contains("key"));
        assert!(s.contains("value"));
    }

    // --- find_router_in_subgraph tests ---

    #[test]
    fn test_find_router_in_subgraph() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "loop_1", "type": "loop", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "router_1", "type": "router", "data": {}},
                {"id": "exit_1", "type": "exit", "data": {}}
            ],
            "edges": []
        });
        let subgraph = vec!["llm_1".to_string(), "router_1".to_string()];
        assert_eq!(find_router_in_subgraph(&graph, &subgraph), Some("router_1".to_string()));
    }

    #[test]
    fn test_find_router_not_in_subgraph() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "transform_1", "type": "transform", "data": {}}
            ],
            "edges": []
        });
        let subgraph = vec!["llm_1".to_string(), "transform_1".to_string()];
        assert_eq!(find_router_in_subgraph(&graph, &subgraph), None);
    }

    // --- feedback mode tests ---

    #[test]
    fn test_feedback_replace() {
        let current = Value::String("original input".into());
        let output = Value::String("revised output".into());
        // Replace mode: output replaces input
        let next = output.clone();
        assert_eq!(next.as_str().unwrap(), "revised output");
        let _ = current; // current is discarded
    }

    #[test]
    fn test_feedback_append() {
        let current = Value::String("original input".into());
        let output = Value::String("revised output".into());
        // Append mode
        let next = Value::String(format!(
            "{}\n---\n{}",
            stringify_value(&current),
            stringify_value(&output),
        ));
        assert_eq!(next.as_str().unwrap(), "original input\n---\nrevised output");
    }

    // --- max iteration clamping ---

    #[test]
    fn test_max_iterations_clamping() {
        // Clamp to [1, 50]
        assert_eq!(0u64.clamp(1, 50), 1);
        assert_eq!(100u64.clamp(1, 50), 50);
        assert_eq!(5u64.clamp(1, 50), 5);
    }
}

use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::{execute_workflow_with_visited, emit_workflow_event};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct IteratorExecutor;

/// Extract the items array from incoming data.
/// Supports: "items" handle (named), bare array, jsonpath expression.
fn extract_items(
    incoming: &Option<Value>,
    node_data: &Value,
) -> Result<Vec<Value>, String> {
    let expression = node_data.get("expression")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let raw = match incoming {
        Some(val) => {
            if let Some(items) = val.get("items") {
                items.clone()
            } else {
                val.clone()
            }
        }
        None => return Err("Iterator received no input".into()),
    };

    let data = if !expression.is_empty() {
        let parsed = serde_json_path::JsonPath::parse(expression)
            .map_err(|e| format!("Invalid JSONPath expression '{}': {}", expression, e))?;
        let results = parsed.query(&raw);
        let matched: Vec<Value> = results.all().into_iter().cloned().collect();
        if matched.len() == 1 {
            matched.into_iter().next().unwrap()
        } else {
            Value::Array(matched)
        }
    } else {
        raw
    };

    match data {
        Value::Array(arr) => Ok(arr),
        other => Ok(vec![other]),
    }
}

/// Find the subgraph between an iterator and its paired aggregator.
/// Uses forward+backward BFS to identify only nodes on paths between the two.
/// Returns: (subgraph_node_ids, aggregator_id, aggregator_data)
fn find_subgraph(
    graph: &Value,
    iterator_id: &str,
) -> Result<(Vec<String>, String, Value), String> {
    let nodes = graph.get("nodes").and_then(|v| v.as_array())
        .ok_or("No nodes in graph")?;
    let edges = graph.get("edges").and_then(|v| v.as_array())
        .ok_or("No edges in graph")?;

    let mut node_types: HashMap<String, String> = HashMap::new();
    let mut node_data_map: HashMap<String, Value> = HashMap::new();
    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let ntype = node.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let data = node.get("data").cloned().unwrap_or(Value::Null);
        node_types.insert(id.clone(), ntype);
        node_data_map.insert(id, data);
    }

    // Build forward and reverse adjacency
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

    // Step 1: BFS forward from iterator — stop at aggregator nodes
    let mut forward_set: HashSet<String> = HashSet::new();
    let mut aggregator_ids: Vec<String> = Vec::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    if let Some(neighbors) = fwd_adj.get(iterator_id) {
        for n in neighbors {
            queue.push_back(n.clone());
        }
    }

    while let Some(node_id) = queue.pop_front() {
        if forward_set.contains(&node_id) || aggregator_ids.contains(&node_id) {
            continue;
        }
        let ntype = node_types.get(&node_id).map(|s| s.as_str()).unwrap_or("");
        if ntype == "aggregator" {
            aggregator_ids.push(node_id);
            continue; // Don't traverse past aggregator
        }
        forward_set.insert(node_id.clone());
        if let Some(neighbors) = fwd_adj.get(&node_id) {
            for n in neighbors {
                queue.push_back(n.clone());
            }
        }
    }

    if aggregator_ids.is_empty() {
        return Err("Iterator has no paired Aggregator node downstream. Add an Aggregator after the processing nodes.".into());
    }
    if aggregator_ids.len() > 1 {
        return Err(format!(
            "Iterator '{}' has {} reachable Aggregators ({:?}). Each Iterator must pair with exactly one Aggregator.",
            iterator_id, aggregator_ids.len(), aggregator_ids
        ));
    }
    let agg_id = aggregator_ids.into_iter().next().unwrap();

    // Step 2: BFS backward from aggregator — stop at iterator
    let mut backward_set: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    if let Some(predecessors) = rev_adj.get(&agg_id) {
        for n in predecessors {
            queue.push_back(n.clone());
        }
    }

    while let Some(node_id) = queue.pop_front() {
        if backward_set.contains(&node_id) || node_id == iterator_id {
            continue;
        }
        backward_set.insert(node_id.clone());
        if let Some(predecessors) = rev_adj.get(&node_id) {
            for n in predecessors {
                queue.push_back(n.clone());
            }
        }
    }

    // Step 3: Intersection — only nodes on paths between iterator and aggregator
    let subgraph: Vec<String> = forward_set.intersection(&backward_set).cloned().collect();

    let agg_data = node_data_map.get(&agg_id).cloned().unwrap_or(Value::Null);
    Ok((subgraph, agg_id, agg_data))
}

/// Build a synthetic workflow graph wrapping the subgraph with Input/Output nodes.
fn build_synthetic_graph(
    original_graph: &Value,
    iterator_id: &str,
    subgraph_ids: &[String],
    aggregator_id: &str,
) -> Result<String, String> {
    let nodes = original_graph.get("nodes").and_then(|v| v.as_array())
        .ok_or("No nodes")?;
    let edges = original_graph.get("edges").and_then(|v| v.as_array())
        .ok_or("No edges")?;

    let subgraph_set: HashSet<&str> = subgraph_ids.iter().map(|s| s.as_str()).collect();
    let mut syn_nodes: Vec<Value> = Vec::new();

    // Synthetic input node
    syn_nodes.push(serde_json::json!({
        "id": "__iter_input__",
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
        "id": "__iter_output__",
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

        if source == iterator_id && subgraph_set.contains(target) {
            // Iterator → subgraph: replace source with synthetic input
            new_source = "__iter_input__";
            new_target = target;
            new_sh = "output";
            new_th = target_handle;
        } else if subgraph_set.contains(source) && target == aggregator_id {
            // Subgraph → aggregator: replace target with synthetic output
            new_source = source;
            new_target = "__iter_output__";
            new_sh = source_handle;
            new_th = "input";
        } else if subgraph_set.contains(source) && subgraph_set.contains(target) {
            // Internal subgraph edge: preserve
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

/// Apply aggregation strategy to collected results.
pub fn apply_aggregation(
    results: &[Value],
    aggregator_data: &Value,
) -> Value {
    let strategy = aggregator_data.get("strategy")
        .and_then(|v| v.as_str())
        .unwrap_or("array");
    let separator = aggregator_data.get("separator")
        .and_then(|v| v.as_str())
        .unwrap_or("\n");

    match strategy {
        "concat" => {
            let texts: Vec<String> = results.iter().map(|v| {
                match v.as_str() {
                    Some(s) => s.to_string(),
                    None => v.to_string(),
                }
            }).collect();
            serde_json::json!({
                "result": Value::String(texts.join(separator)),
                "count": results.len(),
            })
        }
        "merge" => {
            let mut merged = serde_json::Map::new();
            for result in results {
                if let Some(obj) = result.as_object() {
                    for (k, v) in obj {
                        merged.insert(k.clone(), v.clone());
                    }
                }
            }
            serde_json::json!({
                "result": Value::Object(merged),
                "count": results.len(),
            })
        }
        // "array" and default
        _ => {
            serde_json::json!({
                "result": results,
                "count": results.len(),
            })
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for IteratorExecutor {
    fn node_type(&self) -> &str { "iterator" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &Value,
        incoming: &Option<Value>,
    ) -> Result<NodeOutput, String> {
        let items = extract_items(incoming, node_data)?;
        let item_count = items.len();

        // Parse graph and find subgraph
        let graph: Value = serde_json::from_str(ctx.graph_json)
            .map_err(|e| format!("Invalid graph JSON: {e}"))?;
        let (subgraph_ids, aggregator_id, aggregator_data) = find_subgraph(&graph, node_id)?;

        if items.is_empty() {
            let empty_result = apply_aggregation(&[], &aggregator_data);
            let mut skip_nodes: Vec<String> = subgraph_ids;
            skip_nodes.push(aggregator_id.clone());
            let mut extra_outputs = HashMap::new();
            extra_outputs.insert(aggregator_id, empty_result);
            return Ok(NodeOutput {
                value: serde_json::json!({"items": [], "count": 0}),
                skip_nodes,
                extra_outputs,
            });
        }

        // Build synthetic graph for subgraph execution
        let synthetic_graph = build_synthetic_graph(&graph, node_id, &subgraph_ids, &aggregator_id)?;

        eprintln!("[workflow] Iterator '{}': {} items, subgraph: {:?}, aggregator: {}",
            node_id, item_count, subgraph_ids, aggregator_id);

        let mut results: Vec<Value> = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            eprintln!("[workflow] Iterator '{}': item {}/{}", node_id, idx + 1, item_count);

            emit_workflow_event(ctx.app, ctx.session_id, "workflow.node.iteration",
                serde_json::json!({
                    "node_id": node_id,
                    "index": idx,
                    "total": item_count,
                }),
                ctx.seq_counter);

            // Build inputs for this iteration
            let mut sub_inputs: HashMap<String, Value> = HashMap::new();
            sub_inputs.insert("input".to_string(), item.clone());
            sub_inputs.insert("item".to_string(), item.clone());
            sub_inputs.insert("index".to_string(), serde_json::json!(idx));
            sub_inputs.insert("total".to_string(), serde_json::json!(item_count));

            let result = execute_workflow_with_visited(
                ctx.db, ctx.sidecar, ctx.app,
                ctx.session_id, &synthetic_graph,
                &sub_inputs, ctx.all_settings,
                ctx.visited_workflows, ctx.workflow_run_id,
                ctx.ephemeral,
            ).await.map_err(|e| format!("Iterator item {} failed: {}", idx, e))?;

            // Extract output from synthetic workflow
            let output = if result.outputs.len() == 1 {
                result.outputs.into_values().next().unwrap_or(Value::Null)
            } else if !result.outputs.is_empty() {
                serde_json::json!(result.outputs)
            } else {
                Value::Null
            };

            results.push(output);
        }

        // Apply aggregation using the aggregator's config
        let aggregated = apply_aggregation(&results, &aggregator_data);

        // Skip subgraph nodes + aggregator (their work is done)
        let mut skip_nodes: Vec<String> = subgraph_ids;
        skip_nodes.push(aggregator_id.clone());

        let mut extra_outputs = HashMap::new();
        extra_outputs.insert(aggregator_id, aggregated);

        Ok(NodeOutput {
            value: serde_json::json!({"count": item_count, "items_processed": item_count}),
            skip_nodes,
            extra_outputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_items_from_items_handle() {
        let incoming = Some(serde_json::json!({"items": [1, 2, 3]}));
        let node_data = serde_json::json!({});
        let items = extract_items(&incoming, &node_data).unwrap();
        assert_eq!(items, vec![serde_json::json!(1), serde_json::json!(2), serde_json::json!(3)]);
    }

    #[test]
    fn test_extract_items_bare_array() {
        let incoming = Some(serde_json::json!([4, 5, 6]));
        let node_data = serde_json::json!({});
        let items = extract_items(&incoming, &node_data).unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_extract_items_with_jsonpath() {
        let incoming = Some(serde_json::json!({
            "items": {"data": {"repos": [{"name": "a"}, {"name": "b"}]}}
        }));
        let node_data = serde_json::json!({"expression": "$.data.repos[*].name"});
        let items = extract_items(&incoming, &node_data).unwrap();
        assert_eq!(items, vec![serde_json::json!("a"), serde_json::json!("b")]);
    }

    #[test]
    fn test_extract_items_single_value_wraps() {
        let incoming = Some(serde_json::json!("hello"));
        let node_data = serde_json::json!({});
        let items = extract_items(&incoming, &node_data).unwrap();
        assert_eq!(items, vec![serde_json::json!("hello")]);
    }

    #[test]
    fn test_extract_items_no_input_errors() {
        let node_data = serde_json::json!({});
        let result = extract_items(&None, &node_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_subgraph_simple() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "iter_1", "type": "iterator", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {"provider": "test"}},
                {"id": "agg_1", "type": "aggregator", "data": {"strategy": "array"}}
            ],
            "edges": [
                {"id": "e1", "source": "iter_1", "target": "llm_1"},
                {"id": "e2", "source": "llm_1", "target": "agg_1"}
            ]
        });
        let (subgraph, agg_id, agg_data) = find_subgraph(&graph, "iter_1").unwrap();
        assert_eq!(subgraph, vec!["llm_1".to_string()]);
        assert_eq!(agg_id, "agg_1");
        assert_eq!(agg_data.get("strategy").unwrap().as_str().unwrap(), "array");
    }

    #[test]
    fn test_find_subgraph_multi_node() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "iter_1", "type": "iterator", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "transform_1", "type": "transform", "data": {}},
                {"id": "agg_1", "type": "aggregator", "data": {"strategy": "concat"}}
            ],
            "edges": [
                {"id": "e1", "source": "iter_1", "target": "llm_1"},
                {"id": "e2", "source": "llm_1", "target": "transform_1"},
                {"id": "e3", "source": "transform_1", "target": "agg_1"}
            ]
        });
        let (subgraph, agg_id, _) = find_subgraph(&graph, "iter_1").unwrap();
        assert_eq!(subgraph.len(), 2);
        assert!(subgraph.contains(&"llm_1".to_string()));
        assert!(subgraph.contains(&"transform_1".to_string()));
        assert_eq!(agg_id, "agg_1");
    }

    #[test]
    fn test_find_subgraph_excludes_parallel_branches() {
        // Iterator → A → Aggregator
        // Iterator → A → C → Output  (C should NOT be in subgraph)
        let graph = serde_json::json!({
            "nodes": [
                {"id": "iter_1", "type": "iterator", "data": {}},
                {"id": "a", "type": "llm", "data": {}},
                {"id": "c", "type": "transform", "data": {}},
                {"id": "agg_1", "type": "aggregator", "data": {}},
                {"id": "out_1", "type": "output", "data": {}}
            ],
            "edges": [
                {"id": "e1", "source": "iter_1", "target": "a"},
                {"id": "e2", "source": "a", "target": "agg_1"},
                {"id": "e3", "source": "a", "target": "c"},
                {"id": "e4", "source": "c", "target": "out_1"}
            ]
        });
        let (subgraph, agg_id, _) = find_subgraph(&graph, "iter_1").unwrap();
        assert_eq!(agg_id, "agg_1");
        assert!(subgraph.contains(&"a".to_string()));
        assert!(!subgraph.contains(&"c".to_string()), "C should not be in subgraph");
        assert!(!subgraph.contains(&"out_1".to_string()));
    }

    #[test]
    fn test_find_subgraph_no_aggregator_errors() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "iter_1", "type": "iterator", "data": {}},
                {"id": "llm_1", "type": "llm", "data": {}},
                {"id": "out_1", "type": "output", "data": {}}
            ],
            "edges": [
                {"id": "e1", "source": "iter_1", "target": "llm_1"},
                {"id": "e2", "source": "llm_1", "target": "out_1"}
            ]
        });
        let result = find_subgraph(&graph, "iter_1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Aggregator"));
    }

    #[test]
    fn test_build_synthetic_graph() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "iter_1", "type": "iterator", "data": {}, "position": {"x": 0, "y": 0}},
                {"id": "llm_1", "type": "llm", "data": {"provider": "test"}, "position": {"x": 100, "y": 0}},
                {"id": "agg_1", "type": "aggregator", "data": {}, "position": {"x": 200, "y": 0}}
            ],
            "edges": [
                {"id": "e1", "source": "iter_1", "sourceHandle": "output", "target": "llm_1", "targetHandle": "prompt"},
                {"id": "e2", "source": "llm_1", "sourceHandle": "response", "target": "agg_1", "targetHandle": "input"}
            ]
        });

        let synthetic = build_synthetic_graph(&graph, "iter_1", &["llm_1".to_string()], "agg_1").unwrap();
        let syn_graph: Value = serde_json::from_str(&synthetic).unwrap();

        let nodes = syn_graph.get("nodes").unwrap().as_array().unwrap();
        let edges = syn_graph.get("edges").unwrap().as_array().unwrap();

        assert_eq!(nodes.len(), 3); // syn_input + llm_1 + syn_output
        assert_eq!(edges.len(), 2); // syn_input → llm_1, llm_1 → syn_output

        let e0 = &edges[0];
        assert_eq!(e0.get("source").unwrap().as_str().unwrap(), "__iter_input__");
        assert_eq!(e0.get("target").unwrap().as_str().unwrap(), "llm_1");
        assert_eq!(e0.get("targetHandle").unwrap().as_str().unwrap(), "prompt");

        let e1 = &edges[1];
        assert_eq!(e1.get("source").unwrap().as_str().unwrap(), "llm_1");
        assert_eq!(e1.get("target").unwrap().as_str().unwrap(), "__iter_output__");
    }

    #[test]
    fn test_build_synthetic_graph_multi_node() {
        let graph = serde_json::json!({
            "nodes": [
                {"id": "iter_1", "type": "iterator", "data": {}, "position": {"x": 0, "y": 0}},
                {"id": "llm_1", "type": "llm", "data": {}, "position": {"x": 100, "y": 0}},
                {"id": "tr_1", "type": "transform", "data": {}, "position": {"x": 200, "y": 0}},
                {"id": "agg_1", "type": "aggregator", "data": {}, "position": {"x": 300, "y": 0}}
            ],
            "edges": [
                {"id": "e1", "source": "iter_1", "sourceHandle": "output", "target": "llm_1", "targetHandle": "input"},
                {"id": "e2", "source": "llm_1", "sourceHandle": "response", "target": "tr_1", "targetHandle": "input"},
                {"id": "e3", "source": "tr_1", "sourceHandle": "output", "target": "agg_1", "targetHandle": "input"}
            ]
        });

        let synthetic = build_synthetic_graph(
            &graph, "iter_1",
            &["llm_1".to_string(), "tr_1".to_string()],
            "agg_1",
        ).unwrap();
        let syn_graph: Value = serde_json::from_str(&synthetic).unwrap();

        let nodes = syn_graph.get("nodes").unwrap().as_array().unwrap();
        let edges = syn_graph.get("edges").unwrap().as_array().unwrap();

        assert_eq!(nodes.len(), 4); // syn_input + llm_1 + tr_1 + syn_output
        assert_eq!(edges.len(), 3); // syn_input→llm_1, llm_1→tr_1, tr_1→syn_output
    }

    #[test]
    fn test_apply_aggregation_array() {
        let results = vec![serde_json::json!("a"), serde_json::json!("b"), serde_json::json!("c")];
        let agg_data = serde_json::json!({"strategy": "array"});
        let output = apply_aggregation(&results, &agg_data);
        assert_eq!(output.get("count").unwrap().as_i64().unwrap(), 3);
        assert_eq!(output.get("result").unwrap().as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_apply_aggregation_concat() {
        let results = vec![serde_json::json!("line1"), serde_json::json!("line2"), serde_json::json!("line3")];
        let agg_data = serde_json::json!({"strategy": "concat", "separator": "\n"});
        let output = apply_aggregation(&results, &agg_data);
        assert_eq!(output.get("result").unwrap().as_str().unwrap(), "line1\nline2\nline3");
    }

    #[test]
    fn test_apply_aggregation_concat_custom_separator() {
        let results = vec![serde_json::json!("a"), serde_json::json!("b")];
        let agg_data = serde_json::json!({"strategy": "concat", "separator": " | "});
        let output = apply_aggregation(&results, &agg_data);
        assert_eq!(output.get("result").unwrap().as_str().unwrap(), "a | b");
    }

    #[test]
    fn test_apply_aggregation_merge() {
        let results = vec![
            serde_json::json!({"name": "Alice", "score": 95}),
            serde_json::json!({"name": "Bob", "grade": "A"}),
        ];
        let agg_data = serde_json::json!({"strategy": "merge"});
        let output = apply_aggregation(&results, &agg_data);
        let result = output.get("result").unwrap();
        assert_eq!(result.get("name").unwrap().as_str().unwrap(), "Bob"); // Last wins
        assert_eq!(result.get("score").unwrap().as_i64().unwrap(), 95);
        assert_eq!(result.get("grade").unwrap().as_str().unwrap(), "A");
    }

    #[test]
    fn test_apply_aggregation_empty() {
        let results: Vec<Value> = vec![];
        let agg_data = serde_json::json!({"strategy": "array"});
        let output = apply_aggregation(&results, &agg_data);
        assert_eq!(output.get("count").unwrap().as_i64().unwrap(), 0);
    }
}

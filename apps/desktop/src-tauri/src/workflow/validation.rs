use super::types::ValidationResult;

/// Validate a workflow graph. Pure function — no DB needed.
pub fn validate_graph_json(graph_json: &str) -> Result<ValidationResult, String> {
    let graph: serde_json::Value = serde_json::from_str(graph_json)
        .map_err(|e| format!("Invalid graph JSON: {e}"))?;

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let nodes = graph.get("nodes").and_then(|v| v.as_array());
    let edges = graph.get("edges").and_then(|v| v.as_array());

    let nodes = match nodes {
        Some(n) => n,
        None => {
            errors.push("Graph has no nodes array".to_string());
            return Ok(ValidationResult { valid: false, errors, warnings });
        }
    };

    if nodes.is_empty() {
        errors.push("Workflow has no nodes".to_string());
        return Ok(ValidationResult { valid: false, errors, warnings });
    }

    let edges = edges.cloned().unwrap_or_default();

    let mut node_ids: Vec<String> = Vec::new();
    let mut node_types: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut has_input = false;
    let mut has_output = false;

    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let ntype = node.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if ntype == "input" || ntype == "file_read" || ntype == "file_glob" || ntype == "iterator" || ntype == "loop" || ntype == "tool" || ntype == "http_request" || ntype == "shell_exec" { has_input = true; }
        if ntype == "output" || ntype == "file_write" || ntype == "aggregator" || ntype == "exit" { has_output = true; }
        node_ids.push(id.clone());
        node_types.insert(id, ntype);
    }

    if !has_input {
        errors.push("Workflow must have at least one Input node".to_string());
    }
    if !has_output {
        errors.push("Workflow must have at least one Output node".to_string());
    }

    // Build adjacency list for cycle detection
    let mut adj: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut in_degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut connected_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();

    for id in &node_ids {
        adj.entry(id.clone()).or_default();
        in_degree.entry(id.clone()).or_insert(0);
    }

    for edge in &edges {
        let source = edge.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let target = edge.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if !source.is_empty() && !target.is_empty() {
            adj.entry(source.clone()).or_default().push(target.clone());
            *in_degree.entry(target.clone()).or_insert(0) += 1;
            connected_nodes.insert(source);
            connected_nodes.insert(target);
        }
    }

    // Kahn's algorithm for cycle detection
    let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    for (id, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(id.clone());
        }
    }

    let mut visited_count = 0usize;
    while let Some(node) = queue.pop_front() {
        visited_count += 1;
        if let Some(neighbors) = adj.get(&node) {
            for n in neighbors {
                if let Some(d) = in_degree.get_mut(n) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(n.clone());
                    }
                }
            }
        }
    }

    if visited_count < node_ids.len() {
        errors.push("Workflow contains a cycle — execution would loop forever".to_string());
    }

    // Check for nested iterators (not yet supported — BFS subgraph extraction can't handle nesting)
    let iterator_count = node_types.values().filter(|t| t.as_str() == "iterator").count();
    if iterator_count > 1 {
        warnings.push("Multiple Iterator nodes detected — nested iteration is not yet supported and may produce unexpected results".to_string());
    }

    // Loop↔Exit pairing validation
    let loop_count = node_types.values().filter(|t| t.as_str() == "loop").count();
    let exit_count = node_types.values().filter(|t| t.as_str() == "exit").count();
    if loop_count > 0 && exit_count == 0 {
        warnings.push("Loop node has no paired Exit node — add an Exit node downstream to mark the loop boundary".to_string());
    }
    if exit_count > 0 && loop_count == 0 {
        warnings.push("Exit node found without a paired Loop node — Exit nodes should only be used inside a Loop".to_string());
    }

    // Nesting warnings: multiple loops or loop + iterator coexistence
    if loop_count > 1 {
        warnings.push("Multiple Loop nodes detected — nested loops are not yet supported and may produce unexpected results".to_string());
    }
    if loop_count > 0 && iterator_count > 0 {
        warnings.push("Loop and Iterator nodes in the same workflow — nesting loops inside iterators (or vice versa) is not yet supported".to_string());
    }

    // Check for orphan nodes
    for id in &node_ids {
        let ntype = node_types.get(id).map(|s| s.as_str()).unwrap_or("");
        if !connected_nodes.contains(id) && nodes.len() > 1 {
            if ntype == "input" || ntype == "output" {
                warnings.push(format!("Node '{}' ({}) has no connections", id, ntype));
            } else {
                warnings.push(format!("Orphan node '{}' ({}) — not connected to any edge", id, ntype));
            }
        }
    }

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph(nodes: &[(&str, &str)], edges: &[(&str, &str)]) -> String {
        let nodes_json: Vec<String> = nodes.iter().map(|(id, ntype)| {
            format!(r#"{{"id":"{}","type":"{}","position":{{"x":0,"y":0}},"data":{{}}}}"#, id, ntype)
        }).collect();
        let edges_json: Vec<String> = edges.iter().enumerate().map(|(i, (src, tgt))| {
            format!(r#"{{"id":"e{}","source":"{}","target":"{}"}}"#, i, src, tgt)
        }).collect();
        format!(r#"{{"nodes":[{}],"edges":[{}]}}"#, nodes_json.join(","), edges_json.join(","))
    }

    #[test]
    fn test_valid_simple_pipeline() {
        let graph = make_graph(
            &[("in1", "input"), ("llm1", "llm"), ("out1", "output")],
            &[("in1", "llm1"), ("llm1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid, "errors: {:?}", result.errors);
        assert!(result.warnings.is_empty(), "warnings: {:?}", result.warnings);
    }

    #[test]
    fn test_missing_input_node() {
        let graph = make_graph(
            &[("llm1", "llm"), ("out1", "output")],
            &[("llm1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Input node")));
    }

    #[test]
    fn test_missing_output_node() {
        let graph = make_graph(
            &[("in1", "input"), ("llm1", "llm")],
            &[("in1", "llm1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Output node")));
    }

    #[test]
    fn test_empty_workflow() {
        let graph = r#"{"nodes":[],"edges":[]}"#;
        let result = validate_graph_json(graph).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("no nodes")));
    }

    #[test]
    fn test_cycle_detection() {
        let graph = make_graph(
            &[("in1", "input"), ("a", "llm"), ("b", "transform"), ("out1", "output")],
            &[("in1", "a"), ("a", "b"), ("b", "a"), ("b", "out1")],
        );
        let result = validate_graph_json(graph.as_str()).unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("cycle")));
    }

    #[test]
    fn test_orphan_node_warning() {
        let graph = make_graph(
            &[("in1", "input"), ("llm1", "llm"), ("orphan", "transform"), ("out1", "output")],
            &[("in1", "llm1"), ("llm1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid, "should be valid despite orphan");
        assert!(result.warnings.iter().any(|w| w.contains("Orphan") || w.contains("orphan")));
    }

    #[test]
    fn test_complex_dag_valid() {
        let graph = make_graph(
            &[
                ("in1", "input"),
                ("llm1", "llm"),
                ("router1", "router"),
                ("tool1", "tool"),
                ("transform1", "transform"),
                ("out1", "output"),
                ("out2", "output"),
            ],
            &[
                ("in1", "llm1"),
                ("llm1", "router1"),
                ("router1", "tool1"),
                ("router1", "transform1"),
                ("tool1", "out1"),
                ("transform1", "out2"),
            ],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid, "errors: {:?}", result.errors);
    }

    #[test]
    fn test_nested_iterator_warning() {
        let graph = make_graph(
            &[
                ("in1", "input"), ("iter1", "iterator"), ("llm1", "llm"),
                ("iter2", "iterator"), ("llm2", "llm"), ("agg2", "aggregator"),
                ("agg1", "aggregator"), ("out1", "output"),
            ],
            &[
                ("in1", "iter1"), ("iter1", "iter2"), ("iter2", "llm2"),
                ("llm2", "agg2"), ("agg2", "llm1"), ("llm1", "agg1"), ("agg1", "out1"),
            ],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid, "should be valid");
        assert!(result.warnings.iter().any(|w| w.contains("nested iteration")),
            "warnings: {:?}", result.warnings);
    }

    #[test]
    fn test_invalid_json() {
        let result = validate_graph_json("not json at all");
        assert!(result.is_err() || !result.unwrap().valid);
    }

    #[test]
    fn test_loop_exit_valid() {
        let graph = make_graph(
            &[("in1", "input"), ("loop1", "loop"), ("llm1", "llm"), ("exit1", "exit"), ("out1", "output")],
            &[("in1", "loop1"), ("loop1", "llm1"), ("llm1", "exit1"), ("exit1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid, "errors: {:?}", result.errors);
        assert!(result.warnings.is_empty(), "warnings: {:?}", result.warnings);
    }

    #[test]
    fn test_loop_without_exit_warning() {
        let graph = make_graph(
            &[("in1", "input"), ("loop1", "loop"), ("llm1", "llm"), ("out1", "output")],
            &[("in1", "loop1"), ("loop1", "llm1"), ("llm1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("no paired Exit")),
            "warnings: {:?}", result.warnings);
    }

    #[test]
    fn test_exit_without_loop_warning() {
        let graph = make_graph(
            &[("in1", "input"), ("llm1", "llm"), ("exit1", "exit"), ("out1", "output")],
            &[("in1", "llm1"), ("llm1", "exit1"), ("exit1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("without a paired Loop")),
            "warnings: {:?}", result.warnings);
    }

    #[test]
    fn test_multiple_loops_warning() {
        let graph = make_graph(
            &[("in1", "input"), ("loop1", "loop"), ("loop2", "loop"), ("llm1", "llm"),
              ("exit1", "exit"), ("exit2", "exit"), ("out1", "output")],
            &[("in1", "loop1"), ("loop1", "llm1"), ("llm1", "exit1"),
              ("exit1", "loop2"), ("loop2", "exit2"), ("exit2", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("Multiple Loop")),
            "warnings: {:?}", result.warnings);
    }

    #[test]
    fn test_loop_iterator_coexistence_warning() {
        let graph = make_graph(
            &[("in1", "input"), ("loop1", "loop"), ("llm1", "llm"), ("exit1", "exit"),
              ("iter1", "iterator"), ("llm2", "llm"), ("agg1", "aggregator"), ("out1", "output")],
            &[("in1", "loop1"), ("loop1", "llm1"), ("llm1", "exit1"),
              ("exit1", "iter1"), ("iter1", "llm2"), ("llm2", "agg1"), ("agg1", "out1")],
        );
        let result = validate_graph_json(&graph).unwrap();
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("Loop and Iterator")),
            "warnings: {:?}", result.warnings);
    }
}

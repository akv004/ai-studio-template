use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;
use serde_json::Value;

pub struct TransformExecutor;

#[async_trait::async_trait]
impl NodeExecutor for TransformExecutor {
    fn node_type(&self) -> &str { "transform" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &Value,
        incoming: &Option<Value>,
    ) -> Result<NodeOutput, String> {
        let mode = node_data.get("mode").and_then(|v| v.as_str()).unwrap_or("template");
        let template = node_data.get("template").and_then(|v| v.as_str()).unwrap_or("{{input}}");

        let mut local_inputs = ctx.inputs.clone();
        if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                for (k, v) in obj {
                    local_inputs.insert(k.clone(), v.clone());
                }
            } else {
                local_inputs.insert("input".to_string(), inc.clone());
            }
        }

        match mode {
            "jsonpath" => execute_jsonpath(template, incoming, &local_inputs),
            "script" => execute_script(template, incoming, &local_inputs),
            _ => execute_template(template, ctx, &local_inputs, incoming),
        }
    }
}

fn execute_template(
    template: &str,
    ctx: &ExecutionContext<'_>,
    local_inputs: &std::collections::HashMap<String, Value>,
    incoming: &Option<Value>,
) -> Result<NodeOutput, String> {
    if template.contains("{{") {
        let result = resolve_template(template, ctx.node_outputs, local_inputs);
        return Ok(NodeOutput::value(Value::String(result)));
    }
    Ok(NodeOutput::value(incoming.clone().unwrap_or(Value::Null)))
}

fn execute_jsonpath(
    expression: &str,
    incoming: &Option<Value>,
    local_inputs: &std::collections::HashMap<String, Value>,
) -> Result<NodeOutput, String> {
    let path: serde_json_path::JsonPath = expression.parse()
        .map_err(|e| format!("Invalid JSONPath '{}': {}", expression, e))?;

    // Build the document to query: incoming data merged with inputs
    let doc = build_query_document(incoming, local_inputs);

    let node_list = path.query(&doc);
    let matches: Vec<&Value> = node_list.all();

    let result = match matches.len() {
        0 => Value::Null,
        1 => matches[0].clone(),
        _ => Value::Array(matches.into_iter().cloned().collect()),
    };

    Ok(NodeOutput::value(result))
}

fn execute_script(
    expression: &str,
    incoming: &Option<Value>,
    local_inputs: &std::collections::HashMap<String, Value>,
) -> Result<NodeOutput, String> {
    let expr = expression.trim();

    // Build context from incoming + inputs
    let doc = build_query_document(incoming, local_inputs);

    // Built-in operations: .field, [index], | length, | keys, | values, | first, | last, | flatten, | join(sep)
    let result = evaluate_expression(expr, &doc)?;
    Ok(NodeOutput::value(result))
}

fn build_query_document(
    incoming: &Option<Value>,
    local_inputs: &std::collections::HashMap<String, Value>,
) -> Value {
    let mut doc = serde_json::Map::new();

    // Add all local inputs
    for (k, v) in local_inputs {
        doc.insert(k.clone(), v.clone());
    }

    // Merge incoming data
    if let Some(inc) = incoming {
        if let Some(obj) = inc.as_object() {
            for (k, v) in obj {
                doc.insert(k.clone(), v.clone());
            }
        } else {
            doc.insert("input".to_string(), inc.clone());
        }
    }

    Value::Object(doc)
}

fn evaluate_expression(expr: &str, doc: &Value) -> Result<Value, String> {
    // Pipe chain: "$.data | length" or "$.items | first | keys"
    let parts: Vec<&str> = expr.splitn(2, '|').collect();
    let (source_expr, pipe_op) = if parts.len() == 2 {
        (parts[0].trim(), Some(parts[1].trim()))
    } else {
        (expr, None)
    };

    // Resolve source value
    let source = if source_expr.starts_with('$') {
        // JSONPath query on the document
        let path: serde_json_path::JsonPath = source_expr.parse()
            .map_err(|e| format!("Invalid path '{}': {}", source_expr, e))?;
        let matches: Vec<&Value> = path.query(doc).all();
        match matches.len() {
            0 => Value::Null,
            1 => matches[0].clone(),
            _ => Value::Array(matches.into_iter().cloned().collect()),
        }
    } else if source_expr == "input" || source_expr == "." {
        doc.clone()
    } else {
        // Try as field name
        doc.get(source_expr).cloned().unwrap_or(Value::Null)
    };

    // Apply pipe operation if present
    match pipe_op {
        None => Ok(source),
        Some(op) => apply_pipe(op, &source),
    }
}

fn apply_pipe(op_chain: &str, value: &Value) -> Result<Value, String> {
    // Handle chained pipes: "first | keys"
    let parts: Vec<&str> = op_chain.splitn(2, '|').collect();
    let (op, rest) = (parts[0].trim(), parts.get(1).map(|s| s.trim()));

    let result = apply_single_pipe(op, value)?;

    match rest {
        Some(next_op) => apply_pipe(next_op, &result),
        None => Ok(result),
    }
}

fn apply_single_pipe(op: &str, value: &Value) -> Result<Value, String> {
    match op {
        "length" => match value {
            Value::Array(arr) => Ok(Value::from(arr.len())),
            Value::String(s) => Ok(Value::from(s.len())),
            Value::Object(obj) => Ok(Value::from(obj.len())),
            _ => Ok(Value::from(0)),
        },
        "keys" => match value {
            Value::Object(obj) => Ok(Value::Array(
                obj.keys().map(|k| Value::String(k.clone())).collect()
            )),
            _ => Err("keys requires an object".to_string()),
        },
        "values" => match value {
            Value::Object(obj) => Ok(Value::Array(obj.values().cloned().collect())),
            _ => Err("values requires an object".to_string()),
        },
        "first" => match value {
            Value::Array(arr) => Ok(arr.first().cloned().unwrap_or(Value::Null)),
            _ => Ok(value.clone()),
        },
        "last" => match value {
            Value::Array(arr) => Ok(arr.last().cloned().unwrap_or(Value::Null)),
            _ => Ok(value.clone()),
        },
        "flatten" => match value {
            Value::Array(arr) => {
                let mut flat = Vec::new();
                for item in arr {
                    if let Value::Array(inner) = item {
                        flat.extend(inner.iter().cloned());
                    } else {
                        flat.push(item.clone());
                    }
                }
                Ok(Value::Array(flat))
            },
            _ => Ok(value.clone()),
        },
        "sort" => match value {
            Value::Array(arr) => {
                let mut sorted = arr.clone();
                sorted.sort_by(|a, b| {
                    let a_str = a.as_str().unwrap_or("");
                    let b_str = b.as_str().unwrap_or("");
                    a_str.cmp(b_str)
                });
                Ok(Value::Array(sorted))
            },
            _ => Ok(value.clone()),
        },
        "reverse" => match value {
            Value::Array(arr) => {
                let mut rev = arr.clone();
                rev.reverse();
                Ok(Value::Array(rev))
            },
            _ => Ok(value.clone()),
        },
        "unique" => match value {
            Value::Array(arr) => {
                let mut seen = std::collections::HashSet::new();
                let mut unique = Vec::new();
                for item in arr {
                    let key = item.to_string();
                    if seen.insert(key) {
                        unique.push(item.clone());
                    }
                }
                Ok(Value::Array(unique))
            },
            _ => Ok(value.clone()),
        },
        "to_string" => Ok(Value::String(match value {
            Value::String(s) => s.clone(),
            _ => value.to_string(),
        })),
        "from_json" => match value {
            Value::String(s) => serde_json::from_str(s)
                .map_err(|e| format!("Invalid JSON: {}", e)),
            _ => Ok(value.clone()),
        },
        _ => {
            // Check for parameterized ops: join(sep), map(field), select(field, value)
            if let Some(inner) = extract_param(op, "join") {
                let sep = inner.trim_matches(|c| c == '"' || c == '\'');
                match value {
                    Value::Array(arr) => {
                        let strings: Vec<String> = arr.iter().map(|v| match v {
                            Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        }).collect();
                        Ok(Value::String(strings.join(sep)))
                    },
                    _ => Ok(value.clone()),
                }
            } else if let Some(inner) = extract_param(op, "map") {
                let field = inner.trim();
                match value {
                    Value::Array(arr) => Ok(Value::Array(
                        arr.iter().filter_map(|item| item.get(field).cloned()).collect()
                    )),
                    _ => Ok(value.clone()),
                }
            } else if let Some(inner) = extract_param(op, "select") {
                // select(field=value) or select(field,"value")
                let parts: Vec<&str> = inner.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let field = parts[0].trim();
                    let target = parts[1].trim().trim_matches(|c| c == '"' || c == '\'');
                    match value {
                        Value::Array(arr) => Ok(Value::Array(
                            arr.iter().filter(|item| {
                                item.get(field)
                                    .and_then(|v| v.as_str())
                                    .map(|s| s == target)
                                    .unwrap_or(false)
                            }).cloned().collect()
                        )),
                        _ => Ok(value.clone()),
                    }
                } else {
                    Err(format!("select requires field=value: select({})", inner))
                }
            } else if let Some(inner) = extract_param(op, "take") {
                let n: usize = inner.trim().parse()
                    .map_err(|_| format!("take requires a number: take({})", inner))?;
                match value {
                    Value::Array(arr) => Ok(Value::Array(arr.iter().take(n).cloned().collect())),
                    _ => Ok(value.clone()),
                }
            } else if let Some(inner) = extract_param(op, "skip") {
                let n: usize = inner.trim().parse()
                    .map_err(|_| format!("skip requires a number: skip({})", inner))?;
                match value {
                    Value::Array(arr) => Ok(Value::Array(arr.iter().skip(n).cloned().collect())),
                    _ => Ok(value.clone()),
                }
            } else {
                Err(format!("Unknown pipe operation: {}", op))
            }
        }
    }
}

fn extract_param<'a>(op: &'a str, name: &str) -> Option<&'a str> {
    if op.starts_with(name) && op.contains('(') && op.ends_with(')') {
        let start = op.find('(')? + 1;
        let end = op.len() - 1;
        Some(&op[start..end])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // --- JSONPath mode tests ---

    #[test]
    fn test_jsonpath_extract_field() {
        let incoming = Some(serde_json::json!({
            "body": "{\"name\": \"ai-studio\", \"stars\": 100}",
            "status": 200
        }));
        let inputs = HashMap::new();
        let result = execute_jsonpath("$.status", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(200));
    }

    #[test]
    fn test_jsonpath_extract_nested() {
        let incoming = Some(serde_json::json!({
            "data": {"items": [{"name": "v1.0"}, {"name": "v2.0"}, {"name": "v3.0"}]}
        }));
        let inputs = HashMap::new();
        let result = execute_jsonpath("$.data.items[*].name", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(["v1.0", "v2.0", "v3.0"]));
    }

    #[test]
    fn test_jsonpath_single_match() {
        let incoming = Some(serde_json::json!({
            "tags": [{"name": "v1.0", "sha": "abc"}, {"name": "v2.0", "sha": "def"}]
        }));
        let inputs = HashMap::new();
        let result = execute_jsonpath("$.tags[0].name", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!("v1.0"));
    }

    #[test]
    fn test_jsonpath_no_match() {
        let incoming = Some(serde_json::json!({"foo": "bar"}));
        let inputs = HashMap::new();
        let result = execute_jsonpath("$.nonexistent", &incoming, &inputs).unwrap();
        assert_eq!(result.value, Value::Null);
    }

    #[test]
    fn test_jsonpath_with_inputs() {
        let incoming: Option<Value> = None;
        let mut inputs = HashMap::new();
        inputs.insert("repos".to_string(), serde_json::json!([
            {"name": "ai-studio", "lang": "rust"},
            {"name": "ghoststag", "lang": "python"}
        ]));
        let result = execute_jsonpath("$.repos[*].name", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(["ai-studio", "ghoststag"]));
    }

    #[test]
    fn test_jsonpath_invalid_expression() {
        let incoming = Some(serde_json::json!({"a": 1}));
        let inputs = HashMap::new();
        let result = execute_jsonpath("$[invalid", &incoming, &inputs);
        assert!(result.is_err());
    }

    // --- Script mode tests ---

    #[test]
    fn test_script_field_access() {
        let incoming = Some(serde_json::json!({"name": "ai-studio", "stars": 100}));
        let inputs = HashMap::new();
        let result = execute_script("$.name", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!("ai-studio"));
    }

    #[test]
    fn test_script_pipe_length() {
        let incoming = Some(serde_json::json!({
            "items": ["a", "b", "c", "d"]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.items | length", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(4));
    }

    #[test]
    fn test_script_pipe_first() {
        let incoming = Some(serde_json::json!({
            "tags": [{"name": "v3.0"}, {"name": "v2.0"}, {"name": "v1.0"}]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.tags | first", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!({"name": "v3.0"}));
    }

    #[test]
    fn test_script_pipe_last() {
        let incoming = Some(serde_json::json!({
            "items": [1, 2, 3]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.items | last", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(3));
    }

    #[test]
    fn test_script_pipe_map() {
        let incoming = Some(serde_json::json!({
            "repos": [
                {"name": "ai-studio", "stars": 100},
                {"name": "ghoststag", "stars": 50}
            ]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.repos | map(name)", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(["ai-studio", "ghoststag"]));
    }

    #[test]
    fn test_script_pipe_select() {
        let incoming = Some(serde_json::json!({
            "items": [
                {"status": "active", "name": "a"},
                {"status": "archived", "name": "b"},
                {"status": "active", "name": "c"}
            ]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.items | select(status=active)", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!([
            {"status": "active", "name": "a"},
            {"status": "active", "name": "c"}
        ]));
    }

    #[test]
    fn test_script_pipe_join() {
        let incoming = Some(serde_json::json!({
            "names": ["Alice", "Bob", "Charlie"]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.names | join(\", \")", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!("Alice, Bob, Charlie"));
    }

    #[test]
    fn test_script_pipe_chain() {
        let incoming = Some(serde_json::json!({
            "repos": [
                {"name": "ai-studio", "lang": "rust"},
                {"name": "ghoststag", "lang": "python"},
                {"name": "snowowl", "lang": "rust"}
            ]
        }));
        let inputs = HashMap::new();
        let result = execute_script(
            "$.repos | select(lang=rust) | map(name) | join(\", \")",
            &incoming, &inputs
        ).unwrap();
        assert_eq!(result.value, serde_json::json!("ai-studio, snowowl"));
    }

    #[test]
    fn test_script_pipe_sort_reverse() {
        let incoming = Some(serde_json::json!({
            "tags": ["v1.0", "v3.0", "v2.0"]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.tags | sort | reverse", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(["v3.0", "v2.0", "v1.0"]));
    }

    #[test]
    fn test_script_pipe_take_skip() {
        let incoming = Some(serde_json::json!({
            "items": [1, 2, 3, 4, 5]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.items | skip(1) | take(3)", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!([2, 3, 4]));
    }

    #[test]
    fn test_script_pipe_unique() {
        let incoming = Some(serde_json::json!({
            "items": ["a", "b", "a", "c", "b"]
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.items | unique", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(["a", "b", "c"]));
    }

    #[test]
    fn test_script_pipe_keys_values() {
        let incoming = Some(serde_json::json!({
            "config": {"host": "localhost", "port": "8080"}
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.config | keys", &incoming, &inputs).unwrap();
        let keys = result.value.as_array().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&serde_json::json!("host")));
        assert!(keys.contains(&serde_json::json!("port")));
    }

    #[test]
    fn test_script_from_json() {
        let incoming = Some(serde_json::json!({
            "body": "[{\"name\":\"v1.0\"},{\"name\":\"v2.0\"}]"
        }));
        let inputs = HashMap::new();
        let result = execute_script("$.body | from_json | map(name)", &incoming, &inputs).unwrap();
        assert_eq!(result.value, serde_json::json!(["v1.0", "v2.0"]));
    }

    #[test]
    fn test_script_dot_input() {
        let incoming = Some(serde_json::json!({"a": 1, "b": 2}));
        let inputs = HashMap::new();
        let result = execute_script(". | keys | length", &incoming, &inputs).unwrap();
        // doc has "a", "b", and "input" (or just "a", "b" from incoming object)
        assert!(result.value.as_u64().unwrap() >= 2);
    }

    // --- GitHub automation scenario test ---

    #[test]
    fn test_github_tags_extraction() {
        // Simulate: HTTP Request returns GitHub API tags response as body string
        let api_response = serde_json::json!({
            "body": "[{\"name\":\"v0.1.0\",\"commit\":{\"sha\":\"abc123\"}},{\"name\":\"v0.0.9\",\"commit\":{\"sha\":\"def456\"}},{\"name\":\"v0.0.8\",\"commit\":{\"sha\":\"ghi789\"}}]",
            "status": 200
        });

        let inputs = HashMap::new();

        // Step 1: Parse JSON body and extract tag names
        let result = execute_script(
            "$.body | from_json | map(name)",
            &Some(api_response.clone()), &inputs
        ).unwrap();
        assert_eq!(result.value, serde_json::json!(["v0.1.0", "v0.0.9", "v0.0.8"]));

        // Step 2: Get latest tag
        let result = execute_script(
            "$.body | from_json | first",
            &Some(api_response.clone()), &inputs
        ).unwrap();
        assert_eq!(result.value.get("name").unwrap(), "v0.1.0");

        // Step 3: Format as comma-separated list
        let result = execute_script(
            "$.body | from_json | map(name) | join(\", \")",
            &Some(api_response), &inputs
        ).unwrap();
        assert_eq!(result.value, serde_json::json!("v0.1.0, v0.0.9, v0.0.8"));
    }

    // --- Helper function tests ---

    #[test]
    fn test_extract_param() {
        assert_eq!(extract_param("join(\", \")", "join"), Some("\", \""));
        assert_eq!(extract_param("map(name)", "map"), Some("name"));
        assert_eq!(extract_param("take(5)", "take"), Some("5"));
        assert_eq!(extract_param("length", "join"), None);
        assert_eq!(extract_param("join_stuff", "join"), None);
    }

    #[test]
    fn test_build_query_document_merges() {
        let incoming = Some(serde_json::json!({"body": "hello", "status": 200}));
        let mut inputs = HashMap::new();
        inputs.insert("repo".to_string(), serde_json::json!("ai-studio"));
        let doc = build_query_document(&incoming, &inputs);
        assert_eq!(doc.get("body").unwrap(), "hello");
        assert_eq!(doc.get("status").unwrap(), 200);
        assert_eq!(doc.get("repo").unwrap(), "ai-studio");
    }

    #[test]
    fn test_build_query_document_scalar_incoming() {
        let incoming = Some(serde_json::json!("raw string"));
        let inputs = HashMap::new();
        let doc = build_query_document(&incoming, &inputs);
        assert_eq!(doc.get("input").unwrap(), "raw string");
    }
}

use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;
use crate::workflow::executors::file_read::expand_tilde;

/// Same denied paths as file_read
fn is_path_denied(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    for denied in &["/etc/shadow", "/etc/passwd"] {
        if path_str.as_ref() == *denied {
            return true;
        }
    }
    for component in path.components() {
        let s = component.as_os_str().to_string_lossy();
        for denied in &[".ssh", ".gnupg", ".config/ai-studio"] {
            if s.as_ref() == *denied {
                return true;
            }
        }
    }
    false
}

pub struct FileWriteExecutor;

#[async_trait::async_trait]
impl NodeExecutor for FileWriteExecutor {
    fn node_type(&self) -> &str { "file_write" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        // Resolve path
        let config_path = node_data.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let path_str = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("path").and_then(|v| v.as_str()).unwrap_or(config_path).to_string()
            } else {
                config_path.to_string()
            }
        } else {
            config_path.to_string()
        };
        let path_str = resolve_template(&path_str, ctx.node_outputs, ctx.inputs);
        let path_str = expand_tilde(&path_str);

        if path_str.is_empty() {
            return Err("File Write: path is empty".into());
        }

        let path = std::path::Path::new(&path_str);

        // Security check on the parent dir (file may not exist yet)
        if let Some(parent) = path.parent() {
            if parent.exists() {
                let canonical_parent = parent.canonicalize()
                    .map_err(|e| format!("Cannot resolve parent directory: {}", e))?;
                if is_path_denied(&canonical_parent) {
                    return Err(format!("File Write: access denied to sensitive path '{}'", path_str));
                }
            }
        }
        // Also check the target path components
        if is_path_denied(path) {
            return Err(format!("File Write: access denied to sensitive path '{}'", path_str));
        }

        // Resolve content from incoming edge
        let content_value = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("content").cloned().unwrap_or(serde_json::Value::Null)
            } else {
                inc.clone()
            }
        } else {
            return Err("File Write: no content provided".into());
        };

        if content_value.is_null() {
            return Err("File Write: content is null".into());
        }

        let mode = node_data.get("mode").and_then(|v| v.as_str()).unwrap_or("text");
        let write_mode = node_data.get("writeMode").and_then(|v| v.as_str()).unwrap_or("overwrite");
        let create_dirs = node_data.get("createDirs").and_then(|v| v.as_bool()).unwrap_or(true);

        // Create parent directories if needed
        if create_dirs {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directories: {}", e))?;
            }
        }

        // Convert content to string based on mode
        let content_str = match mode {
            "json" => {
                let pretty = node_data.get("jsonPretty").and_then(|v| v.as_bool()).unwrap_or(true);
                if pretty {
                    serde_json::to_string_pretty(&content_value)
                        .map_err(|e| format!("JSON serialization error: {}", e))?
                } else {
                    serde_json::to_string(&content_value)
                        .map_err(|e| format!("JSON serialization error: {}", e))?
                }
            }
            "csv" => {
                let delimiter = node_data.get("csvDelimiter")
                    .and_then(|v| v.as_str())
                    .unwrap_or(",")
                    .chars().next().unwrap_or(',');
                json_to_csv(&content_value, delimiter)?
            }
            _ => {
                // text mode
                match content_value.as_str() {
                    Some(s) => s.to_string(),
                    None => content_value.to_string(),
                }
            }
        };

        // Write file
        let bytes_written = content_str.len();
        match write_mode {
            "append" => {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|e| format!("Failed to open file for append: {}", e))?;
                file.write_all(content_str.as_bytes())
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
            _ => {
                std::fs::write(path, &content_str)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
        }

        Ok(NodeOutput::value(serde_json::json!({
            "path": path_str,
            "bytes": bytes_written,
        })))
    }
}

/// Convert JSON array of objects to CSV string
fn json_to_csv(value: &serde_json::Value, delimiter: char) -> Result<String, String> {
    let arr = value.as_array()
        .ok_or("CSV mode requires array input (rows of objects)")?;

    if arr.is_empty() {
        return Ok(String::new());
    }

    // Extract headers from first object
    let headers: Vec<String> = if let Some(first) = arr[0].as_object() {
        first.keys().cloned().collect()
    } else {
        return Err("CSV rows must be objects".into());
    };

    let mut csv = String::new();
    csv.push_str(&headers.join(&delimiter.to_string()));
    csv.push('\n');

    for row in arr {
        if let Some(obj) = row.as_object() {
            let fields: Vec<String> = headers.iter().map(|h| {
                let val = obj.get(h).unwrap_or(&serde_json::Value::Null);
                let s = match val.as_str() {
                    Some(s) => s.to_string(),
                    None => val.to_string(),
                };
                // Quote if contains delimiter or newline
                if s.contains(delimiter) || s.contains('\n') || s.contains('"') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s
                }
            }).collect();
            csv.push_str(&fields.join(&delimiter.to_string()));
            csv.push('\n');
        }
    }

    Ok(csv)
}

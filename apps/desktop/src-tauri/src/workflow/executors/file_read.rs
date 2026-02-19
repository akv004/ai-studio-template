use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;

/// Paths that are always denied for security reasons
const DENIED_PATHS: &[&str] = &[
    ".ssh", ".gnupg", ".config/ai-studio",
];
const DENIED_FILES: &[&str] = &[
    "/etc/shadow", "/etc/passwd",
];

pub fn is_path_denied(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    for denied in DENIED_FILES {
        if path_str.as_ref() == *denied {
            return true;
        }
    }
    // Check if any component matches denied directory names
    for component in path.components() {
        let s = component.as_os_str().to_string_lossy();
        for denied in DENIED_PATHS {
            if s.as_ref() == *denied {
                return true;
            }
        }
    }
    false
}

pub struct FileReadExecutor;

#[async_trait::async_trait]
impl NodeExecutor for FileReadExecutor {
    fn node_type(&self) -> &str { "file_read" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        // Resolve path: incoming "path" edge > config path
        let config_path = node_data.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let path_str = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("path").and_then(|v| v.as_str()).unwrap_or(config_path).to_string()
            } else if let Some(s) = inc.as_str() {
                s.to_string()
            } else {
                config_path.to_string()
            }
        } else {
            config_path.to_string()
        };
        let path_str = resolve_template(&path_str, ctx.node_outputs, ctx.inputs);

        if path_str.is_empty() {
            return Err("File Read: path is empty".into());
        }

        let path = std::path::Path::new(&path_str);

        // Canonicalize and check security
        let canonical = path.canonicalize()
            .map_err(|e| format!("File not found or inaccessible: {} ({})", path_str, e))?;

        if is_path_denied(&canonical) {
            return Err(format!("File Read: access denied to sensitive path '{}'", path_str));
        }

        let mode = node_data.get("mode").and_then(|v| v.as_str()).unwrap_or("text");
        let max_size_mb = node_data.get("maxSize").and_then(|v| v.as_f64()).unwrap_or(10.0);
        let max_size_bytes = (max_size_mb * 1_048_576.0) as u64;

        // Check file size
        let metadata = std::fs::metadata(&canonical)
            .map_err(|e| format!("Cannot read file metadata: {}", e))?;
        let file_size = metadata.len();
        if file_size > max_size_bytes {
            return Err(format!(
                "File too large: {:.1}MB > {:.0}MB limit",
                file_size as f64 / 1_048_576.0,
                max_size_mb
            ));
        }

        match mode {
            "json" => {
                let content = std::fs::read_to_string(&canonical)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                let parsed: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| format!("Invalid JSON in file: {}", e))?;
                Ok(NodeOutput::value(serde_json::json!({
                    "content": parsed,
                    "size": file_size,
                })))
            }
            "csv" => {
                let content = std::fs::read_to_string(&canonical)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                let delimiter = node_data.get("csvDelimiter")
                    .and_then(|v| v.as_str())
                    .unwrap_or(",")
                    .chars().next().unwrap_or(',');
                let has_header = node_data.get("csvHasHeader")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let rows = parse_csv(&content, delimiter, has_header)?;
                Ok(NodeOutput::value(serde_json::json!({
                    "rows": rows,
                    "content": content,
                    "size": file_size,
                })))
            }
            "binary" => {
                let bytes = std::fs::read(&canonical)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let mime = guess_mime_type(&canonical);
                Ok(NodeOutput::value(serde_json::json!({
                    "content": encoded,
                    "encoding": "base64",
                    "mime_type": mime,
                    "size": file_size,
                })))
            }
            _ => {
                // text mode (default)
                let content = std::fs::read_to_string(&canonical)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                Ok(NodeOutput::value(serde_json::json!({
                    "content": content,
                    "size": file_size,
                })))
            }
        }
    }
}

/// Guess MIME type from file extension
pub fn guess_mime_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "pdf" => "application/pdf",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    }
}

/// Simple CSV parser — handles quoted fields, returns array of objects
pub fn parse_csv(content: &str, delimiter: char, has_header: bool) -> Result<Vec<serde_json::Value>, String> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Ok(vec![]);
    }

    let headers = if has_header {
        parse_csv_line(lines[0], delimiter)
    } else {
        (0..parse_csv_line(lines[0], delimiter).len())
            .map(|i| format!("col_{}", i))
            .collect()
    };

    let data_start = if has_header { 1 } else { 0 };
    let mut rows = Vec::new();

    for line in &lines[data_start..] {
        if line.trim().is_empty() { continue; }
        let fields = parse_csv_line(line, delimiter);
        let mut obj = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            let val = fields.get(i).cloned().unwrap_or_default();
            obj.insert(header.clone(), serde_json::Value::String(val));
        }
        rows.push(serde_json::Value::Object(obj));
    }

    Ok(rows)
}

/// Parse a single CSV line, handling quoted fields
fn parse_csv_line(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '"' {
            if in_quotes {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                in_quotes = true;
            }
        } else if c == delimiter && !in_quotes {
            fields.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(c);
        }
    }
    fields.push(current.trim().to_string());
    fields
}

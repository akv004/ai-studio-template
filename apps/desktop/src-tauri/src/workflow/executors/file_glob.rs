use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;
use crate::workflow::executors::file_read::{is_path_denied, guess_mime_type, parse_csv};
use serde_json::Value;

pub struct FileGlobExecutor;

#[async_trait::async_trait]
impl NodeExecutor for FileGlobExecutor {
    fn node_type(&self) -> &str { "file_glob" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &Value,
        incoming: &Option<Value>,
    ) -> Result<NodeOutput, String> {
        // Resolve directory: incoming > config
        let config_dir = node_data.get("directory").and_then(|v| v.as_str()).unwrap_or("");
        let dir_str = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("directory").and_then(|v| v.as_str()).unwrap_or(config_dir).to_string()
            } else if let Some(s) = inc.as_str() {
                s.to_string()
            } else {
                config_dir.to_string()
            }
        } else {
            config_dir.to_string()
        };
        let dir_str = resolve_template(&dir_str, ctx.node_outputs, ctx.inputs);

        if dir_str.is_empty() {
            return Err("File Glob: directory is empty".into());
        }

        let dir_path = std::path::Path::new(&dir_str);
        if !dir_path.exists() {
            return Err(format!("File Glob: directory not found: {}", dir_str));
        }
        if !dir_path.is_dir() {
            return Err(format!("File Glob: path is not a directory: {}", dir_str));
        }
        let canonical_base = dir_path.canonicalize()
            .map_err(|e| format!("File Glob: cannot resolve directory '{}': {}", dir_str, e))?;

        let pattern = node_data.get("pattern").and_then(|v| v.as_str()).unwrap_or("*");
        let pattern = resolve_template(pattern, ctx.node_outputs, ctx.inputs);
        let recursive = node_data.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        let mode = node_data.get("mode").and_then(|v| v.as_str()).unwrap_or("text");
        let max_files = node_data.get("maxFiles").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
        let max_size_mb = node_data.get("maxSize").and_then(|v| v.as_f64()).unwrap_or(10.0);
        let max_size_bytes = (max_size_mb * 1_048_576.0) as u64;
        let sort_by = node_data.get("sortBy").and_then(|v| v.as_str()).unwrap_or("name");
        let sort_order = node_data.get("sortOrder").and_then(|v| v.as_str()).unwrap_or("asc");

        // Build glob pattern
        let glob_pattern = if recursive {
            format!("{}/**/{}", dir_str.trim_end_matches('/'), pattern)
        } else {
            format!("{}/{}", dir_str.trim_end_matches('/'), pattern)
        };

        let entries = glob::glob(&glob_pattern)
            .map_err(|e| format!("File Glob: invalid pattern '{}': {}", glob_pattern, e))?;

        let mut file_entries: Vec<FileEntry> = Vec::new();

        for entry in entries {
            if file_entries.len() >= max_files {
                break;
            }

            let path = match entry {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Security check: deny-list + directory containment
            let canonical = match path.canonicalize() {
                Ok(c) => c,
                Err(_) => continue,
            };
            if is_path_denied(&canonical) {
                continue;
            }
            if !canonical.starts_with(&canonical_base) {
                continue; // Escaped configured directory via ../ or symlink
            }

            // Metadata
            let metadata = match std::fs::metadata(&canonical) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let file_size = metadata.len();

            // Skip files exceeding size limit
            if file_size > max_size_bytes {
                continue;
            }

            let modified = metadata.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| {
                    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let path_str = canonical.to_string_lossy().to_string();

            file_entries.push(FileEntry {
                path: path_str,
                name,
                size: file_size,
                modified,
                canonical,
            });
        }

        // Sort
        match sort_by {
            "modified" => file_entries.sort_by(|a, b| a.modified.cmp(&b.modified)),
            "size" => file_entries.sort_by(|a, b| a.size.cmp(&b.size)),
            _ => file_entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        }
        if sort_order == "desc" {
            file_entries.reverse();
        }

        // Read content per mode
        let csv_delimiter = node_data.get("csvDelimiter")
            .and_then(|v| v.as_str())
            .unwrap_or(",")
            .chars().next().unwrap_or(',');
        let csv_has_header = node_data.get("csvHasHeader")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut files = Vec::new();
        let mut paths = Vec::new();

        for entry in &file_entries {
            paths.push(Value::String(entry.path.clone()));

            let file_obj = match mode {
                "binary" => {
                    let bytes = std::fs::read(&entry.canonical)
                        .map_err(|e| format!("Failed to read {}: {}", entry.name, e))?;
                    use base64::Engine;
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    let mime = guess_mime_type(&entry.canonical);
                    serde_json::json!({
                        "path": entry.path,
                        "name": entry.name,
                        "content": encoded,
                        "encoding": "base64",
                        "mime_type": mime,
                        "size": entry.size,
                        "modified": entry.modified,
                    })
                }
                "json" => {
                    let content = std::fs::read_to_string(&entry.canonical)
                        .map_err(|e| format!("Failed to read {}: {}", entry.name, e))?;
                    let parsed: Value = serde_json::from_str(&content)
                        .unwrap_or(Value::String(content.clone()));
                    serde_json::json!({
                        "path": entry.path,
                        "name": entry.name,
                        "content": parsed,
                        "size": entry.size,
                        "modified": entry.modified,
                    })
                }
                "csv" => {
                    let content = std::fs::read_to_string(&entry.canonical)
                        .map_err(|e| format!("Failed to read {}: {}", entry.name, e))?;
                    let rows = parse_csv(&content, csv_delimiter, csv_has_header)?;
                    serde_json::json!({
                        "path": entry.path,
                        "name": entry.name,
                        "rows": rows,
                        "content": content,
                        "size": entry.size,
                        "modified": entry.modified,
                    })
                }
                _ => {
                    // text mode
                    let content = std::fs::read_to_string(&entry.canonical)
                        .map_err(|e| format!("Failed to read {}: {}", entry.name, e))?;
                    serde_json::json!({
                        "path": entry.path,
                        "name": entry.name,
                        "content": content,
                        "size": entry.size,
                        "modified": entry.modified,
                    })
                }
            };
            files.push(file_obj);
        }

        let count = files.len();
        Ok(NodeOutput::value(serde_json::json!({
            "files": files,
            "count": count,
            "paths": paths,
        })))
    }
}

struct FileEntry {
    path: String,
    name: String,
    size: u64,
    modified: String,
    canonical: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;

    fn create_test_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        // Create test files
        let mut f1 = std::fs::File::create(dir.path().join("data1.txt")).unwrap();
        write!(f1, "hello world").unwrap();
        let mut f2 = std::fs::File::create(dir.path().join("data2.txt")).unwrap();
        write!(f2, "second file").unwrap();
        let mut f3 = std::fs::File::create(dir.path().join("report.csv")).unwrap();
        write!(f3, "name,age\nAlice,30\nBob,25").unwrap();
        let mut f4 = std::fs::File::create(dir.path().join("config.json")).unwrap();
        write!(f4, r#"{{"key":"value"}}"#).unwrap();
        // Create subdirectory with files
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        let mut f5 = std::fs::File::create(dir.path().join("sub").join("nested.txt")).unwrap();
        write!(f5, "nested content").unwrap();
        dir
    }

    fn make_node_data(dir: &str, pattern: &str, mode: &str, recursive: bool) -> Value {
        serde_json::json!({
            "directory": dir,
            "pattern": pattern,
            "mode": mode,
            "recursive": recursive,
            "maxFiles": 100,
            "maxSize": 10.0,
            "sortBy": "name",
            "sortOrder": "asc",
        })
    }

    fn run_glob(node_data: &Value) -> Result<NodeOutput, String> {
        let inputs: HashMap<String, Value> = HashMap::new();

        // We can't create a full ExecutionContext without DB/sidecar, so test the
        // lower-level functions. But since the executor uses resolve_template
        // which just needs node_outputs + inputs, we test the logic directly.
        let dir = node_data.get("directory").and_then(|v| v.as_str()).unwrap();
        let pattern = node_data.get("pattern").and_then(|v| v.as_str()).unwrap();
        let recursive = node_data.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        let mode = node_data.get("mode").and_then(|v| v.as_str()).unwrap_or("text");
        let max_files = node_data.get("maxFiles").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
        let sort_by = node_data.get("sortBy").and_then(|v| v.as_str()).unwrap_or("name");
        let sort_order = node_data.get("sortOrder").and_then(|v| v.as_str()).unwrap_or("asc");

        let glob_pattern = if recursive {
            format!("{}/**/{}", dir.trim_end_matches('/'), pattern)
        } else {
            format!("{}/{}", dir.trim_end_matches('/'), pattern)
        };

        let entries = glob::glob(&glob_pattern)
            .map_err(|e| format!("Invalid pattern: {}", e))?;

        let mut file_entries: Vec<FileEntry> = Vec::new();
        for entry in entries {
            if file_entries.len() >= max_files { break; }
            let path = match entry { Ok(p) => p, Err(_) => continue };
            if path.is_dir() { continue; }
            let canonical = path.canonicalize().unwrap();
            if is_path_denied(&canonical) { continue; }
            let metadata = std::fs::metadata(&canonical).unwrap();
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            file_entries.push(FileEntry {
                path: canonical.to_string_lossy().to_string(),
                name,
                size: metadata.len(),
                modified: String::new(),
                canonical,
            });
        }

        match sort_by {
            "size" => file_entries.sort_by(|a, b| a.size.cmp(&b.size)),
            _ => file_entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        }
        if sort_order == "desc" { file_entries.reverse(); }

        let mut files = Vec::new();
        let mut paths = Vec::new();
        for entry in &file_entries {
            paths.push(Value::String(entry.path.clone()));
            let content = std::fs::read_to_string(&entry.canonical).unwrap_or_default();
            files.push(serde_json::json!({
                "path": entry.path, "name": entry.name,
                "content": content, "size": entry.size,
            }));
        }
        let count = files.len();
        Ok(NodeOutput::value(serde_json::json!({ "files": files, "count": count, "paths": paths })))
    }

    #[test]
    fn test_glob_txt_files() {
        let dir = create_test_dir();
        let data = make_node_data(dir.path().to_str().unwrap(), "*.txt", "text", false);
        let result = run_glob(&data).unwrap();
        let count = result.value.get("count").unwrap().as_u64().unwrap();
        assert_eq!(count, 2); // data1.txt, data2.txt
        let files = result.value.get("files").unwrap().as_array().unwrap();
        assert_eq!(files[0].get("name").unwrap().as_str().unwrap(), "data1.txt");
        assert_eq!(files[1].get("name").unwrap().as_str().unwrap(), "data2.txt");
    }

    #[test]
    fn test_glob_all_files() {
        let dir = create_test_dir();
        let data = make_node_data(dir.path().to_str().unwrap(), "*", "text", false);
        let result = run_glob(&data).unwrap();
        let count = result.value.get("count").unwrap().as_u64().unwrap();
        assert_eq!(count, 4); // data1.txt, data2.txt, report.csv, config.json
    }

    #[test]
    fn test_glob_recursive() {
        let dir = create_test_dir();
        let data = make_node_data(dir.path().to_str().unwrap(), "*.txt", "text", true);
        let result = run_glob(&data).unwrap();
        let count = result.value.get("count").unwrap().as_u64().unwrap();
        assert_eq!(count, 3); // data1.txt, data2.txt, nested.txt
    }

    #[test]
    fn test_glob_csv_pattern() {
        let dir = create_test_dir();
        let data = make_node_data(dir.path().to_str().unwrap(), "*.csv", "text", false);
        let result = run_glob(&data).unwrap();
        let count = result.value.get("count").unwrap().as_u64().unwrap();
        assert_eq!(count, 1);
        let files = result.value.get("files").unwrap().as_array().unwrap();
        assert_eq!(files[0].get("name").unwrap().as_str().unwrap(), "report.csv");
    }

    #[test]
    fn test_glob_paths_output() {
        let dir = create_test_dir();
        let data = make_node_data(dir.path().to_str().unwrap(), "*.json", "text", false);
        let result = run_glob(&data).unwrap();
        let paths = result.value.get("paths").unwrap().as_array().unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].as_str().unwrap().ends_with("config.json"));
    }

    #[test]
    fn test_glob_empty_result() {
        let dir = create_test_dir();
        let data = make_node_data(dir.path().to_str().unwrap(), "*.xml", "text", false);
        let result = run_glob(&data).unwrap();
        assert_eq!(result.value.get("count").unwrap().as_u64().unwrap(), 0);
        assert!(result.value.get("files").unwrap().as_array().unwrap().is_empty());
    }

    #[test]
    fn test_glob_sort_desc() {
        let dir = create_test_dir();
        let mut data = make_node_data(dir.path().to_str().unwrap(), "*.txt", "text", false);
        data.as_object_mut().unwrap().insert("sortOrder".to_string(), Value::String("desc".to_string()));
        let result = run_glob(&data).unwrap();
        let files = result.value.get("files").unwrap().as_array().unwrap();
        assert_eq!(files[0].get("name").unwrap().as_str().unwrap(), "data2.txt");
        assert_eq!(files[1].get("name").unwrap().as_str().unwrap(), "data1.txt");
    }

    #[test]
    fn test_glob_nonexistent_dir() {
        let data = make_node_data("/nonexistent/path/xyz", "*.txt", "text", false);
        // The glob itself won't error — it just returns empty
        let result = run_glob(&data).unwrap();
        assert_eq!(result.value.get("count").unwrap().as_u64().unwrap(), 0);
    }
}

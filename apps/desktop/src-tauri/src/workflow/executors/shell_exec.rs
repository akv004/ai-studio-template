use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;

pub struct ShellExecExecutor;

#[async_trait::async_trait]
impl NodeExecutor for ShellExecExecutor {
    fn node_type(&self) -> &str { "shell_exec" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        // Resolve command: incoming "command" edge > config command
        let config_cmd = node_data.get("command").and_then(|v| v.as_str()).unwrap_or("");
        let command = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("command").and_then(|v| v.as_str()).unwrap_or(config_cmd).to_string()
            } else if let Some(s) = inc.as_str() {
                s.to_string()
            } else {
                config_cmd.to_string()
            }
        } else {
            config_cmd.to_string()
        };
        // Merge incoming object fields into inputs for template resolution
        // (same pattern as TransformExecutor)
        let mut local_inputs = ctx.inputs.clone();
        if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                for (k, v) in obj {
                    local_inputs.insert(k.clone(), v.clone());
                }
            }
        }
        let command = resolve_template(&command, ctx.node_outputs, &local_inputs);

        if command.is_empty() {
            return Err("Shell Exec: command is empty".into());
        }

        let shell = node_data.get("shell").and_then(|v| v.as_str()).unwrap_or("bash");
        let timeout_secs = node_data.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);
        let working_dir = node_data.get("workingDir").and_then(|v| v.as_str()).unwrap_or("");

        // Build environment: start clean, add only essentials
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("HOME".to_string(),
            dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default());
        env_vars.insert("PATH".to_string(), "/usr/local/bin:/usr/bin:/bin".to_string());

        // Add configured env vars
        if let Some(extra_env) = node_data.get("envVars").and_then(|v| v.as_object()) {
            for (k, v) in extra_env {
                if let Some(s) = v.as_str() {
                    env_vars.insert(k.clone(), s.to_string());
                }
            }
        }

        // Resolve stdin from incoming edge
        let stdin_data = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("stdin").and_then(|v| v.as_str()).map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Build command
        let mut cmd = tokio::process::Command::new(shell);
        cmd.arg("-c").arg(&command);
        cmd.env_clear();
        for (k, v) in &env_vars {
            cmd.env(k, v);
        }

        if !working_dir.is_empty() {
            cmd.current_dir(working_dir);
        }

        // Configure stdin
        if stdin_data.is_some() {
            cmd.stdin(std::process::Stdio::piped());
        } else {
            cmd.stdin(std::process::Stdio::null());
        }
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn with new session for clean process group cleanup
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn shell process: {}", e))?;

        // Save PID before wait_with_output() consumes child
        let child_pid = child.id();

        // Write stdin if provided
        if let Some(stdin_str) = stdin_data {
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin.write_all(stdin_str.as_bytes()).await;
                drop(stdin);
            }
        }

        // Wait with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            child.wait_with_output(),
        ).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(NodeOutput::value(serde_json::json!({
                    "stdout": stdout,
                    "stderr": stderr,
                    "exit_code": exit_code,
                })))
            }
            Ok(Err(e)) => {
                Err(format!("Shell process error: {}", e))
            }
            Err(_) => {
                // Timeout — kill the process group using saved PID
                #[cfg(unix)]
                {
                    if let Some(pid) = child_pid {
                        unsafe { libc::kill(-(pid as i32), libc::SIGKILL); }
                    }
                }
                Err(format!("Command timed out after {}s", timeout_secs))
            }
        }
    }
}

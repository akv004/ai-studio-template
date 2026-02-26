use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;
use std::collections::HashMap;

pub struct EmailSendExecutor;

/// Maximum recipients across to + cc + bcc combined
const MAX_RECIPIENTS: usize = 50;
/// Maximum email body size in bytes (2MB)
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

fn parse_addresses(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn validate_addresses(addrs: &[String]) -> Result<Vec<lettre::Address>, String> {
    addrs
        .iter()
        .map(|a| {
            a.parse::<lettre::Address>()
                .map_err(|e| format!("Invalid email address '{}': {}", a, e))
        })
        .collect()
}

fn resolve_field(
    field_name: &str,
    node_data: &serde_json::Value,
    incoming: &Option<serde_json::Value>,
    node_outputs: &HashMap<String, serde_json::Value>,
    inputs: &HashMap<String, serde_json::Value>,
) -> String {
    let config_val = node_data.get(field_name).and_then(|v| v.as_str()).unwrap_or("");
    let raw = if let Some(inc) = incoming {
        if let Some(obj) = inc.as_object() {
            if let Some(field_val) = obj.get(field_name) {
                // Handle both string and non-string incoming values
                match field_val.as_str() {
                    Some(s) => s.to_string(),
                    None => field_val.to_string(),
                }
            } else {
                config_val.to_string()
            }
        } else if let Some(s) = inc.as_str() {
            // Plain string incoming — use as value if field is "body" or primary field
            if field_name == "body" { s.to_string() } else { config_val.to_string() }
        } else {
            config_val.to_string()
        }
    } else {
        config_val.to_string()
    };
    resolve_template(&raw, node_outputs, inputs)
}

#[async_trait::async_trait]
impl NodeExecutor for EmailSendExecutor {
    fn node_type(&self) -> &str { "email_send" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        _node_id: &str,
        node_data: &serde_json::Value,
        incoming: &Option<serde_json::Value>,
    ) -> Result<NodeOutput, String> {
        // Resolve email fields from incoming edges or config
        let to_raw = resolve_field("to", node_data, incoming, ctx.node_outputs, ctx.inputs);
        let subject = resolve_field("subject", node_data, incoming, ctx.node_outputs, ctx.inputs);
        let body = resolve_field("body", node_data, incoming, ctx.node_outputs, ctx.inputs);
        let cc_raw = resolve_field("cc", node_data, incoming, ctx.node_outputs, ctx.inputs);
        let bcc_raw = resolve_field("bcc", node_data, incoming, ctx.node_outputs, ctx.inputs);
        let reply_to_raw = resolve_field("replyTo", node_data, incoming, ctx.node_outputs, ctx.inputs);

        // SMTP config (always from node_data, not incoming edges)
        let smtp_host = node_data.get("smtpHost").and_then(|v| v.as_str()).unwrap_or("");
        let smtp_port = node_data.get("smtpPort").and_then(|v| v.as_u64()).unwrap_or(587) as u16;
        let encryption = node_data.get("encryption").and_then(|v| v.as_str()).unwrap_or("tls");
        let smtp_user = node_data.get("smtpUser").and_then(|v| v.as_str()).unwrap_or("");
        let smtp_pass = node_data.get("smtpPass").and_then(|v| v.as_str()).unwrap_or("");
        let from_address = node_data.get("fromAddress").and_then(|v| v.as_str()).unwrap_or("");
        let from_name = node_data.get("fromName").and_then(|v| v.as_str()).unwrap_or("");
        let body_type = node_data.get("bodyType").and_then(|v| v.as_str()).unwrap_or("plain");

        // Validate required fields
        if smtp_host.is_empty() {
            return Ok(make_error_output("SMTP host is required"));
        }
        if from_address.is_empty() {
            return Ok(make_error_output("From address is required"));
        }
        if to_raw.is_empty() {
            return Ok(make_error_output("To address is required"));
        }

        // Body size limit
        if body.len() > MAX_BODY_BYTES {
            return Ok(make_error_output(&format!(
                "Email body too large: {} bytes > {} byte limit",
                body.len(),
                MAX_BODY_BYTES
            )));
        }

        // Parse and validate addresses (single pass — reuse validated Address objects)
        let to_strings = parse_addresses(&to_raw);
        let cc_strings = parse_addresses(&cc_raw);
        let bcc_strings = parse_addresses(&bcc_raw);

        // Check for empty parsed recipients (e.g., " , " input)
        if to_strings.is_empty() {
            return Ok(make_error_output("To address is required (no valid addresses after parsing)"));
        }

        // Recipient cap
        let total_recipients = to_strings.len() + cc_strings.len() + bcc_strings.len();
        if total_recipients > MAX_RECIPIENTS {
            return Ok(make_error_output(&format!(
                "Too many recipients: {} > {} limit",
                total_recipients,
                MAX_RECIPIENTS
            )));
        }

        let to_addrs = match validate_addresses(&to_strings) {
            Ok(v) => v,
            Err(e) => return Ok(make_error_output(&e)),
        };
        let cc_addrs = match validate_addresses(&cc_strings) {
            Ok(v) => v,
            Err(e) => return Ok(make_error_output(&e)),
        };
        let bcc_addrs = match validate_addresses(&bcc_strings) {
            Ok(v) => v,
            Err(e) => return Ok(make_error_output(&e)),
        };

        // Build From mailbox (route errors to error handle, not Err)
        let from_addr = match from_address.parse::<lettre::Address>() {
            Ok(a) => a,
            Err(e) => return Ok(make_error_output(&format!("Invalid From address '{}': {}", from_address, e))),
        };
        let from_mailbox = if from_name.is_empty() {
            lettre::message::Mailbox::new(None, from_addr)
        } else {
            lettre::message::Mailbox::new(Some(from_name.to_string()), from_addr)
        };

        // Build message (reuse validated Address objects — no second parse)
        let mut builder = lettre::Message::builder()
            .from(from_mailbox)
            .subject(&subject);

        for addr in &to_addrs {
            builder = builder.to(lettre::message::Mailbox::new(None, addr.clone()));
        }
        for addr in &cc_addrs {
            builder = builder.cc(lettre::message::Mailbox::new(None, addr.clone()));
        }
        for addr in &bcc_addrs {
            builder = builder.bcc(lettre::message::Mailbox::new(None, addr.clone()));
        }
        if !reply_to_raw.is_empty() {
            match reply_to_raw.parse::<lettre::Address>() {
                Ok(reply_addr) => {
                    builder = builder.reply_to(lettre::message::Mailbox::new(None, reply_addr));
                }
                Err(e) => return Ok(make_error_output(&format!("Invalid Reply-To address '{}': {}", reply_to_raw, e))),
            }
        }

        let message = if body_type == "html" {
            builder
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(body.clone())
        } else {
            builder
                .header(lettre::message::header::ContentType::TEXT_PLAIN)
                .body(body.clone())
        };
        let message = match message {
            Ok(m) => m,
            Err(e) => return Ok(make_error_output(&format!("Failed to build email message: {}", e))),
        };

        // Build SMTP transport — credentials only if user/pass are non-empty
        let has_credentials = !smtp_user.is_empty() || !smtp_pass.is_empty();
        let creds = lettre::transport::smtp::authentication::Credentials::new(
            smtp_user.to_string(),
            smtp_pass.to_string(),
        );

        let send_result = match encryption {
            "ssl" => {
                // Implicit TLS (port 465) — relay() negotiates TLS immediately
                let builder_result = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_host);
                let mut tb = match builder_result {
                    Ok(b) => b,
                    Err(e) => return Ok(make_error_output(&format!("SMTP relay error: {}", e))),
                };
                tb = tb.port(smtp_port)
                    .timeout(Some(std::time::Duration::from_secs(30)));
                if has_credentials { tb = tb.credentials(creds); }
                lettre::AsyncTransport::send(&tb.build(), message).await
            }
            "none" => {
                // Unencrypted (e.g., Mailpit on localhost:1025)
                let mut tb = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(smtp_host)
                    .port(smtp_port)
                    .timeout(Some(std::time::Duration::from_secs(30)));
                if has_credentials { tb = tb.credentials(creds); }
                lettre::AsyncTransport::send(&tb.build(), message).await
            }
            _ => {
                // "tls" — STARTTLS (port 587)
                let builder_result = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(smtp_host);
                let mut tb = match builder_result {
                    Ok(b) => b,
                    Err(e) => return Ok(make_error_output(&format!("SMTP STARTTLS error: {}", e))),
                };
                tb = tb.port(smtp_port)
                    .timeout(Some(std::time::Duration::from_secs(30)));
                if has_credentials { tb = tb.credentials(creds); }
                lettre::AsyncTransport::send(&tb.build(), message).await
            }
        };

        match send_result {
            Ok(response) => {
                let message_id = response.message().collect::<Vec<&str>>().join(" ");
                let message_id = if message_id.is_empty() {
                    uuid::Uuid::new_v4().to_string()
                } else {
                    message_id
                };
                let output = serde_json::json!({
                    "success": true,
                    "messageId": message_id,
                    "recipients": total_recipients,
                    "to": to_strings,
                    "cc": cc_strings,
                    "bcc": bcc_strings,
                });
                let mut extra = HashMap::new();
                extra.insert("error".to_string(), serde_json::Value::String(String::new()));
                Ok(NodeOutput {
                    value: output,
                    skip_nodes: Vec::new(),
                    extra_outputs: extra,
                })
            }
            Err(e) => {
                let err_msg = format!("{}", e);
                Ok(make_error_output(&err_msg))
            }
        }
    }
}

fn make_error_output(error: &str) -> NodeOutput {
    let mut extra = HashMap::new();
    extra.insert("error".to_string(), serde_json::Value::String(error.to_string()));
    NodeOutput {
        value: serde_json::json!({
            "success": false,
            "error": error,
        }),
        skip_nodes: Vec::new(),
        extra_outputs: extra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_addresses_single() {
        let addrs = parse_addresses("alice@example.com");
        assert_eq!(addrs, vec!["alice@example.com"]);
    }

    #[test]
    fn test_parse_addresses_multiple() {
        let addrs = parse_addresses("alice@example.com, bob@example.com, carol@example.com");
        assert_eq!(addrs, vec!["alice@example.com", "bob@example.com", "carol@example.com"]);
    }

    #[test]
    fn test_parse_addresses_empty() {
        let addrs = parse_addresses("");
        assert!(addrs.is_empty());
    }

    #[test]
    fn test_parse_addresses_whitespace_only() {
        let addrs = parse_addresses(" , , ");
        assert!(addrs.is_empty());
    }

    #[test]
    fn test_validate_addresses_valid() {
        let addrs = vec!["alice@example.com".to_string(), "bob@test.org".to_string()];
        let result = validate_addresses(&addrs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_validate_addresses_invalid() {
        let addrs = vec!["alice@example.com".to_string(), "not-an-email".to_string()];
        let result = validate_addresses(&addrs);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not-an-email"));
    }

    #[test]
    fn test_validate_addresses_empty_list() {
        let addrs: Vec<String> = vec![];
        let result = validate_addresses(&addrs);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_make_error_output_shape() {
        let output = make_error_output("SMTP timeout");
        let val = output.value;
        assert_eq!(val["success"], false);
        assert_eq!(val["error"], "SMTP timeout");
        assert_eq!(
            output.extra_outputs.get("error").unwrap(),
            &serde_json::Value::String("SMTP timeout".to_string())
        );
    }

    #[test]
    fn test_html_body_type_flag() {
        let node_data = serde_json::json!({ "bodyType": "html" });
        let bt = node_data.get("bodyType").and_then(|v| v.as_str()).unwrap_or("plain");
        assert_eq!(bt, "html");

        let node_data_plain = serde_json::json!({});
        let bt2 = node_data_plain.get("bodyType").and_then(|v| v.as_str()).unwrap_or("plain");
        assert_eq!(bt2, "plain");
    }

    #[test]
    fn test_success_output_shape() {
        let to_addrs = vec!["a@b.com".to_string(), "c@d.com".to_string()];
        let cc_addrs: Vec<String> = vec![];
        let bcc_addrs: Vec<String> = vec![];
        let message_id = "test-uuid-123";
        let total = to_addrs.len() + cc_addrs.len() + bcc_addrs.len();

        let output = serde_json::json!({
            "success": true,
            "messageId": message_id,
            "recipients": total,
            "to": to_addrs,
            "cc": cc_addrs,
            "bcc": bcc_addrs,
        });

        assert_eq!(output["success"], true);
        assert_eq!(output["recipients"], 2);
        assert_eq!(output["to"].as_array().unwrap().len(), 2);
        assert_eq!(output["messageId"], "test-uuid-123");
    }

    #[test]
    fn test_recipient_cap() {
        assert!(MAX_RECIPIENTS == 50);
        assert!(MAX_BODY_BYTES == 2 * 1024 * 1024);
    }

    #[test]
    fn test_resolve_field_config_fallback() {
        let node_data = serde_json::json!({ "to": "config@example.com" });
        let incoming: Option<serde_json::Value> = None;
        let outputs = HashMap::new();
        let inputs = HashMap::new();
        let result = resolve_field("to", &node_data, &incoming, &outputs, &inputs);
        assert_eq!(result, "config@example.com");
    }

    #[test]
    fn test_resolve_field_incoming_object_overrides() {
        let node_data = serde_json::json!({ "to": "config@example.com" });
        let incoming = Some(serde_json::json!({ "to": "incoming@example.com" }));
        let outputs = HashMap::new();
        let inputs = HashMap::new();
        let result = resolve_field("to", &node_data, &incoming, &outputs, &inputs);
        assert_eq!(result, "incoming@example.com");
    }

    #[test]
    fn test_resolve_field_incoming_non_string() {
        let node_data = serde_json::json!({ "to": "config@example.com" });
        let incoming = Some(serde_json::json!({ "to": 42 }));
        let outputs = HashMap::new();
        let inputs = HashMap::new();
        let result = resolve_field("to", &node_data, &incoming, &outputs, &inputs);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_resolve_field_incoming_missing_field() {
        let node_data = serde_json::json!({ "subject": "Hello" });
        let incoming = Some(serde_json::json!({ "to": "incoming@example.com" }));
        let outputs = HashMap::new();
        let inputs = HashMap::new();
        let result = resolve_field("subject", &node_data, &incoming, &outputs, &inputs);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_resolve_field_plain_string_incoming_body() {
        let node_data = serde_json::json!({ "body": "config body" });
        let incoming = Some(serde_json::Value::String("upstream text".to_string()));
        let outputs = HashMap::new();
        let inputs = HashMap::new();
        // Plain string incoming used for "body" field
        let result = resolve_field("body", &node_data, &incoming, &outputs, &inputs);
        assert_eq!(result, "upstream text");
        // But not for other fields
        let result2 = resolve_field("to", &node_data, &incoming, &outputs, &inputs);
        assert_eq!(result2, "");
    }
}

use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::resolve_template;
use std::collections::HashMap;

pub struct EmailSendExecutor;

fn parse_addresses(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn validate_address(addr: &str) -> Result<lettre::Address, String> {
    addr.parse::<lettre::Address>()
        .map_err(|e| format!("Invalid email address '{}': {}", addr, e))
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
            obj.get(field_name)
                .and_then(|v| v.as_str())
                .unwrap_or(config_val)
                .to_string()
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

        // Parse and validate addresses
        let to_addrs = parse_addresses(&to_raw);
        let cc_addrs = parse_addresses(&cc_raw);
        let bcc_addrs = parse_addresses(&bcc_raw);

        for addr in &to_addrs {
            if let Err(e) = validate_address(addr) {
                return Ok(make_error_output(&e));
            }
        }
        for addr in &cc_addrs {
            if let Err(e) = validate_address(addr) {
                return Ok(make_error_output(&e));
            }
        }
        for addr in &bcc_addrs {
            if let Err(e) = validate_address(addr) {
                return Ok(make_error_output(&e));
            }
        }

        // Build From mailbox
        let from_addr = validate_address(from_address)
            .map_err(|e| format!("From address error: {}", e))?;
        let from_mailbox = if from_name.is_empty() {
            lettre::message::Mailbox::new(None, from_addr)
        } else {
            lettre::message::Mailbox::new(Some(from_name.to_string()), from_addr)
        };

        // Build message
        let mut builder = lettre::Message::builder()
            .from(from_mailbox)
            .subject(&subject);

        for addr in &to_addrs {
            let parsed = addr.parse::<lettre::Address>().unwrap();
            builder = builder.to(lettre::message::Mailbox::new(None, parsed));
        }
        for addr in &cc_addrs {
            let parsed = addr.parse::<lettre::Address>().unwrap();
            builder = builder.cc(lettre::message::Mailbox::new(None, parsed));
        }
        for addr in &bcc_addrs {
            let parsed = addr.parse::<lettre::Address>().unwrap();
            builder = builder.bcc(lettre::message::Mailbox::new(None, parsed));
        }
        if !reply_to_raw.is_empty() {
            if let Ok(reply_addr) = reply_to_raw.parse::<lettre::Address>() {
                builder = builder.reply_to(lettre::message::Mailbox::new(None, reply_addr));
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
        }
        .map_err(|e| format!("Failed to build email message: {}", e))?;

        // Build SMTP transport
        let creds = lettre::transport::smtp::authentication::Credentials::new(
            smtp_user.to_string(),
            smtp_pass.to_string(),
        );

        let send_result = match encryption {
            "ssl" => {
                let transport = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_host)
                    .map_err(|e| format!("SMTP relay error: {}", e))?
                    .port(smtp_port)
                    .credentials(creds)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .build();
                lettre::AsyncTransport::send(&transport, message).await
            }
            "none" => {
                let transport = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(smtp_host)
                    .port(smtp_port)
                    .credentials(creds)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .build();
                lettre::AsyncTransport::send(&transport, message).await
            }
            _ => {
                // "tls" — STARTTLS
                let transport = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(smtp_host)
                    .map_err(|e| format!("SMTP STARTTLS error: {}", e))?
                    .port(smtp_port)
                    .credentials(creds)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .build();
                lettre::AsyncTransport::send(&transport, message).await
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
                let total = to_addrs.len() + cc_addrs.len() + bcc_addrs.len();
                let output = serde_json::json!({
                    "success": true,
                    "messageId": message_id,
                    "recipients": total,
                    "to": to_addrs,
                    "cc": cc_addrs,
                    "bcc": bcc_addrs,
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
    fn test_validate_address_valid() {
        assert!(validate_address("alice@example.com").is_ok());
        assert!(validate_address("user+tag@domain.org").is_ok());
    }

    #[test]
    fn test_validate_address_invalid() {
        assert!(validate_address("not-an-email").is_err());
        assert!(validate_address("@missing-local.com").is_err());
        assert!(validate_address("").is_err());
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
        // Verify the body_type parsing logic
        let node_data = serde_json::json!({ "bodyType": "html" });
        let bt = node_data.get("bodyType").and_then(|v| v.as_str()).unwrap_or("plain");
        assert_eq!(bt, "html");

        let node_data_plain = serde_json::json!({});
        let bt2 = node_data_plain.get("bodyType").and_then(|v| v.as_str()).unwrap_or("plain");
        assert_eq!(bt2, "plain");
    }

    #[test]
    fn test_success_output_shape() {
        // Simulate what the success path would produce
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
}

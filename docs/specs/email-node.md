# Email Send Node

**Status**: DRAFT — pending peer review
**Phase**: 4C (automation canvas)
**Priority**: P0 — critical gap vs web-based workflow builders
**Author**: AI Studio PM
**Date**: 2026-02-25

---

## Problem Statement

AI Studio has 19 node types for processing, routing, and I/O — but no way to send notifications. Real automation workflows need to email results: alert on failures, send reports, notify approvers. Every major workflow builder (web-based or otherwise) includes email as a core integration. Without it, AI Studio cannot demo end-to-end automation.

---

## Design

### Node Type: `email_send`

A processing node (not a source/trigger) that sends an email via SMTP when reached in the workflow.

### Config

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| smtpHost | string | required | SMTP server hostname |
| smtpPort | int | 587 | Port (587=TLS, 465=SSL, 25=plain) |
| smtpUser | string | required | SMTP username |
| smtpPass | string | required | SMTP password (stored in node config for now; migrates to Connections Manager later) |
| encryption | enum | tls | tls / ssl / none |
| fromAddress | string | required | Sender email address |
| fromName | string | optional | Sender display name |

### Input Handles

| Handle | Type | Required | Description |
|--------|------|----------|-------------|
| to | text | yes | Recipient email(s), comma-separated |
| subject | text | yes | Email subject line |
| body | text | yes | Email body (plain text or HTML) |
| cc | text | no | CC recipients |
| bcc | text | no | BCC recipients |
| replyTo | text | no | Reply-to address |

All input handles support template resolution: `{{node_id.output}}` syntax works.

### Output Handles

| Handle | Type | Description |
|--------|------|-------------|
| output | json | `{success: true, messageId: "...", recipients: 3}` |
| error | text | Error message if send failed (empty on success) |

### Template Resolution

Subject and body fields support the standard `{{variable}}` template syntax:

```
Subject: Report for {{input.date}}
Body: {{llm_1.output}}
```

This means upstream LLM output can flow directly into the email body.

---

## Node UI

```
+---------------------------+
|   EMAIL SEND              |
| [to] ← recipients        |
| [subject] ← subject      |
| [body] ← email body      |
|                           |
|   SMTP: smtp.gmail.com   |
|   From: alerts@acme.com  |
|                           |
|         output → [json]   |
|          error → [text]   |
+---------------------------+
```

### Config Panel

```
┌──────────────────────────────────────┐
│  Email Send Configuration            │
│                                      │
│  SMTP Server                         │
│  Host:       [smtp.gmail.com    ]    │
│  Port:       [587               ]    │
│  Encryption: [TLS ▼]                │
│  Username:   [user@gmail.com    ]    │
│  Password:   [••••••••          ]    │
│  From:       [alerts@acme.com   ]    │
│  From Name:  [AI Studio         ]    │
│                                      │
│  [Test Connection]  ✓ EHLO OK (92ms) │
│                                      │
│  --- Email ---                       │
│  To:         [{{input.to}}      ]    │
│  CC:         [                  ]    │
│  Subject:    [Report: {{input}} ]    │
│  Body:       [{{llm_1.output}} ]    │
│  Body Type:  [HTML ▼]               │
│                                      │
│  [Send Test Email]                   │
└──────────────────────────────────────┘
```

---

## Executor (Rust)

### Implementation: `EmailSendExecutor`

Uses the `lettre` crate (Rust-native SMTP client, well-maintained, async support).

```
1. Resolve template variables in to/subject/body/cc/bcc
2. Validate email addresses (basic regex)
3. Build SMTP transport:
   - TLS: lettre::transport::smtp::AsyncSmtpTransport with STARTTLS
   - SSL: Direct TLS connection
   - None: Unencrypted (warn in logs)
4. Build message:
   - From: config.fromAddress
   - To: resolved recipients
   - Subject: resolved subject
   - Body: plain text or HTML (based on bodyType config)
5. Send via async transport
6. Return {success, messageId, recipients} or error
```

### Error Handling

| Error | Behavior |
|-------|----------|
| SMTP connection failed | Return error output, workflow continues |
| Auth failed (535) | Return error with "SMTP authentication failed" |
| Invalid recipient | Return error with specific address |
| Timeout (default 30s) | Return error with "SMTP timeout" |
| TLS handshake failure | Return error with "TLS negotiation failed" |

The node does NOT stop the workflow on error — it outputs the error on the `error` handle so downstream nodes can react.

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `lettre` | 0.11 | SMTP client (async, TLS, DKIM) |

`lettre` is the de facto Rust email crate: 1.8k stars, actively maintained, supports async-std and tokio.

---

## Security Considerations

| Concern | Mitigation |
|---------|-----------|
| SMTP password in node config | Stored as JSON in SQLite. Future: migrate to Connections Manager with AES-256 encryption |
| Password visible in graph JSON | Config panel masks it. Export/template strips credentials. |
| Email injection (header injection) | lettre validates headers, rejects newlines in subject/addresses |
| Rate limiting | No built-in rate limit — SMTP server enforces its own limits |
| TLS downgrade | Default to TLS. Warn if encryption=none. |

---

## Tests (Rust unit tests)

| # | Test | What |
|---|------|------|
| 1 | Valid email address parsing | Single + multiple comma-separated |
| 2 | Invalid email address rejection | Missing @, empty string |
| 3 | Template resolution in subject/body | `{{node.output}}` substitution |
| 4 | Output format on success | Correct JSON shape |
| 5 | Output format on error | Error handle populated |
| 6 | HTML body type detection | bodyType=html wraps in Content-Type |

Note: Actual SMTP send tests require a mail server — use `lettre::transport::stub::StubTransport` for unit tests.

---

## Implementation Plan

### Session 1: Executor + Node (1 session)
- [ ] Add `lettre = "0.11"` to Cargo.toml
- [ ] Create `src/workflow/executors/email_send.rs`
- [ ] Implement `EmailSendExecutor` with template resolution
- [ ] Register in executor registry
- [ ] Add validation: email_send not as source node
- [ ] 6 unit tests
- [ ] UI: `EmailSendNode.tsx` + config panel
- [ ] Add to node palette (Communication category)

---

## Future Enhancements (Not This Session)

- **Attachments**: Accept file data from upstream File Read/Glob nodes
- **HTML templates**: Built-in email templates with variable injection
- **Connections Manager integration**: SMTP credentials from encrypted connection store
- **Email Receive trigger**: IMAP polling trigger (separate spec)
- **Calendar invite**: ICS attachment generation

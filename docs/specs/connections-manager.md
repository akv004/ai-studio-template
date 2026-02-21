# Connections Manager

**Status**: PLANNED
**Phase**: 5B (production-ready)
**Priority**: P0 — prerequisite for SQL Query, HTTP Request, Webhook, and integration nodes
**Author**: AI Studio PM
**Date**: 2026-02-21

---

## Problem Statement

Multiple planned features (SQL Query node, Webhook triggers, HTTP Request node, Content Enricher) all need to connect to external services. Each would need its own credential storage, connection testing, and configuration UI. Without a unified system, every integration node reinvents connection management — duplicating code, inconsistent security, and poor UX.

---

## Design Principles

1. **One place for all connections** — Settings → Connections tab
2. **Credentials encrypted at rest** — never stored as plaintext in SQLite
3. **Connection testing** — verify before saving
4. **Reusable** — one connection used by multiple nodes/workflows
5. **Secure by default** — credentials never exposed to UI after save, never logged

---

## Connection Types

### Database

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | yes | Display name ("prod-db", "analytics") |
| driver | enum | yes | postgresql / mysql / sqlite / mssql |
| host | string | yes* | Hostname (* not needed for SQLite) |
| port | int | no | Default per driver (5432, 3306, 1433) |
| database | string | yes | Database name or file path (SQLite) |
| username | string | yes* | (* not needed for SQLite) |
| password | string | yes* | Encrypted at rest |
| ssl | bool | no | Enable SSL/TLS (default: true for remote) |
| sslCert | string | no | Path to CA cert file |
| options | json | no | Driver-specific options |

### HTTP / API

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | yes | Display name ("GitHub API", "Slack") |
| baseUrl | string | yes | Base URL (e.g. `https://api.example.com/v1`) |
| authType | enum | yes | none / bearer / basic / api_key / oauth2 |
| authToken | string | cond | Bearer token (encrypted) |
| username | string | cond | For basic auth |
| password | string | cond | For basic auth (encrypted) |
| apiKeyHeader | string | cond | Header name for API key auth (e.g. `X-API-Key`) |
| apiKeyValue | string | cond | API key value (encrypted) |
| headers | json | no | Default headers for all requests |
| timeout | int | no | Request timeout in seconds (default: 30) |

### SMTP (Email)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | yes | Display name ("Gmail", "SendGrid") |
| host | string | yes | SMTP host |
| port | int | yes | 587 (TLS) or 465 (SSL) |
| username | string | yes | |
| password | string | yes | Encrypted at rest |
| fromAddress | string | yes | Default "from" address |
| encryption | enum | yes | tls / ssl / none |

### Webhook Endpoint (Outbound)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | yes | Display name ("Slack webhook", "PagerDuty") |
| url | string | yes | Webhook URL |
| method | enum | no | POST (default) / PUT |
| authType | enum | no | none / bearer / hmac |
| authSecret | string | cond | Token or HMAC secret (encrypted) |
| headers | json | no | Custom headers |

---

## Credential Encryption

### At-Rest Encryption

- Credentials encrypted using **AES-256-GCM** before storage in SQLite
- Encryption key derived from a **machine-specific key** stored in OS keychain:
  - macOS: Keychain Access (`security` command)
  - Linux: `libsecret` / GNOME Keyring
  - Windows: Windows Credential Manager
- Key name: `ai-studio-connection-key`

### Flow

```
User enters password
    ↓
UI sends to Rust via IPC (Tauri secure channel)
    ↓
Rust encrypts with AES-256-GCM (key from OS keychain)
    ↓
Encrypted blob stored in SQLite
    ↓
On read: Rust decrypts, passes to sidecar via env/header
    ↓
UI never sees decrypted credentials after initial save
```

### What the UI Sees

After saving a connection, credential fields show `••••••••` with a "Change" button. The actual values are never sent back to the UI.

---

## Connection Testing

Every connection type supports a "Test Connection" action:

| Type | Test Method |
|------|------------|
| Database | Execute `SELECT 1` (or equivalent) |
| HTTP/API | Send `HEAD` or `GET` to baseUrl |
| SMTP | EHLO handshake |
| Webhook | Send test payload with `X-Test: true` header |

**Test result UI**:
```
[Test Connection]
  ✓ Connected successfully (145ms)
  — or —
  ✗ Connection failed: timeout after 10s
  — or —
  ✗ Authentication failed: 401 Unauthorized
```

---

## Settings UI: Connections Tab

```
┌─────────────────────────────────────────────────────┐
│  Settings → Connections                              │
│                                                      │
│  [+ Add Connection]                                  │
│                                                      │
│  ┌─────────────────────────────────────────────────┐ │
│  │ 🗄 prod-db                    PostgreSQL        │ │
│  │   localhost:5432/myapp        Last tested: 2m   │ │
│  │   Used by: 3 workflows                          │ │
│  │   [Test] [Edit] [Delete]                        │ │
│  ├─────────────────────────────────────────────────┤ │
│  │ 🌐 GitHub API                 HTTP / Bearer     │ │
│  │   https://api.github.com      Last tested: 1hr  │ │
│  │   Used by: 1 workflow                           │ │
│  │   [Test] [Edit] [Delete]                        │ │
│  ├─────────────────────────────────────────────────┤ │
│  │ 🔔 Slack Webhook             Webhook            │ │
│  │   https://hooks.slack.com/... Last tested: 1d   │ │
│  │   Used by: 2 workflows                          │ │
│  │   [Test] [Edit] [Delete]                        │ │
│  └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### Add/Edit Connection Dialog

```
┌──────────────────────────────────────┐
│  Add Connection                       │
│                                      │
│  Type: [Database ▼]                  │
│                                      │
│  Name:     [prod-db            ]     │
│  Driver:   [PostgreSQL ▼]            │
│  Host:     [localhost          ]     │
│  Port:     [5432               ]     │
│  Database: [myapp              ]     │
│  Username: [admin              ]     │
│  Password: [••••••••           ]     │
│  SSL:      [✓]                       │
│                                      │
│  [Test Connection]  ✓ Connected (82ms)│
│                                      │
│  [Cancel]              [Save]        │
└──────────────────────────────────────┘
```

---

## Node Integration

### How Nodes Reference Connections

Nodes that need external access get a `connectionId` field in their config:

```
┌─────────────────────────────┐
│  SQL QUERY                   │
│                              │
│  Connection: [prod-db ▼]    │
│  Query:                      │
│  SELECT * FROM users         │
│  WHERE created > {{input}}   │
│                              │
│        rows → [json]         │
│       count → [float]        │
└─────────────────────────────┘
```

The dropdown shows all connections of the appropriate type (e.g., SQL Query only shows Database connections).

### Runtime Resolution

When a node executes:
1. Rust reads `connectionId` from node config
2. Rust decrypts credentials from SQLite
3. For database nodes: Rust connects directly (using `sqlx` crate)
4. For HTTP/webhook nodes: Rust passes auth headers to sidecar, or executes directly
5. Credentials are never passed to the UI or logged

---

## Data Model

### New table: `connections`

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Connection ID |
| name | TEXT | Display name |
| conn_type | TEXT | database / http / smtp / webhook |
| config | TEXT (JSON) | Non-sensitive config (host, port, driver, baseUrl) |
| credentials | BLOB | AES-256-GCM encrypted sensitive fields |
| last_tested | TEXT | ISO timestamp (nullable) |
| test_status | TEXT | success / failed / untested |
| created_at | TEXT | ISO timestamp |
| updated_at | TEXT | ISO timestamp |

### Storage Split

Non-sensitive fields (host, port, driver) stored in `config` as JSON — readable by UI.
Sensitive fields (password, tokens, keys) stored in `credentials` as encrypted blob — never sent to UI.

---

## IPC Commands

```rust
// CRUD
create_connection(conn_type, name, config, credentials) -> Connection
update_connection(id, name?, config?, credentials?) -> Connection
delete_connection(id) -> ()
list_connections(conn_type?) -> Vec<ConnectionSummary>  // no credentials in response
get_connection(id) -> ConnectionDetail  // no credentials in response

// Testing
test_connection(id) -> TestResult  // {success, latency_ms, error?}

// Internal (not exposed to UI)
resolve_connection(id) -> DecryptedConnection  // used by executors only
```

---

## Implementation Plan

### Phase 1: Core Infrastructure (1 session)
- [ ] Rust: `connections` table + migration
- [ ] Rust: AES-256-GCM encryption with OS keychain key
- [ ] Rust: CRUD commands (create, list, get, update, delete)
- [ ] Rust: Credential split (config vs encrypted credentials)
- [ ] 8 tests (CRUD, encryption roundtrip, no credentials in list response)

### Phase 2: Connection Testing + UI (1 session)
- [ ] Rust: Test connection per type (SELECT 1, HEAD request, EHLO)
- [ ] UI: Settings → Connections tab
- [ ] UI: Add/Edit connection dialog per type
- [ ] UI: Test Connection button with result display
- [ ] UI: Masked credential fields
- [ ] 5 tests (test per type, timeout, auth failure)

### Phase 3: Node Integration (1 session)
- [ ] Rust: `resolve_connection()` for executors
- [ ] UI: Connection dropdown component (filtered by type)
- [ ] Wire into SQL Query node (from killer-features.md spec)
- [ ] Wire into HTTP Request node
- [ ] 5 tests (executor resolution, missing connection error, type mismatch)

---

## Security Considerations

| Concern | Mitigation |
|---------|-----------|
| Credentials in SQLite | AES-256-GCM encryption, key in OS keychain |
| Credentials in UI | Never sent after initial save. Masked display. |
| Credentials in logs | Redacted in all log output |
| Credentials in sidecar | Passed via request header, not env var. Not persisted. |
| Connection to remote DBs | SSL/TLS by default. User must explicitly disable. |
| SQL injection via node | Parameterized queries enforced by SQL executor |
| Stale connections | Test button + last_tested timestamp. Warn if untested >7 days. |

---

## Dependencies

| Feature | Depends On | Blocks |
|---------|-----------|--------|
| OS Keychain access | Tauri `keyring` plugin or `keyring` crate | All credential storage |
| Database connections | `sqlx` crate (Rust) | SQL Query node |
| HTTP connections | `reqwest` (already used) | HTTP Request, Webhook, Enricher nodes |
| SMTP | `lettre` crate (Rust) | Email alert sink |

---

## Migration Path

Existing provider API keys (stored as `provider.{id}.{field}` in settings table) remain as-is. The Connections Manager is for **external service** credentials, not LLM provider keys. In a future version, provider keys could migrate to the Connections Manager for consistency.

# Dual-Mode Deployment: Desktop + Server

**Status**: PLANNED — future phase, needs peer review before implementation
**Phase**: 5+ (post-automation canvas)
**Priority**: P1 — enables cloud deployment without discarding desktop-first architecture
**Author**: AI Studio PM
**Date**: 2026-02-26
**Related specs**: `architecture.md` (current 3-layer design)

---

## Problem Statement

AI Studio is a desktop-native app (Tauri 2). This is its strength — local-first, data never leaves the machine, no cloud dependency. Most users will run it this way.

But enterprise teams and demo scenarios need cloud deployment: run on an Azure VM, expose the UI via browser, share across a team. Today, the only option is running the full desktop app on a VM via RDP — functional but not a real cloud product.

**The goal**: Ship a single codebase that produces two artifacts:
1. **Desktop installer** — Tauri app, same as today (primary)
2. **Docker image** — Web-accessible server, same engine, HTTP transport (secondary)

No features lost. No code duplicated. Same 237+ Rust tests cover both modes.

---

## Architecture: Core + Shell Pattern

### Current Architecture (Desktop Only)

```
UI (React 19) ──Tauri IPC──→ Rust Backend ──HTTP──→ Python Sidecar
                              (73 commands)           (FastAPI)
                              (SQLite)
                              (workflow engine)
                              (RAG, triggers, routing)
```

### Proposed Architecture (Dual Mode)

```
                    ┌──────────────────────────┐
                    │     ai-studio-core       │
                    │                          │
                    │  commands/  (73 pure fn)  │
                    │  workflow/  (engine)      │
                    │  rag/      (index/search) │
                    │  triggers/ (cron/webhook) │
                    │  routing/  (hybrid intel) │
                    │  db/       (SQLite/PG)    │
                    │                          │
                    │  No Tauri deps           │
                    │  No Axum deps            │
                    │  Just pure Rust           │
                    └────────┬─────────────────┘
                             │
                ┌────────────┼────────────────┐
                │                             │
    ┌───────────▼───────────┐    ┌────────────▼────────────┐
    │   desktop shell       │    │    server shell          │
    │                       │    │                          │
    │  #[tauri::command]    │    │  Axum HTTP routes        │
    │  wrappers (1-3 lines  │    │  wrappers (1-3 lines    │
    │  each, call core::)   │    │  each, call core::)     │
    │                       │    │                          │
    │  Tauri webview        │    │  Static file server      │
    │  Sidecar spawning     │    │  JWT auth middleware     │
    │  File dialogs         │    │  CORS                    │
    │  System tray          │    │  Health check endpoint   │
    │                       │    │                          │
    │  cargo build          │    │  docker build            │
    │  --features desktop   │    │  --features server       │
    └───────────────────────┘    └──────────────────────────┘
```

### What Lives Where

| Component | Crate | Lines (approx) | Tauri-dependent? |
|-----------|-------|-----------------|------------------|
| Agent CRUD | core/commands/agents.rs | 120 | No |
| Session CRUD | core/commands/sessions.rs | 150 | No |
| Workflow CRUD | core/commands/workflows.rs | 100 | No |
| Workflow engine | core/workflow/ | 800+ | No |
| RAG module | core/rag/ | 500+ | No |
| Trigger manager | core/triggers/ | 300+ | No |
| Routing engine | core/routing.rs | 200 | No |
| DB layer | core/db.rs | 150 | No |
| Cost calculation | core/commands/events.rs | 100 | No |
| Sidecar proxy | core/sidecar.rs | 200 | **Partially** — spawning is Tauri-specific, HTTP proxy is not |
| IPC wrappers | desktop/ipc.rs | 200 | Yes (thin) |
| Axum routes | server/routes.rs | 200 | No (thin) |
| Auth middleware | server/auth.rs | 100 | No |

**Key insight**: ~2500 lines of core logic, ~200 lines of shell wrapper per mode. 93% of Rust code is shared.

---

## Cargo Workspace Structure

```
apps/desktop/src-tauri/
├── Cargo.toml              ← workspace root
├── crates/
│   ├── core/
│   │   ├── Cargo.toml      ← [lib] no Tauri, no Axum deps
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db.rs
│   │   │   ├── error.rs     ← unified AppError type
│   │   │   ├── sidecar.rs   ← HTTP client only (no spawning)
│   │   │   ├── commands/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── agents.rs
│   │   │   │   ├── sessions.rs
│   │   │   │   ├── workflows.rs
│   │   │   │   ├── events.rs
│   │   │   │   ├── settings.rs
│   │   │   │   ├── plugins.rs
│   │   │   │   ├── templates.rs
│   │   │   │   ├── triggers.rs
│   │   │   │   └── rag.rs
│   │   │   ├── workflow/
│   │   │   │   ├── mod.rs        ← engine
│   │   │   │   ├── executors/    ← all node executors
│   │   │   │   ├── validation.rs
│   │   │   │   ├── templates.rs
│   │   │   │   └── live.rs
│   │   │   ├── rag/
│   │   │   │   ├── chunker.rs
│   │   │   │   ├── index.rs
│   │   │   │   ├── search.rs
│   │   │   │   └── format.rs
│   │   │   ├── triggers/
│   │   │   │   ├── manager.rs
│   │   │   │   ├── webhook.rs
│   │   │   │   └── cron.rs
│   │   │   └── routing.rs
│   │   └── tests/           ← ALL 237+ tests live here
│   │
│   ├── desktop/
│   │   ├── Cargo.toml       ← depends on core + tauri
│   │   └── src/
│   │       ├── main.rs       ← tauri::Builder
│   │       ├── lib.rs        ← setup, plugin registration
│   │       ├── ipc.rs        ← #[tauri::command] wrappers
│   │       └── sidecar.rs    ← Tauri sidecar spawning
│   │
│   └── server/
│       ├── Cargo.toml       ← depends on core + axum + tower
│       └── src/
│           ├── main.rs       ← Axum server + static files
│           ├── routes.rs     ← HTTP route handlers
│           ├── auth.rs       ← JWT middleware
│           ├── ws.rs         ← WebSocket for events (replaces Tauri events)
│           └── sidecar.rs    ← Process spawning (non-Tauri)
```

---

## Core Crate: Command Signatures

Every command becomes a pure async function with explicit dependencies:

```rust
// crates/core/src/commands/agents.rs

use crate::db::Db;
use crate::error::AppError;

pub async fn create_agent(
    db: &Db,
    name: &str,
    provider: &str,
    model: &str,
    system_prompt: &str,
    routing_mode: Option<&str>,
    routing_rules: Option<&str>,
) -> Result<Agent, AppError> {
    let id = uuid();
    let now = utc_now();
    db.execute(
        "INSERT INTO agents (id, name, provider, model, system_prompt, ...) VALUES (?, ?, ...)",
        params![id, name, provider, model, system_prompt, ...],
    ).await?;
    Ok(Agent { id, name: name.to_string(), ... })
}

pub async fn list_agents(db: &Db) -> Result<Vec<Agent>, AppError> {
    // ...
}
```

---

## Desktop Shell: Thin IPC Wrappers

```rust
// crates/desktop/src/ipc.rs

use ai_studio_core as core;
use tauri::State;

#[tauri::command]
pub async fn create_agent(
    db: State<'_, core::db::Db>,
    name: String,
    provider: String,
    model: String,
    system_prompt: String,
    routing_mode: Option<String>,
    routing_rules: Option<String>,
) -> Result<core::Agent, String> {
    core::commands::create_agent(
        &db, &name, &provider, &model, &system_prompt,
        routing_mode.as_deref(), routing_rules.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}
```

Each wrapper: 5-10 lines. Pure delegation. No logic.

---

## Server Shell: Axum HTTP Routes

```rust
// crates/server/src/routes.rs

use ai_studio_core as core;
use axum::{extract::State, Json};

#[derive(Deserialize)]
struct CreateAgentReq {
    name: String,
    provider: String,
    model: String,
    system_prompt: String,
    routing_mode: Option<String>,
    routing_rules: Option<String>,
}

async fn create_agent(
    State(ctx): State<AppContext>,
    Json(req): Json<CreateAgentReq>,
) -> Result<Json<core::Agent>, AppError> {
    let agent = core::commands::create_agent(
        &ctx.db, &req.name, &req.provider, &req.model, &req.system_prompt,
        req.routing_mode.as_deref(), req.routing_rules.as_deref(),
    ).await?;
    Ok(Json(agent))
}

// Route registration
pub fn api_routes() -> Router<AppContext> {
    Router::new()
        // Agents
        .route("/api/agents", post(create_agent).get(list_agents))
        .route("/api/agents/:id", get(get_agent).put(update_agent).delete(delete_agent))
        // Sessions
        .route("/api/sessions", post(create_session).get(list_sessions))
        .route("/api/sessions/:id", get(get_session).delete(delete_session))
        // ... 73 routes total, each 5-10 lines
        // Workflows
        .route("/api/workflows", post(create_workflow).get(list_workflows))
        .route("/api/workflows/:id", get(get_workflow).put(update_workflow).delete(delete_workflow))
        .route("/api/workflows/:id/run", post(run_workflow))
        .route("/api/workflows/:id/live/start", post(start_live_workflow))
        .route("/api/workflows/:id/live/stop", post(stop_live_workflow))
        // Triggers
        .route("/api/triggers", post(create_trigger).get(list_triggers))
        .route("/api/triggers/:id/arm", post(arm_trigger))
        .route("/api/triggers/:id/disarm", post(disarm_trigger))
        // Settings
        .route("/api/settings", get(get_all_settings))
        .route("/api/settings/:key", put(set_setting))
        // RAG
        .route("/api/rag/index", post(index_folder))
        .route("/api/rag/search", post(search_index))
        .route("/api/rag/stats/:id", get(get_index_stats))
        // ... etc
}
```

---

## Server Shell: Auth Middleware

Desktop mode needs no auth (single user, local machine). Server mode needs JWT:

```rust
// crates/server/src/auth.rs

use axum::middleware;
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Deserialize)]
struct Claims {
    sub: String,    // user ID
    exp: usize,     // expiry
    role: String,   // admin / viewer
}

async fn auth_middleware(
    State(ctx): State<AppContext>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match token {
        Some(t) => {
            let claims = decode::<Claims>(t, &ctx.jwt_key, &Validation::default())
                .map_err(|_| StatusCode::UNAUTHORIZED)?;
            // Inject user context
            req.extensions_mut().insert(claims.claims);
            Ok(next.run(req).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
```

Auth options for server mode:

| Method | Complexity | Best for |
|--------|-----------|----------|
| Static API key (env var) | Minimal | Internal demos, single team |
| JWT with local user table | Medium | Small teams, self-hosted |
| OAuth2 / Azure AD | Higher | Enterprise, SSO integration |

**Phase 1**: Start with static API key (simplest). Upgrade to JWT later.

---

## Server Shell: WebSocket Events

Desktop uses Tauri's `emit()` for real-time events (node states, live feed, streaming). Server mode replaces this with WebSocket:

```rust
// crates/server/src/ws.rs

use axum::extract::ws::{WebSocket, WebSocketUpgrade};

async fn ws_handler(ws: WebSocketUpgrade, State(ctx): State<AppContext>) -> Response {
    ws.on_upgrade(|socket| handle_ws(socket, ctx))
}

async fn handle_ws(mut socket: WebSocket, ctx: AppContext) {
    let mut rx = ctx.event_bus.subscribe();
    while let Ok(event) = rx.recv().await {
        let msg = serde_json::to_string(&event).unwrap();
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
```

Core emits events to a `tokio::broadcast` channel. Desktop shell forwards to Tauri `emit()`. Server shell forwards to WebSocket clients.

---

## UI Transport Adapter

Replace scattered `isTauri()` checks with a unified backend adapter:

```typescript
// apps/ui/src/lib/backend.ts

interface Backend {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
    listen(event: string, handler: (payload: unknown) => void): () => void;
}

// Auto-detected at startup
const backend: Backend = isTauri() ? tauriBackend : httpBackend;

// --- Tauri backend (existing behavior) ---
const tauriBackend: Backend = {
    async invoke(command, args) {
        const { invoke } = await import('@tauri-apps/api/core');
        return invoke(command, args);
    },
    listen(event, handler) {
        // Tauri event listener
    },
};

// --- HTTP backend (new) ---
const httpBackend: Backend = {
    async invoke(command, args) {
        // Map command name to REST endpoint
        const { method, path } = commandToRoute(command);
        const res = await fetch(`/api${path}`, {
            method,
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${getToken()}`,
            },
            body: method !== 'GET' ? JSON.stringify(args) : undefined,
        });
        if (!res.ok) throw new Error(await res.text());
        return res.json();
    },
    listen(event, handler) {
        // WebSocket event listener
        const ws = getWebSocket();
        ws.addEventListener('message', (e) => {
            const data = JSON.parse(e.data);
            if (data.event === event) handler(data.payload);
        });
        return () => ws.removeEventListener('message', handler);
    },
};

export default backend;
```

**Store migration**: Replace all `invoke()` calls in `store.ts` with `backend.invoke()`. Same API, different transport.

---

## Docker Deployment

### Dockerfile

```dockerfile
# Stage 1: Build Rust server
FROM rust:1.78-slim AS rust-builder
WORKDIR /app
COPY apps/desktop/src-tauri/ .
RUN cargo build --release --features server -p ai-studio-server

# Stage 2: Build React UI
FROM node:20-slim AS ui-builder
WORKDIR /app
COPY apps/ui/ .
RUN npm install -g pnpm && pnpm install && pnpm build

# Stage 3: Runtime
FROM python:3.11-slim
WORKDIR /app

# Rust server binary
COPY --from=rust-builder /app/target/release/ai-studio-server /usr/local/bin/

# React static build
COPY --from=ui-builder /app/dist/ /app/static/

# Python sidecar
COPY apps/sidecar/ /app/sidecar/
RUN pip install -r /app/sidecar/requirements.txt

# Startup script
COPY deploy/start.sh /app/start.sh
RUN chmod +x /app/start.sh

EXPOSE 8080
ENV AI_STUDIO_MODE=server
ENV AI_STUDIO_PORT=8080
ENV SIDECAR_PORT=8765
ENV DATABASE_URL=/data/ai-studio.db

VOLUME /data

CMD ["/app/start.sh"]
```

### start.sh

```bash
#!/bin/bash
# Start sidecar in background
cd /app/sidecar
python -m uvicorn server:app --host 0.0.0.0 --port ${SIDECAR_PORT} &

# Start Rust server (serves API + static UI)
ai-studio-server \
    --port ${AI_STUDIO_PORT} \
    --static-dir /app/static \
    --database ${DATABASE_URL} \
    --sidecar-url http://localhost:${SIDECAR_PORT}
```

### docker-compose.yml (for Azure VM or local)

```yaml
version: '3.8'
services:
  ai-studio:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ai-studio-data:/data
    environment:
      - AI_STUDIO_AUTH_KEY=your-secret-key
      - AZURE_OPENAI_API_KEY=${AZURE_OPENAI_API_KEY}
      - AZURE_OPENAI_ENDPOINT=${AZURE_OPENAI_ENDPOINT}
    restart: unless-stopped

volumes:
  ai-studio-data:
```

### Azure Deployment Options

| Option | Effort | Cost | Best for |
|--------|--------|------|----------|
| **Azure VM + Docker** | Low | $70-140/mo | Demo, small team |
| **Azure Container Instance** | Low | Pay-per-use | Intermittent demos |
| **Azure App Service (container)** | Medium | $50-100/mo | Always-on, auto-scaling |
| **Azure Kubernetes (AKS)** | Higher | Variable | Multi-tenant, enterprise |

**Recommended for demo**: Azure Container Instance — spin up in 2 minutes, pay only when running.

```bash
az container create \
    --resource-group ai-studio-demo \
    --name ai-studio \
    --image ghcr.io/akv004/ai-studio:latest \
    --ports 8080 \
    --cpu 2 --memory 4 \
    --environment-variables \
        AI_STUDIO_AUTH_KEY=demo-key \
        AZURE_OPENAI_API_KEY=$KEY \
    --dns-name-label ai-studio-demo
# Access at: http://ai-studio-demo.eastus.azurecontainer.io:8080
```

---

## Feature Parity Matrix

| Feature | Desktop | Server | Notes |
|---------|---------|--------|-------|
| Workflow canvas | Yes | Yes | Same React UI |
| All 22+ node types | Yes | Yes | Core crate, shared |
| Workflow execution | Yes | Yes | Core engine |
| Live mode | Yes | Yes | WebSocket events instead of Tauri events |
| Streaming output | Yes | Yes | SSE works over HTTP |
| Inspector | Yes | Yes | Same UI, data from same DB |
| RAG Knowledge Base | Yes | Yes | Core crate |
| Triggers (cron/webhook) | Yes | Yes | Core crate |
| MCP tools | Yes | Partial | stdio-based MCP needs process spawning (works in Docker) |
| Plugins | Yes | Partial | Subprocess plugins work, UI plugins need review |
| File dialogs | Native | HTML `<input type="file">` | Different UX, same result |
| System tray | Yes | No | Not applicable for web |
| Offline mode | Yes | Depends | Server needs network for LLM providers |
| Multi-user | No | Yes (with auth) | JWT + user context |
| Auto-update | Tauri updater | Docker image pull | Different mechanism |

---

## Database: SQLite vs PostgreSQL

**Phase 1 (server mode)**: Keep SQLite with Docker volume mount. Simple, same schema, no migration.

**Phase 2 (if multi-user needed)**: Add PostgreSQL support via compile-time feature flag.

```rust
// crates/core/src/db.rs

#[cfg(feature = "sqlite")]
pub type Db = SqliteDb;

#[cfg(feature = "postgres")]
pub type Db = PgDb;

// Both implement the same trait
#[async_trait]
pub trait Database {
    async fn execute(&self, sql: &str, params: &[&dyn ToSql]) -> Result<(), AppError>;
    async fn query_one<T: FromRow>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<T, AppError>;
    async fn query_all<T: FromRow>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Vec<T>, AppError>;
}
```

SQLite is fine for single-user server mode (team of 1-5). PostgreSQL only needed for true multi-tenant.

---

## Sidecar Management: Desktop vs Server

| Aspect | Desktop | Server |
|--------|---------|--------|
| Spawning | Tauri `Command::new()` with sidecar plugin | Docker entrypoint / supervisord / separate container |
| Auth | Random token in env var | Same — token passed via env var |
| Health check | Tauri polls `/health` on startup | Docker healthcheck or startup probe |
| Lifecycle | Tauri starts/stops with app | Docker manages process lifecycle |
| Port | Dynamic (Tauri picks available port) | Fixed (env var, default 8765) |

The sidecar HTTP interface is identical in both modes. Only the process management differs.

---

## Migration Path: What Changes When

### Step 1: Extract Core Crate (non-breaking)

Move all business logic from `src/` to `crates/core/src/`. Desktop shell wraps with `#[tauri::command]`. All 237+ tests move to core. **Desktop app still works identically.**

Estimated diff:
- ~50 files moved/renamed
- ~200 lines of new IPC wrapper code in desktop shell
- 0 logic changes
- All tests pass in core crate

### Step 2: Build Server Shell (additive)

New `crates/server/` with Axum routes. ~200 lines of route wrappers + auth + WebSocket. Build produces `ai-studio-server` binary.

### Step 3: UI Transport Adapter (non-breaking)

Replace `isTauri()` checks with `backend.invoke()`. Desktop mode unchanged. Server mode uses HTTP. ~100 lines changed in UI.

### Step 4: Docker + CI (additive)

Dockerfile, docker-compose, GitHub Actions to build both artifacts.

**Total new code**: ~800 lines (server shell + auth + Docker config)
**Total moved code**: ~2500 lines (core extraction, mechanical)
**Total changed code**: ~300 lines (UI adapter, IPC wrappers)

---

## Implementation Plan

### Phase A: Core Extraction (2 sessions)
- [ ] Create Cargo workspace with core/desktop/server crates
- [ ] Move all command handlers to core (pure functions)
- [ ] Move workflow engine, RAG, triggers to core
- [ ] Create AppError type (replaces String errors)
- [ ] Desktop shell: thin #[tauri::command] wrappers
- [ ] All tests pass in core crate
- [ ] Desktop app builds and works identically

### Phase B: Server Shell (2 sessions)
- [ ] Axum route handlers wrapping core functions
- [ ] Static file serving (React build)
- [ ] JWT auth middleware (start with API key)
- [ ] WebSocket event bridge
- [ ] Sidecar HTTP client (non-Tauri spawning)
- [ ] Health check endpoint
- [ ] Server binary builds and starts

### Phase C: UI Adapter (1 session)
- [ ] Create `backend.ts` transport adapter
- [ ] Replace all `invoke()` calls in store with `backend.invoke()`
- [ ] Replace Tauri event listeners with WebSocket
- [ ] File upload via HTML input (server mode)
- [ ] Test full UI against Axum server in browser

### Phase D: Docker + Deployment (1 session)
- [ ] Multi-stage Dockerfile
- [ ] docker-compose.yml
- [ ] GitHub Actions: build desktop + Docker image
- [ ] Azure Container Instance deploy script
- [ ] Deployment docs in README

### Phase E: Polish (1 session)
- [ ] Peer review (Gemini architecture + Codex implementation)
- [ ] E2E test in Docker
- [ ] Performance comparison (IPC vs HTTP latency)
- [ ] README deployment section update

**Total: ~7 sessions, low risk (mechanical refactor, no logic changes)**

---

## Security Considerations

| Concern | Desktop | Server |
|---------|---------|--------|
| Auth | Tauri security boundary (no external access) | JWT + API key required |
| Network | localhost only | Bind to 0.0.0.0, needs TLS termination (Nginx/Azure) |
| File access | Full local filesystem | Container filesystem only (volume mounts) |
| Secrets | Stored in node config (local SQLite) | Env vars or Azure Key Vault |
| CORS | Not applicable (same origin via webview) | Configured in Axum middleware |
| Rate limiting | Not needed (single user) | Needed for webhook/API endpoints |

---

## Open Questions (for peer review)

1. **Should server mode support MCP stdio tools?** Docker containers can spawn subprocesses, but discovery and permissions are different from desktop.
2. **Should we support PostgreSQL from day one, or start with SQLite in Docker?** SQLite is simpler but limits concurrent writes.
3. **Should the server mode be a separate npm script (`pnpm server`) or only Docker?** Running without Docker is useful for development.
4. **Multi-user data isolation**: Should each user see all workflows, or only their own? Shared team workspace vs isolated accounts?
5. **Plugin system in server mode**: Plugins spawn subprocesses — does this work in a container? Security implications?

---

## Success Criteria

1. `cargo build --features desktop` produces the same Tauri app as today
2. `cargo build --features server` produces an Axum binary that serves the React UI + REST API
3. `docker-compose up` starts a fully functional AI Studio accessible at `http://localhost:8080`
4. All 237+ Rust tests pass in the core crate (shared between both modes)
5. A user can create a workflow, add a cron trigger, arm it, and receive an email — all from a browser tab pointed at the Docker container
6. Desktop and server modes have the same React UI with no visible differences (except file dialogs)
